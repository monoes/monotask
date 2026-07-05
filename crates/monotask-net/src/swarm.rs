use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use iroh::{Endpoint, EndpointAddr, EndpointId};
use monotask_storage::Storage;
use crate::{NetCommand, NetConfig, NetError, NetEvent};
use crate::sync_protocol::{read_cbor, write_cbor, SyncRequest, SyncResponse, PROTOCOL_ALPN};

// ---------------------------------------------------------------------------
// Internal event: per-connection tasks → run_inner
// ---------------------------------------------------------------------------
enum PeerLifecycle {
    Disconnected { node_id_hex: String },
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------
pub(crate) async fn run(
    config: NetConfig,
    storage: Arc<Mutex<Storage>>,
    identity_bytes: [u8; 32],
    mut cmd_rx: mpsc::Receiver<NetCommand>,
    event_tx: mpsc::Sender<NetEvent>,
) {
    if let Err(e) = run_inner(config, storage, identity_bytes, &mut cmd_rx, &event_tx).await {
        tracing::error!("net task failed: {e}");
    }
}

async fn run_inner(
    config: NetConfig,
    storage: Arc<Mutex<Storage>>,
    identity_bytes: [u8; 32],
    cmd_rx: &mut mpsc::Receiver<NetCommand>,
    event_tx: &mpsc::Sender<NetEvent>,
) -> Result<(), NetError> {
    let endpoint = Arc::new(
        crate::behaviour::build_endpoint(identity_bytes, &config).await?,
    );

    // Shared mutable state accessed from both the main loop and spawned tasks.
    let connections: Arc<Mutex<HashMap<String, iroh::endpoint::Connection>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let sync_states: Arc<Mutex<HashMap<String, automerge::sync::State>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let my_spaces: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    // Internal lifecycle channel.
    let (lifecycle_tx, mut lifecycle_rx) = mpsc::channel::<PeerLifecycle>(64);

    // Peer addresses to re-dial; grows via AddPeer and saved_peers.txt.
    let mut bootstrap_peer_addrs: Vec<String> = load_saved_peers(&config.data_dir);
    for addr in &config.bootstrap_peers {
        if !bootstrap_peer_addrs.contains(addr) {
            bootstrap_peer_addrs.push(addr.clone());
        }
    }

    // Accept loop — runs as a separate Tokio task.
    {
        let ep = endpoint.clone();
        let conns = connections.clone();
        let states = sync_states.clone();
        let spaces = my_spaces.clone();
        let ev_tx = event_tx.clone();
        let lc_tx = lifecycle_tx.clone();
        let storage = storage.clone();
        tokio::spawn(async move {
            accept_loop(ep, conns, states, spaces, identity_bytes, ev_tx, lc_tx, storage).await;
        });
    }

    // LAN discovery loop.
    let our_node_id_hex = hex::encode(endpoint.id().as_bytes());
    {
        // Resolve the actual bound port (in case listen_port was 0 = OS-assigned).
        let actual_port = endpoint.bound_sockets()
            .into_iter()
            .filter_map(|a| if a.is_ipv4() { Some(a.port()) } else { None })
            .next()
            .unwrap_or(config.listen_port);
        let (lan_tx, mut lan_rx) = mpsc::channel::<crate::discovery::DiscoveredPeer>(32);
        let node_id = our_node_id_hex.clone();
        let port = actual_port;
        tokio::spawn(async move {
            crate::discovery::run_lan_discovery(node_id, port, lan_tx).await;
        });
        let ep = endpoint.clone();
        let conns = connections.clone();
        let states = sync_states.clone();
        let spaces = my_spaces.clone();
        let ev_tx = event_tx.clone();
        let lc_tx = lifecycle_tx.clone();
        let storage = storage.clone();
        tokio::spawn(async move {
            while let Some(discovered) = lan_rx.recv().await {
                // Only dial if not already connected.
                let already = conns.lock().unwrap().contains_key(&discovered.node_id_hex);
                if already { continue }
                if let Ok(node_id) = parse_node_id(&discovered.node_id_hex) {
                    let node_addr = EndpointAddr::new(node_id)
                        .with_ip_addr(discovered.addr);
                    dial_and_sync(
                        ep.clone(), node_addr, conns.clone(), states.clone(),
                        spaces.clone(), identity_bytes, ev_tx.clone(),
                        lc_tx.clone(), storage.clone(),
                    ).await;
                }
            }
        });
    }

    // Connect to bootstrap peers immediately at startup.
    for addr in bootstrap_peer_addrs.clone() {
        if let Some(node_addr) = parse_peer_addr(&addr) {
            let ep = endpoint.clone();
            let conns = connections.clone();
            let states = sync_states.clone();
            let spaces = my_spaces.clone();
            let ev_tx = event_tx.clone();
            let lc_tx = lifecycle_tx.clone();
            let storage = storage.clone();
            tokio::spawn(async move {
                dial_and_sync(ep, node_addr, conns, states, spaces, identity_bytes, ev_tx, lc_tx, storage).await;
            });
        }
    }

    let mut reconnect_tick = tokio::time::interval(Duration::from_secs(30));
    let mut reannounce = tokio::time::interval(Duration::from_secs(20 * 3600));
    let mut presence_tick = tokio::time::interval(Duration::from_secs(30));
    // Suppress the immediate first tick so we don't duplicate the startup dials above.
    reconnect_tick.reset();
    reannounce.reset();
    presence_tick.reset();

    loop {
        tokio::select! {
            Some(lc) = lifecycle_rx.recv() => {
                match lc {
                    PeerLifecycle::Disconnected { node_id_hex } => {
                        connections.lock().unwrap().remove(&node_id_hex);
                        sync_states.lock().unwrap()
                            .retain(|k, _| !k.contains(&node_id_hex));
                        let _ = event_tx.send(NetEvent::PeerDisconnected {
                            peer_id: node_id_hex,
                        }).await;
                    }
                }
            }

            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    NetCommand::Stop => {
                        tracing::info!("net: stopping");
                        endpoint.close().await;
                        return Ok(());
                    }

                    NetCommand::AnnounceSpaces { space_ids } => {
                        *my_spaces.lock().unwrap() = space_ids.clone();
                        // Re-Hello all connected peers so they get our updated space doc.
                        sync_states.lock().unwrap().retain(|k, _| !k.starts_with("i/"));
                        let conns: Vec<_> = connections.lock().unwrap()
                            .iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                        for (_, conn) in conns {
                            let storage = storage.clone();
                            let spaces = space_ids.clone();
                            let states = sync_states.clone();
                            let ev_tx = event_tx.clone();
                            tokio::spawn(async move {
                                initiate_hello_and_sync(
                                    &conn, &storage, &spaces, identity_bytes, &states, &ev_tx,
                                ).await;
                            });
                        }
                    }

                    NetCommand::TriggerSync { board_id } => {
                        let conns: Vec<_> = connections.lock().unwrap()
                            .iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                        if conns.is_empty() {
                            for addr in &bootstrap_peer_addrs {
                                if let Some(node_addr) = parse_peer_addr(addr) {
                                    tokio::spawn(dial_and_sync(
                                        endpoint.clone(), node_addr, connections.clone(),
                                        sync_states.clone(), my_spaces.clone(), identity_bytes,
                                        event_tx.clone(), lifecycle_tx.clone(), storage.clone(),
                                    ));
                                }
                            }
                        }
                        for (peer_id, conn) in conns {
                            let board_id = board_id.clone();
                            let storage = storage.clone();
                            let states = sync_states.clone();
                            let ev_tx = event_tx.clone();
                            tokio::spawn(async move {
                                sync_board_with_peer(
                                    &conn, &peer_id, &board_id, &storage, &states, &ev_tx,
                                ).await;
                            });
                        }
                    }

                    NetCommand::ForceRediscovery => {
                        sync_states.lock().unwrap().retain(|k, _| !k.starts_with("i/"));
                        let conns: Vec<_> = connections.lock().unwrap()
                            .iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                        let spaces = my_spaces.lock().unwrap().clone();
                        for (_, conn) in conns {
                            let storage = storage.clone();
                            let spaces = spaces.clone();
                            let states = sync_states.clone();
                            let ev_tx = event_tx.clone();
                            tokio::spawn(async move {
                                initiate_hello_and_sync(
                                    &conn, &storage, &spaces, identity_bytes, &states, &ev_tx,
                                ).await;
                            });
                        }
                    }

                    NetCommand::AddPeer { addr } => {
                        if !bootstrap_peer_addrs.contains(&addr) {
                            bootstrap_peer_addrs.push(addr.clone());
                            save_peer_addr(&config.data_dir, &addr);
                        }
                        if let Some(node_addr) = parse_peer_addr(&addr) {
                            dial_and_sync(
                                endpoint.clone(), node_addr,
                                connections.clone(), sync_states.clone(),
                                my_spaces.clone(), identity_bytes,
                                event_tx.clone(), lifecycle_tx.clone(), storage.clone(),
                            ).await;
                        } else {
                            tracing::warn!("net: invalid peer addr '{addr}'");
                        }
                    }

                    NetCommand::GetPeers { reply } => {
                        let peers = connections.lock().unwrap().keys().cloned().collect();
                        let _ = reply.send(peers);
                    }

                    NetCommand::GetListenAddrs { reply } => {
                        let addr = format!("{:?}", endpoint.addr());
                        let _ = reply.send(vec![addr]);
                    }

                    NetCommand::GetPeerPubkeys { reply } => {
                        // With iroh, the peer's NodeId IS their ed25519 public key.
                        let map = connections.lock().unwrap()
                            .keys()
                            .map(|id| (id.clone(), id.clone()))
                            .collect();
                        let _ = reply.send(map);
                    }
                }
            }

            _ = reconnect_tick.tick() => {
                if connections.lock().unwrap().is_empty() && !bootstrap_peer_addrs.is_empty() {
                    for addr in bootstrap_peer_addrs.clone() {
                        if let Some(node_addr) = parse_peer_addr(&addr) {
                            let ep = endpoint.clone();
                            let conns = connections.clone();
                            let states = sync_states.clone();
                            let spaces = my_spaces.clone();
                            let ev_tx = event_tx.clone();
                            let lc_tx = lifecycle_tx.clone();
                            let storage = storage.clone();
                            tokio::spawn(async move {
                                dial_and_sync(ep, node_addr, conns, states, spaces,
                                    identity_bytes, ev_tx, lc_tx, storage).await;
                            });
                        }
                    }
                }
            }

            _ = reannounce.tick() => {
                // iroh relay registration is maintained automatically.
                // LAN discovery handles periodic re-announcements.
            }

            _ = presence_tick.tick() => {
                // Broadcast our presence status to all connected peers.
                let my_pubkey = hex::encode({
                    let secret = iroh::SecretKey::from_bytes(&identity_bytes);
                    *secret.public().as_bytes()
                });
                let (my_status, my_display_name) = {
                    let guard = storage.lock().unwrap();
                    let status = guard.conn().query_row(
                        "SELECT presence, display_name FROM user_profile WHERE pk = 'local' LIMIT 1",
                        [], |r| Ok((r.get::<_, String>(0).unwrap_or_else(|_| "online".into()),
                                    r.get::<_, String>(1).unwrap_or_default()))
                    ).unwrap_or_else(|_| ("online".into(), String::new()));
                    status
                };
                let presence_req = SyncRequest::Presence {
                    pubkey: my_pubkey,
                    status: my_status,
                    display_name: my_display_name,
                };
                let conns: Vec<iroh::endpoint::Connection> = connections.lock().unwrap()
                    .values().cloned().collect();
                for conn in conns {
                    let req = presence_req.clone();
                    tokio::spawn(async move {
                        if let Ok((mut send, _recv)) = conn.open_bi().await {
                            let _ = write_cbor(&mut send, &req).await;
                        }
                    });
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Accept loop (runs as a separate task)
// ---------------------------------------------------------------------------
async fn accept_loop(
    endpoint: Arc<Endpoint>,
    connections: Arc<Mutex<HashMap<String, iroh::endpoint::Connection>>>,
    sync_states: Arc<Mutex<HashMap<String, automerge::sync::State>>>,
    my_spaces: Arc<Mutex<Vec<String>>>,
    identity_bytes: [u8; 32],
    event_tx: mpsc::Sender<NetEvent>,
    lifecycle_tx: mpsc::Sender<PeerLifecycle>,
    storage: Arc<Mutex<Storage>>,
) {
    while let Some(incoming) = endpoint.accept().await {
        let conn = match incoming.await {
            Ok(c) => c,
            Err(e) => { tracing::debug!("net: incoming connection failed: {e}"); continue }
        };
        let node_id_hex = hex::encode(conn.remote_id().as_bytes());
        connections.lock().unwrap().insert(node_id_hex.clone(), conn.clone());

        let _ = event_tx.send(NetEvent::PeerConnected { peer_id: node_id_hex.clone() }).await;

        // Initiate Hello on the new connection.
        let spaces = my_spaces.lock().unwrap().clone();
        if !spaces.is_empty() {
            let conn2 = conn.clone();
            let storage = storage.clone();
            let states = sync_states.clone();
            let ev_tx = event_tx.clone();
            tokio::spawn(async move {
                initiate_hello_and_sync(&conn2, &storage, &spaces, identity_bytes, &states, &ev_tx).await;
            });
        }

        // Per-connection incoming-stream handler.
        let lc_tx = lifecycle_tx.clone();
        let storage = storage.clone();
        let states = sync_states.clone();
        let spaces_arc = my_spaces.clone();
        tokio::spawn(async move {
            handle_incoming_streams(conn, node_id_hex, storage, states, spaces_arc, identity_bytes, lc_tx).await;
        });
    }
}

// ---------------------------------------------------------------------------
// Dial a peer and kick off Hello+sync (idempotent — skips if already connected)
// ---------------------------------------------------------------------------
async fn dial_and_sync(
    endpoint: Arc<Endpoint>,
    node_addr: EndpointAddr,
    connections: Arc<Mutex<HashMap<String, iroh::endpoint::Connection>>>,
    sync_states: Arc<Mutex<HashMap<String, automerge::sync::State>>>,
    my_spaces: Arc<Mutex<Vec<String>>>,
    identity_bytes: [u8; 32],
    event_tx: mpsc::Sender<NetEvent>,
    lifecycle_tx: mpsc::Sender<PeerLifecycle>,
    storage: Arc<Mutex<Storage>>,
) {
    let node_id_hex = hex::encode(node_addr.id.as_bytes());
    if connections.lock().unwrap().contains_key(&node_id_hex) { return }

    let conn = match endpoint.connect(node_addr, PROTOCOL_ALPN).await {
        Ok(c) => c,
        Err(e) => { tracing::debug!("net: connect to {node_id_hex:.8} failed: {e}"); return }
    };

    // Protocol version handshake — dial side opens the first stream.
    match conn.open_bi().await {
        Err(e) => { tracing::warn!("net: version handshake open_bi failed: {e}"); return }
        Ok((mut send, mut recv)) => {
            let hello = crate::sync_protocol::VersionHello { major: crate::sync_protocol::PROTOCOL_MAJOR };
            if let Err(e) = write_cbor(&mut send, &hello).await {
                tracing::warn!("net: write VersionHello failed: {e}"); return;
            }
            let reject: Option<crate::sync_protocol::VersionReject> = read_cbor(&mut recv).await.ok();
            if let Some(rej) = reject {
                if !rej.reason.is_empty() {
                    tracing::warn!("net: peer {node_id_hex:.8} rejected version {}: {} (peer speaks {})", rej.their_major, rej.reason, rej.our_major);
                    return;
                }
            }
        }
    }

    connections.lock().unwrap().insert(node_id_hex.clone(), conn.clone());
    let _ = event_tx.send(NetEvent::PeerConnected { peer_id: node_id_hex.clone() }).await;

    // Initiate Hello + board sync.
    let spaces = my_spaces.lock().unwrap().clone();
    if !spaces.is_empty() {
        let conn2 = conn.clone();
        let storage2 = storage.clone();
        let states2 = sync_states.clone();
        let ev_tx2 = event_tx.clone();
        tokio::spawn(async move {
            initiate_hello_and_sync(&conn2, &storage2, &spaces, identity_bytes, &states2, &ev_tx2).await;
        });
    }

    // Per-connection incoming-stream handler.
    let spaces_arc = my_spaces.clone();
    tokio::spawn(async move {
        handle_incoming_streams(
            conn, node_id_hex, storage, sync_states, spaces_arc,
            identity_bytes, lifecycle_tx,
        ).await;
    });
}

// ---------------------------------------------------------------------------
// Incoming stream handler (responder role for a single connection)
// ---------------------------------------------------------------------------
async fn handle_incoming_streams(
    conn: iroh::endpoint::Connection,
    peer_id: String,
    storage: Arc<Mutex<Storage>>,
    sync_states: Arc<Mutex<HashMap<String, automerge::sync::State>>>,
    my_spaces: Arc<Mutex<Vec<String>>>,
    _identity_bytes: [u8; 32],
    lifecycle_tx: mpsc::Sender<PeerLifecycle>,
) {
    let peer_pubkey: [u8; 32] = *conn.remote_id().as_bytes();

    // First stream from the dialer is always a version handshake.
    match conn.accept_bi().await {
        Err(e) => {
            tracing::debug!("net: version handshake stream from {peer_id:.8} closed: {e}");
            let _ = lifecycle_tx.send(PeerLifecycle::Disconnected { node_id_hex: peer_id }).await;
            return;
        }
        Ok((mut send, mut recv)) => {
            let hello: crate::sync_protocol::VersionHello = match read_cbor(&mut recv).await {
                Ok(h) => h,
                Err(e) => {
                    tracing::warn!("net: read VersionHello failed from {peer_id:.8}: {e}");
                    let _ = lifecycle_tx.send(PeerLifecycle::Disconnected { node_id_hex: peer_id }).await;
                    return;
                }
            };
            if hello.major != crate::sync_protocol::PROTOCOL_MAJOR {
                let reject = crate::sync_protocol::VersionReject {
                    reason: format!("incompatible major version: we speak {}, you speak {}", crate::sync_protocol::PROTOCOL_MAJOR, hello.major),
                    their_major: hello.major,
                    our_major: crate::sync_protocol::PROTOCOL_MAJOR,
                };
                let _ = write_cbor(&mut send, &reject).await;
                tracing::warn!("net: rejected {peer_id:.8} (speaks protocol v{}; we speak v{})", hello.major, crate::sync_protocol::PROTOCOL_MAJOR);
                let _ = lifecycle_tx.send(PeerLifecycle::Disconnected { node_id_hex: peer_id }).await;
                return;
            }
            // Send empty reject (= accepted) so the dialer can continue.
            let ok = crate::sync_protocol::VersionReject { reason: String::new(), their_major: hello.major, our_major: crate::sync_protocol::PROTOCOL_MAJOR };
            let _ = write_cbor(&mut send, &ok).await;
        }
    }

    loop {
        match conn.accept_bi().await {
            Err(e) => {
                tracing::debug!("net: connection from {peer_id:.8} closed: {e}");
                break;
            }
            Ok((mut send, mut recv)) => {
                let request: SyncRequest = match read_cbor(&mut recv).await {
                    Ok(r) => r,
                    Err(e) => { tracing::warn!("net: read request failed: {e}"); break }
                };
                let spaces = my_spaces.lock().unwrap().clone();
                let response = handle_one_request(
                    request, &peer_id, peer_pubkey, &storage, &sync_states, &spaces,
                );
                if let Err(e) = write_cbor(&mut send, &response).await {
                    tracing::warn!("net: write response failed: {e}");
                }
            }
        }
    }
    let _ = lifecycle_tx.send(PeerLifecycle::Disconnected { node_id_hex: peer_id }).await;
}

// ---------------------------------------------------------------------------
// Initiator role: Hello + board sync for all spaces
// ---------------------------------------------------------------------------
async fn initiate_hello_and_sync(
    conn: &iroh::endpoint::Connection,
    storage: &Arc<Mutex<Storage>>,
    my_spaces: &[String],
    identity_bytes: [u8; 32],
    sync_states: &Arc<Mutex<HashMap<String, automerge::sync::State>>>,
    event_tx: &mpsc::Sender<NetEvent>,
) {
    let peer_id = hex::encode(conn.remote_id().as_bytes());

    for space_id in my_spaces {
        let (board_ids, space_doc_bytes) = {
            let guard = storage.lock().unwrap();
            let boards = monotask_storage::space::get_space_boards(guard.conn(), space_id)
                .unwrap_or_default();
            let doc_bytes = monotask_storage::space::load_space_doc(guard.conn(), space_id)
                .unwrap_or_default();
            (boards, doc_bytes)
        };
        let identity = monotask_crypto::Identity::from_secret_bytes(&identity_bytes);
        let signature = identity.sign(space_id.as_bytes());

        let (mut send, mut recv) = match conn.open_bi().await {
            Ok(s) => s,
            Err(e) => { tracing::warn!("net: open_bi (Hello) failed: {e}"); return }
        };
        if let Err(e) = write_cbor(&mut send, &SyncRequest::Hello {
            space_id: space_id.clone(),
            board_ids,
            signature,
            space_doc_bytes,
        }).await {
            tracing::warn!("net: write Hello failed: {e}");
            return;
        }

        let response: SyncResponse = match read_cbor(&mut recv).await {
            Ok(r) => r,
            Err(e) => { tracing::warn!("net: read HelloAck failed: {e}"); return }
        };
        drop(send); drop(recv);

        if let SyncResponse::HelloAck { space_id: ack_space, board_ids: their_boards, space_doc_bytes: their_doc } = response {
            if !their_doc.is_empty() {
                let mut guard = storage.lock().unwrap();
                merge_space_doc(&ack_space, &their_doc, &mut guard);
            }
            let our_boards = storage.lock().unwrap().list_board_ids().unwrap_or_default();
            let all_boards: std::collections::HashSet<String> =
                our_boards.into_iter().chain(their_boards).collect();
            for board_id in all_boards {
                sync_board_with_peer(conn, &peer_id, &board_id, storage, sync_states, event_tx).await;
            }
        } else if let SyncResponse::Rejected { reason } = response {
            tracing::warn!("net: Hello rejected by {peer_id:.8}: {reason}");
        }
    }
}

// ---------------------------------------------------------------------------
// Sync one board with a peer (multi-round Automerge protocol, initiator role)
// ---------------------------------------------------------------------------
async fn sync_board_with_peer(
    conn: &iroh::endpoint::Connection,
    peer_id: &str,
    board_id: &str,
    storage: &Arc<Mutex<Storage>>,
    sync_states: &Arc<Mutex<HashMap<String, automerge::sync::State>>>,
    event_tx: &mpsc::Sender<NetEvent>,
) {
    use automerge::sync::SyncDoc;

    // Clear stale initiator state so the peer gets the latest changes.
    sync_states.lock().unwrap().remove(&format!("i/{peer_id}/{board_id}"));

    loop {
        let msg_bytes: Option<Vec<u8>> = {
            let guard = storage.lock().unwrap();
            let mut doc = match guard.load_board(board_id) {
                Ok(d) => d,
                Err(_) => automerge::AutoCommit::new(),
            };
            let mut states = sync_states.lock().unwrap();
            let state = states
                .entry(format!("i/{peer_id}/{board_id}"))
                .or_insert_with(automerge::sync::State::new);
            let encoded: Option<Vec<u8>> = doc.sync().generate_sync_message(state)
                .map(|m| m.encode());
            encoded
        };

        let Some(bytes) = msg_bytes else {
            let _ = event_tx.send(NetEvent::BoardSynced {
                board_id: board_id.to_string(), peer_id: peer_id.to_string(),
            }).await;
            break;
        };

        let (mut send, mut recv) = match conn.open_bi().await {
            Ok(s) => s,
            Err(e) => { tracing::warn!("net: open_bi (BoardSync) failed: {e}"); break }
        };
        if let Err(e) = write_cbor(&mut send, &SyncRequest::BoardSync {
            board_id: board_id.to_string(), sync_message: bytes,
        }).await {
            tracing::warn!("net: write BoardSync failed: {e}"); break;
        }

        let response: SyncResponse = match read_cbor(&mut recv).await {
            Ok(r) => r,
            Err(e) => { tracing::warn!("net: read BoardSync response failed: {e}"); break }
        };
        drop(send); drop(recv);

        match response {
            SyncResponse::BoardSync { sync_message: None, .. } => {
                let _ = event_tx.send(NetEvent::BoardSynced {
                    board_id: board_id.to_string(), peer_id: peer_id.to_string(),
                }).await;
                break;
            }
            SyncResponse::BoardSync { sync_message: Some(reply_bytes), .. } => {
                let sync_result = {
                    let mut states = sync_states.lock().unwrap();
                    let state = states
                        .entry(format!("i/{peer_id}/{board_id}"))
                        .or_insert_with(automerge::sync::State::new);
                    process_incoming_sync(board_id, &reply_bytes, storage, state)
                }; // MutexGuard dropped before any await
                if let Err(e) = sync_result {
                    let _ = event_tx.send(NetEvent::SyncError {
                        board_id: board_id.to_string(), error: e.to_string(),
                    }).await;
                    break;
                }
                // Continue loop — generate next sync message.
            }
            SyncResponse::Rejected { reason } => {
                tracing::warn!("net: BoardSync rejected: {reason}");
                break;
            }
            _ => break,
        }
    }
}

// ---------------------------------------------------------------------------
// Responder: handle one incoming SyncRequest and produce a SyncResponse
// ---------------------------------------------------------------------------
fn handle_one_request(
    request: SyncRequest,
    peer_id: &str,
    peer_pubkey: [u8; 32],
    storage: &Arc<Mutex<Storage>>,
    sync_states: &Arc<Mutex<HashMap<String, automerge::sync::State>>>,
    _my_spaces: &[String],
) -> SyncResponse {
    use monotask_crypto::Identity;

    match request {
        SyncRequest::Hello { space_id, board_ids: _, signature, space_doc_bytes } => {
            // Verify signature — peer signed space_id with their ed25519 key (= their NodeId).
            if Identity::verify(&peer_pubkey, space_id.as_bytes(), &signature).is_err() {
                return SyncResponse::Rejected { reason: "bad signature".into() };
            }
            let pubkey_hex = hex::encode(peer_pubkey);

            let (my_board_ids, my_space_doc_bytes) = {
                let mut guard = storage.lock().unwrap();
                if !space_doc_bytes.is_empty() {
                    merge_space_doc(&space_id, &space_doc_bytes, &mut guard);
                }
                let is_member = monotask_storage::space::is_active_member(
                    guard.conn(), &space_id, &pubkey_hex,
                ).unwrap_or(false);
                if !is_member {
                    return SyncResponse::Rejected { reason: "not a member".into() };
                }
                let boards = monotask_storage::space::get_space_boards(guard.conn(), &space_id)
                    .unwrap_or_default();
                let doc_bytes = monotask_storage::space::load_space_doc(guard.conn(), &space_id)
                    .unwrap_or_default();
                (boards, doc_bytes)
            };
            SyncResponse::HelloAck {
                space_id,
                board_ids: my_board_ids,
                space_doc_bytes: my_space_doc_bytes,
            }
        }

        SyncRequest::BoardSync { board_id, sync_message } => {
            let mut states = sync_states.lock().unwrap();
            let state = states
                .entry(format!("r/{peer_id}/{board_id}"))
                .or_insert_with(automerge::sync::State::new);
            match process_incoming_sync(&board_id, &sync_message, storage, state) {
                Ok(reply) => SyncResponse::BoardSync { board_id, sync_message: reply },
                Err(e) => SyncResponse::Rejected { reason: e.to_string() },
            }
        }

        SyncRequest::Presence { pubkey, status, display_name } => {
            // Update presence in space_members for this peer across all spaces they belong to.
            if let Ok(guard) = storage.lock() {
                let _ = guard.conn().execute(
                    "UPDATE space_members SET presence = ?1 WHERE pubkey = ?2",
                    rusqlite::params![status, pubkey],
                );
                if !display_name.is_empty() {
                    let _ = guard.conn().execute(
                        "UPDATE space_members SET display_name = ?1 WHERE pubkey = ?2 AND (display_name IS NULL OR display_name = '')",
                        rusqlite::params![display_name, pubkey],
                    );
                }
            }
            // No response needed for presence — return a no-op BoardSync
            SyncResponse::BoardSync { board_id: String::new(), sync_message: None }
        }
    }
}

// ---------------------------------------------------------------------------
// Automerge sync helper (unchanged from libp2p version)
// ---------------------------------------------------------------------------
fn process_incoming_sync(
    board_id: &str,
    sync_message: &[u8],
    storage: &Arc<Mutex<Storage>>,
    sync_state: &mut automerge::sync::State,
) -> Result<Option<Vec<u8>>, crate::NetError> {
    use automerge::{AutoCommit, sync as am_sync};
    use am_sync::SyncDoc;

    let msg = am_sync::Message::decode(sync_message)
        .map_err(|e| crate::NetError::Sync(e.to_string()))?;

    let mut guard = storage.lock().unwrap();
    let mut doc = match guard.load_board(board_id) {
        Ok(d) => d,
        Err(_) => AutoCommit::new(),
    };

    doc.sync()
        .receive_sync_message(sync_state, msg)
        .map_err(|e| crate::NetError::Sync(e.to_string()))?;

    guard.save_board(board_id, &mut doc)
        .map_err(crate::NetError::Storage)?;
    drop(guard);

    let encoded: Option<Vec<u8>> = doc.sync().generate_sync_message(sync_state)
        .map(|m| m.encode());
    Ok(encoded)
}

// ---------------------------------------------------------------------------
// Space doc merge (unchanged from libp2p version)
// ---------------------------------------------------------------------------
pub(crate) fn merge_space_doc(space_id: &str, peer_doc_bytes: &[u8], guard: &mut Storage) {
    use automerge::AutoCommit;
    use monotask_storage::space as ss;
    use monotask_core::space as cs;

    let our_bytes = match ss::load_space_doc(guard.conn(), space_id) {
        Ok(b) => b,
        Err(_) => return,
    };
    let mut our_doc = if our_bytes.is_empty() {
        AutoCommit::new()
    } else {
        match AutoCommit::load(&our_bytes) {
            Ok(d) => d,
            Err(e) => { eprintln!("SPACE_SYNC: failed to load our doc: {e}"); return }
        }
    };
    let mut their_doc = match AutoCommit::load(peer_doc_bytes) {
        Ok(d) => d,
        Err(e) => { eprintln!("SPACE_SYNC: failed to load peer doc: {e}"); return }
    };
    if let Err(e) = our_doc.merge(&mut their_doc) {
        eprintln!("SPACE_SYNC: merge error: {e}"); return;
    }
    let merged_bytes = our_doc.save();
    let _ = ss::update_space_doc(guard.conn(), space_id, &merged_bytes);
    if let Some(name) = cs::get_space_name(&our_doc) {
        let _ = ss::rename_space(guard.conn(), space_id, &name);
    }
    if let Ok(members) = cs::list_members(&our_doc) {
        for m in members {
            let _ = ss::upsert_member(guard.conn(), space_id, &monotask_core::space::Member {
                pubkey: m.pubkey, display_name: m.display_name,
                avatar_blob: m.avatar_blob, bio: m.bio, role: m.role,
                color_accent: m.color_accent, presence: m.presence, kicked: m.kicked,
            });
        }
    }
    if let Ok(board_refs) = cs::list_board_refs(&our_doc) {
        for board_id in board_refs {
            let _ = ss::add_board(guard.conn(), space_id, &board_id);
        }
    }
}

// ---------------------------------------------------------------------------
// Peer address parsing helpers
// ---------------------------------------------------------------------------

/// Parse a hex EndpointId string (32 bytes, 64 hex chars).
fn parse_node_id(hex_str: &str) -> Result<EndpointId, ()> {
    let bytes = hex::decode(hex_str).map_err(|_| ())?;
    let arr: [u8; 32] = bytes.try_into().map_err(|_| ())?;
    EndpointId::from_bytes(&arr).map_err(|_| ())
}

/// Parse a peer address string into an iroh `EndpointAddr`.
///
/// Formats accepted:
/// - `<64-hex-chars>` — EndpointId only, relay will be used
/// - `<64-hex-chars>@<ip>:<port>` — EndpointId + direct UDP hint
pub fn parse_peer_addr(s: &str) -> Option<EndpointAddr> {
    if let Some((id_part, addr_part)) = s.split_once('@') {
        let node_id = parse_node_id(id_part).ok()?;
        let sock: std::net::SocketAddr = addr_part.parse().ok()?;
        Some(EndpointAddr::new(node_id).with_ip_addr(sock))
    } else {
        let node_id = parse_node_id(s).ok()?;
        Some(EndpointAddr::new(node_id))
    }
}

// ---------------------------------------------------------------------------
// Saved-peers persistence (replaces saved_peers.txt / multiaddr format)
// ---------------------------------------------------------------------------
fn saved_peers_path(data_dir: &std::path::Path) -> std::path::PathBuf {
    data_dir.join("saved_peers_iroh.txt")
}

fn load_saved_peers(data_dir: &std::path::Path) -> Vec<String> {
    let path = saved_peers_path(data_dir);
    std::fs::read_to_string(&path)
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_string())
        .collect()
}

fn save_peer_addr(data_dir: &std::path::Path, addr: &str) {
    let path = saved_peers_path(data_dir);
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new().append(true).create(true).open(&path) {
        let _ = writeln!(f, "{}", addr);
    }
}
