use std::io;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// ALPN identifier for the Automerge board-sync protocol over iroh QUIC.
pub const PROTOCOL_ALPN: &[u8] = b"/monotask/board-sync/1.0.0";

/// Increment the major version when making a breaking change to `SyncRequest` or `SyncResponse`.
/// Peers with different major versions will refuse to sync and log a `VersionReject` event.
pub const PROTOCOL_MAJOR: u16 = 1;

const MAX_MSG_SIZE: u32 = 10 * 1024 * 1024; // 10 MB

/// Sent as the very first frame on every new connection (before any `SyncRequest`).
/// If the remote's `major` differs from `PROTOCOL_MAJOR`, the recipient sends
/// a `VersionReject` and closes the stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionHello {
    pub major: u16,
}

/// Sent by the acceptor when `VersionHello.major != PROTOCOL_MAJOR`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionReject {
    pub reason: String,
    pub their_major: u16,
    pub our_major: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncRequest {
    /// Ephemeral peer presence heartbeat. Sent periodically to all connected peers.
    /// Recipients update the `presence` field in `space_members` in storage.
    Presence {
        /// The sender's pubkey hex (redundant but avoids a separate lookup).
        pubkey: String,
        /// "online" | "away" | "busy"
        status: String,
        /// The sender's current display name (may be empty).
        display_name: String,
    },
    /// Prove Space membership and share which boards this peer has.
    Hello {
        space_id: String,
        board_ids: Vec<String>,
        /// Ed25519 signature over `space_id.as_bytes()`.
        signature: Vec<u8>,
        /// Automerge-encoded space doc (members, boards, name). Empty = not available.
        #[serde(default)]
        space_doc_bytes: Vec<u8>,
    },
    /// One round of Automerge sync for a board.
    BoardSync {
        board_id: String,
        /// `automerge::sync::Message::encode()` output.
        sync_message: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncResponse {
    /// Hello accepted; here are the responder's board IDs in this Space.
    HelloAck {
        space_id: String,
        board_ids: Vec<String>,
        /// Automerge-encoded space doc (members, boards, name). Empty = not available.
        #[serde(default)]
        space_doc_bytes: Vec<u8>,
    },
    /// One round of Automerge sync in reply. `None` = this side has converged.
    BoardSync {
        board_id: String,
        sync_message: Option<Vec<u8>>,
    },
    /// Rejected: not in same Space, bad signature, or member is kicked.
    Rejected { reason: String },
}

/// Write a length-prefixed CBOR-encoded value to an async writer.
pub async fn write_cbor<T, W>(writer: &mut W, msg: &T) -> io::Result<()>
where
    T: Serialize,
    W: AsyncWrite + Unpin,
{
    let mut buf = Vec::new();
    ciborium::into_writer(msg, &mut buf)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    let len = buf.len() as u32;
    writer.write_all(&len.to_be_bytes()).await?;
    writer.write_all(&buf).await
}

/// Read a length-prefixed CBOR-encoded value from an async reader.
pub async fn read_cbor<T, R>(reader: &mut R) -> io::Result<T>
where
    T: for<'de> Deserialize<'de>,
    R: AsyncRead + Unpin,
{
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf);
    if len > MAX_MSG_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("message too large: {} bytes (max {})", len, MAX_MSG_SIZE),
        ));
    }
    let mut buf = vec![0u8; len as usize];
    reader.read_exact(&mut buf).await?;
    ciborium::from_reader(buf.as_slice())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cbor_roundtrip_request(req: SyncRequest) -> SyncRequest {
        let mut buf = Vec::new();
        ciborium::into_writer(&req, &mut buf).unwrap();
        ciborium::from_reader(buf.as_slice()).unwrap()
    }

    fn cbor_roundtrip_response(res: SyncResponse) -> SyncResponse {
        let mut buf = Vec::new();
        ciborium::into_writer(&res, &mut buf).unwrap();
        ciborium::from_reader(buf.as_slice()).unwrap()
    }

    #[test]
    fn serialize_hello_roundtrip() {
        let req = SyncRequest::Hello {
            space_id: "space-1".into(),
            board_ids: vec!["b1".into(), "b2".into()],
            signature: vec![1, 2, 3],
            space_doc_bytes: vec![],
        };
        let SyncRequest::Hello { space_id, board_ids, signature, .. } = cbor_roundtrip_request(req)
            else { panic!("wrong variant") };
        assert_eq!(space_id, "space-1");
        assert_eq!(board_ids, vec!["b1", "b2"]);
        assert_eq!(signature, vec![1, 2, 3]);
    }

    #[test]
    fn serialize_board_sync_roundtrip() {
        let req = SyncRequest::BoardSync {
            board_id: "b1".into(),
            sync_message: vec![0xDE, 0xAD],
        };
        let SyncRequest::BoardSync { board_id, sync_message } = cbor_roundtrip_request(req)
            else { panic!("wrong variant") };
        assert_eq!(board_id, "b1");
        assert_eq!(sync_message, vec![0xDE, 0xAD]);
    }

    #[test]
    fn serialize_hello_ack_roundtrip() {
        let res = SyncResponse::HelloAck {
            space_id: "s1".into(),
            board_ids: vec!["x".into()],
            space_doc_bytes: vec![],
        };
        let SyncResponse::HelloAck { space_id, board_ids, .. } = cbor_roundtrip_response(res)
            else { panic!() };
        assert_eq!(space_id, "s1");
        assert_eq!(board_ids, vec!["x"]);
    }

    #[test]
    fn serialize_rejected_roundtrip() {
        let res = SyncResponse::Rejected { reason: "kicked".into() };
        let SyncResponse::Rejected { reason } = cbor_roundtrip_response(res)
            else { panic!() };
        assert_eq!(reason, "kicked");
    }
}
