use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

/// The port used for LAN peer-discovery broadcasts (separate from the sync port).
const LAN_DISCOVERY_PORT: u16 = 7273;
/// How often we re-announce ourselves on LAN.
const ANNOUNCE_INTERVAL: Duration = Duration::from_secs(5);

/// A peer discovered on the local network.
#[derive(Debug, Clone)]
pub struct DiscoveredPeer {
    /// Hex-encoded 32-byte iroh NodeId (== ed25519 pubkey) of the discovered peer.
    pub node_id_hex: String,
    /// Direct UDP address where the peer's iroh endpoint is reachable.
    pub addr: SocketAddr,
}

/// Run a lightweight UDP-broadcast LAN discovery loop.
///
/// Periodically broadcasts our (node_id, listen_port) on the LAN subnet so other
/// monotask instances find us without a relay server.  Each broadcast triggers a
/// `DiscoveredPeer` message on `peer_tx` when a *different* node is heard.
///
/// This task runs forever; cancel by dropping the returned join handle.
pub async fn run_lan_discovery(
    our_node_id_hex: String,
    our_listen_port: u16,
    peer_tx: mpsc::Sender<DiscoveredPeer>,
) {
    if let Err(e) = run_inner(&our_node_id_hex, our_listen_port, &peer_tx).await {
        tracing::warn!("LAN discovery error: {e}");
    }
}

async fn run_inner(
    our_node_id_hex: &str,
    our_listen_port: u16,
    peer_tx: &mpsc::Sender<DiscoveredPeer>,
) -> std::io::Result<()> {
    let socket = UdpSocket::bind(("0.0.0.0", LAN_DISCOVERY_PORT)).await?;
    socket.set_broadcast(true)?;

    let broadcast_addr: SocketAddr = format!("255.255.255.255:{LAN_DISCOVERY_PORT}").parse().unwrap();

    // Fixed-size announcement packet: 32 bytes node_id + 2 bytes port (big-endian).
    let node_id_bytes = hex::decode(our_node_id_hex).unwrap_or_default();
    if node_id_bytes.len() != 32 {
        tracing::warn!("LAN discovery: invalid node_id, skipping");
        return Ok(());
    }
    let mut announce_pkt = [0u8; 34];
    announce_pkt[..32].copy_from_slice(&node_id_bytes);
    announce_pkt[32..].copy_from_slice(&our_listen_port.to_be_bytes());

    let mut announce_tick = tokio::time::interval(ANNOUNCE_INTERVAL);
    let mut recv_buf = [0u8; 64];

    loop {
        tokio::select! {
            _ = announce_tick.tick() => {
                if let Err(e) = socket.send_to(&announce_pkt, broadcast_addr).await {
                    tracing::debug!("LAN discovery: broadcast failed: {e}");
                }
            }

            result = socket.recv_from(&mut recv_buf) => {
                let Ok((n, src)) = result else { continue };
                if n < 34 { continue }

                let peer_node_id_bytes = &recv_buf[..32];
                let peer_port = u16::from_be_bytes([recv_buf[32], recv_buf[33]]);
                let peer_node_id_hex = hex::encode(peer_node_id_bytes);

                // Ignore our own broadcasts.
                if peer_node_id_hex == our_node_id_hex { continue }

                let peer_addr = SocketAddr::new(src.ip(), peer_port);
                let _ = peer_tx.send(DiscoveredPeer {
                    node_id_hex: peer_node_id_hex,
                    addr: peer_addr,
                }).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn announce_packet_layout_is_34_bytes() {
        let node_id = [0u8; 32];
        let port: u16 = 7272;
        let mut pkt = [0u8; 34];
        pkt[..32].copy_from_slice(&node_id);
        pkt[32..].copy_from_slice(&port.to_be_bytes());
        assert_eq!(u16::from_be_bytes([pkt[32], pkt[33]]), port);
    }
}
