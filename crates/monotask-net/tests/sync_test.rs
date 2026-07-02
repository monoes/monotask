use monotask_net::{NetworkHandle, NetConfig, NetEvent};
use monotask_storage::Storage;
use monotask_crypto::Identity;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn make_storage() -> Arc<Mutex<Storage>> {
    Arc::new(Mutex::new(Storage::open_in_memory().unwrap()))
}

fn make_identity() -> [u8; 32] {
    Identity::generate().to_secret_bytes()
}

/// Integration test: two NetworkHandle nodes start, node_b dials node_a directly,
/// and node_a emits a PeerConnected event.
#[tokio::test]
async fn two_nodes_connect_and_emit_peer_connected() {
    let id_a = make_identity();
    let id_b = make_identity();
    let storage_a = make_storage();
    let _storage_b = make_storage();

    // Use fixed ports so node_b can dial node_a directly.
    let port_a: u16 = 17280;

    let mut node_a = NetworkHandle::start(
        NetConfig {
            listen_port: port_a,
            data_dir: std::path::PathBuf::from("/tmp/node_a_iroh"),
            bootstrap_peers: Vec::new(),
        },
        storage_a,
        id_a,
    ).await.expect("node_a start");

    // node_b boots knowing node_a's EndpointId@ip:port
    let peer_a_addr = format!("{}@127.0.0.1:{}", NetworkHandle::peer_id_from_identity(id_a), port_a);
    let _node_b = NetworkHandle::start(
        NetConfig {
            listen_port: 0,
            data_dir: std::path::PathBuf::from("/tmp/node_b_iroh"),
            bootstrap_peers: vec![peer_a_addr],
        },
        make_storage(),
        id_b,
    ).await.expect("node_b start");

    // Wait for node_a to see PeerConnected
    let found = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let evt = node_a.event_rx.as_mut().and_then(|rx| rx.try_recv().ok());
            if let Some(NetEvent::PeerConnected { .. }) = evt {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }).await;

    assert!(found.is_ok(), "nodes did not connect within 10 seconds");

    node_a.stop().await;
}
