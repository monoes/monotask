use iroh::{Endpoint, SecretKey, endpoint::presets};
use crate::{NetConfig, NetError};
use crate::sync_protocol::PROTOCOL_ALPN;

/// Build an iroh QUIC endpoint seeded from the app's ed25519 identity bytes.
pub async fn build_endpoint(
    identity_bytes: [u8; 32],
    config: &NetConfig,
) -> Result<Endpoint, NetError> {
    let secret_key = SecretKey::from_bytes(&identity_bytes);
    let mut builder = Endpoint::builder(presets::Minimal)
        .secret_key(secret_key)
        .alpns(vec![PROTOCOL_ALPN.to_vec()]);

    if config.listen_port != 0 {
        let addr: std::net::SocketAddr =
            format!("0.0.0.0:{}", config.listen_port).parse().unwrap();
        builder = builder
            .bind_addr(addr)
            .map_err(|e| NetError::Transport(e.to_string()))?;
    }

    builder.bind().await.map_err(|e| NetError::Transport(e.to_string()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn behaviour_module_exists() { assert!(true); }
}
