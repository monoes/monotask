mod gmail;
mod outlook;
pub mod imap_client;
mod sync;

pub use sync::{sync_board, SyncResult};
pub use imap_client::ImapCredentials;

use anyhow::{Context, Result};
use automerge::{AutoCommit, ReadDoc, transaction::Transactable};
use std::path::{Path, PathBuf};

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MailToken {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MailConfig {
    pub provider: String,
    pub gmail_client_id: Option<String>,
    pub outlook_client_id: Option<String>,
    pub outlook_tenant_id: String,
    pub inbox_col_id: Option<String>,
    pub keep_last: u64,
    pub last_sync: Option<String>,
}

impl Default for MailConfig {
    fn default() -> Self {
        Self {
            provider: "both".into(),
            gmail_client_id: None,
            outlook_client_id: None,
            outlook_tenant_id: "common".into(),
            inbox_col_id: None,
            keep_last: 2,
            last_sync: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MailMessage {
    pub id: String,
    pub from_name: String,
    pub from_email: String,
    pub subject: String,
    pub date: String,
    pub snippet: String,
    pub provider: String,
}

// ── Token file management ─────────────────────────────────────────────────────

pub fn token_path(data_dir: &Path, provider: &str) -> PathBuf {
    data_dir.join(format!("{}_token.json", provider))
}

pub fn save_token(data_dir: &Path, provider: &str, token: &MailToken) -> Result<()> {
    let path = token_path(data_dir, provider);
    let json = serde_json::to_string(token)?;
    std::fs::write(&path, json)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

pub fn load_token(data_dir: &Path, provider: &str) -> Result<Option<MailToken>> {
    let path = token_path(data_dir, provider);
    if !path.exists() {
        return Ok(None);
    }
    let json = std::fs::read_to_string(&path)?;
    Ok(Some(serde_json::from_str(&json)?))
}

pub fn delete_token(data_dir: &Path, provider: &str) -> Result<()> {
    let path = token_path(data_dir, provider);
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

pub fn token_saved(data_dir: &Path, provider: &str) -> bool {
    token_path(data_dir, provider).exists()
}

// ── IMAP credential management ────────────────────────────────────────────────

pub fn imap_creds_path(data_dir: &Path) -> PathBuf {
    data_dir.join("imap_credentials.json")
}

pub fn save_imap_credentials(data_dir: &Path, creds: &ImapCredentials) -> Result<()> {
    let path = imap_creds_path(data_dir);
    let json = serde_json::to_string(creds)?;
    std::fs::write(&path, json)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

pub fn load_imap_credentials(data_dir: &Path) -> Result<Option<ImapCredentials>> {
    let path = imap_creds_path(data_dir);
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_str(&std::fs::read_to_string(&path)?)?))
}

pub fn delete_imap_credentials(data_dir: &Path) -> Result<()> {
    let path = imap_creds_path(data_dir);
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

pub fn imap_credentials_saved(data_dir: &Path) -> bool {
    imap_creds_path(data_dir).exists()
}

pub async fn get_valid_token(data_dir: &Path, provider: &str, client_id: &str, tenant_id: &str) -> Result<MailToken> {
    let token = load_token(data_dir, provider)?
        .context("No token saved. Connect first.")?;
    if chrono::Utc::now().timestamp() + 60 < token.expires_at {
        return Ok(token);
    }
    let refreshed = refresh_token(provider, client_id, tenant_id, &token.refresh_token).await?;
    save_token(data_dir, provider, &refreshed)?;
    Ok(refreshed)
}

async fn refresh_token(provider: &str, client_id: &str, tenant_id: &str, refresh_token: &str) -> Result<MailToken> {
    let token_url = token_url(provider, tenant_id);
    let params = [
        ("client_id", client_id),
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
    ];
    exchange_request(&token_url, &params).await
}

// ── PKCE helpers ──────────────────────────────────────────────────────────────

pub fn generate_pkce() -> (String, String) {
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
    use sha2::{Sha256, Digest};
    let mut bytes = [0u8; 64];
    for i in 0..4 {
        let id = uuid::Uuid::new_v4();
        bytes[i * 16..(i + 1) * 16].copy_from_slice(id.as_bytes());
    }
    let verifier = URL_SAFE_NO_PAD.encode(&bytes);
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()).as_slice());
    (verifier, challenge)
}

// ── OAuth URL builders ────────────────────────────────────────────────────────

fn token_url(provider: &str, tenant_id: &str) -> String {
    match provider {
        "gmail" => "https://oauth2.googleapis.com/token".into(),
        _ => format!("https://login.microsoftonline.com/{tenant_id}/oauth2/v2.0/token"),
    }
}

pub fn build_auth_url(provider: &str, client_id: &str, tenant_id: &str, challenge: &str, redirect_uri: &str) -> Result<String> {
    let (base, scope) = match provider {
        "gmail" => (
            "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            "https://www.googleapis.com/auth/gmail.readonly",
        ),
        "outlook" => (
            format!("https://login.microsoftonline.com/{tenant_id}/oauth2/v2.0/authorize"),
            "https://graph.microsoft.com/Mail.Read offline_access",
        ),
        p => anyhow::bail!("Unknown provider: {p}"),
    };
    Ok(format!(
        "{base}?client_id={client_id}&response_type=code&redirect_uri={redir}&scope={scope}\
         &code_challenge={challenge}&code_challenge_method=S256&access_type=offline&prompt=consent",
        redir = urlencoded(redirect_uri),
        scope = urlencoded(scope),
    ))
}

fn urlencoded(s: &str) -> String {
    s.chars().map(|c| match c {
        'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
        _ => format!("%{:02X}", c as u32),
    }).collect()
}

// ── Loopback OAuth server ─────────────────────────────────────────────────────

pub async fn wait_and_complete_oauth(
    listener: tokio::net::TcpListener,
    data_dir: &Path,
    provider: &str,
    client_id: &str,
    tenant_id: &str,
    verifier: &str,
    redirect_uri: &str,
) -> Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let (mut stream, _) = tokio::time::timeout(
        std::time::Duration::from_secs(300),
        listener.accept(),
    ).await.context("OAuth timeout after 5 minutes")??;

    let mut buf = [0u8; 8192];
    let n = stream.read(&mut buf).await?;
    let req = std::str::from_utf8(&buf[..n]).unwrap_or("");

    let first_line = req.lines().next().unwrap_or("");
    let qs = first_line.split('?').nth(1).unwrap_or("").split_whitespace().next().unwrap_or("");

    if let Some(err) = qs.split('&').find(|p| p.starts_with("error=")) {
        let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Type: text/html\r\n\r\n<h1>Authorization denied. You can close this tab.</h1>").await;
        anyhow::bail!("OAuth error: {}", err.trim_start_matches("error="));
    }

    let code = qs.split('&')
        .find(|p| p.starts_with("code="))
        .and_then(|p| p.strip_prefix("code="))
        .context("No code in OAuth callback")?
        .to_string();

    let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<h1>Connected! You can close this tab.</h1>").await;
    drop(stream);

    let token_url = token_url(provider, tenant_id);
    let token = exchange_request(&token_url, &[
        ("client_id", client_id),
        ("code", &code),
        ("redirect_uri", redirect_uri),
        ("grant_type", "authorization_code"),
        ("code_verifier", verifier),
    ]).await?;

    save_token(data_dir, provider, &token)
}

async fn exchange_request(token_url: &str, params: &[(&str, &str)]) -> Result<MailToken> {
    let client = reqwest::Client::new();
    let resp: serde_json::Value = client.post(token_url)
        .form(params)
        .send().await?
        .json().await?;

    let access_token = resp["access_token"].as_str()
        .with_context(|| format!("No access_token in response: {resp}"))?
        .to_string();
    let refresh_token = resp["refresh_token"].as_str()
        .context("No refresh_token — did you include offline_access scope?")?
        .to_string();
    let expires_in = resp["expires_in"].as_i64().unwrap_or(3600);

    Ok(MailToken {
        access_token,
        refresh_token,
        expires_at: chrono::Utc::now().timestamp() + expires_in,
    })
}

// ── CRDT board config ─────────────────────────────────────────────────────────

pub fn get_mail_config(doc: &AutoCommit) -> Option<MailConfig> {
    let provider = monotask_core::get_string(doc, &automerge::ROOT, "mail_provider").ok()??;
    Some(MailConfig {
        provider,
        gmail_client_id: monotask_core::get_string(doc, &automerge::ROOT, "mail_gmail_client_id").ok().flatten(),
        outlook_client_id: monotask_core::get_string(doc, &automerge::ROOT, "mail_outlook_client_id").ok().flatten(),
        outlook_tenant_id: monotask_core::get_string(doc, &automerge::ROOT, "mail_outlook_tenant_id")
            .ok().flatten().unwrap_or_else(|| "common".into()),
        inbox_col_id: monotask_core::get_string(doc, &automerge::ROOT, "mail_inbox_col_id").ok().flatten(),
        keep_last: match doc.get(automerge::ROOT, "mail_keep_last").ok().flatten() {
            Some((automerge::Value::Scalar(s), _)) => match s.as_ref() {
                automerge::ScalarValue::Uint(n) => *n,
                automerge::ScalarValue::Int(n) => (*n).max(1) as u64,
                _ => 2,
            },
            _ => 2,
        },
        last_sync: monotask_core::get_string(doc, &automerge::ROOT, "mail_last_sync").ok().flatten(),
    })
}

pub fn set_mail_config(doc: &mut AutoCommit, config: Option<&MailConfig>) -> Result<()> {
    let keys = ["mail_provider","mail_gmail_client_id","mail_outlook_client_id",
                 "mail_outlook_tenant_id","mail_inbox_col_id","mail_keep_last","mail_last_sync"];
    match config {
        None => {
            for k in &keys { let _ = doc.delete(automerge::ROOT, *k); }
        }
        Some(c) => {
            doc.put(automerge::ROOT, "mail_provider", c.provider.as_str())?;
            if let Some(ref id) = c.gmail_client_id {
                doc.put(automerge::ROOT, "mail_gmail_client_id", id.as_str())?;
            }
            if let Some(ref id) = c.outlook_client_id {
                doc.put(automerge::ROOT, "mail_outlook_client_id", id.as_str())?;
            }
            doc.put(automerge::ROOT, "mail_outlook_tenant_id", c.outlook_tenant_id.as_str())?;
            if let Some(ref col) = c.inbox_col_id {
                doc.put(automerge::ROOT, "mail_inbox_col_id", col.as_str())?;
            }
            doc.put(automerge::ROOT, "mail_keep_last", c.keep_last)?;
            if let Some(ref ts) = c.last_sync {
                doc.put(automerge::ROOT, "mail_last_sync", ts.as_str())?;
            }
        }
    }
    Ok(())
}

pub fn get_mail_field_id(doc: &AutoCommit, key: &str) -> Option<String> {
    monotask_core::get_string(doc, &automerge::ROOT, key).ok().flatten()
}

pub fn set_mail_field_id(doc: &mut AutoCommit, key: &str, field_id: &str) -> Result<()> {
    doc.put(automerge::ROOT, key, field_id)?;
    Ok(())
}
