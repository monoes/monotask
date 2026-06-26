use anyhow::{Context, Result};
use crate::MailMessage;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImapCredentials {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub folder: String,
}

impl Default for ImapCredentials {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 993,
            username: String::new(),
            password: String::new(),
            folder: "INBOX".into(),
        }
    }
}

/// Fetch messages received since `since_ts` (Unix timestamp) via IMAP.
/// Runs the synchronous imap client in a blocking thread.
pub async fn fetch_since(creds: ImapCredentials, since_ts: i64) -> Result<Vec<MailMessage>> {
    tokio::task::spawn_blocking(move || fetch_blocking(&creds, since_ts))
        .await
        .context("IMAP thread panicked")?
}

/// Test connection without fetching any messages. Returns Ok(()) on success.
pub fn fetch_since_sync_test(creds: &ImapCredentials) -> Result<()> {
    let tls = native_tls::TlsConnector::builder()
        .build()
        .context("Failed to build TLS connector")?;
    let client = imap::connect(
        (creds.host.as_str(), creds.port),
        &creds.host,
        &tls,
    ).context("IMAP connection failed")?;
    let mut session = client
        .login(&creds.username, &creds.password)
        .map_err(|(e, _)| anyhow::anyhow!("IMAP login failed: {e}"))?;
    let _ = session.logout();
    Ok(())
}

fn fetch_blocking(creds: &ImapCredentials, since_ts: i64) -> Result<Vec<MailMessage>> {
    let since_date = ts_to_imap_date(since_ts);

    let tls = native_tls::TlsConnector::builder()
        .build()
        .context("Failed to build TLS connector")?;

    let client = imap::connect(
        (creds.host.as_str(), creds.port),
        &creds.host,
        &tls,
    ).context("IMAP connection failed")?;

    let mut session = client
        .login(&creds.username, &creds.password)
        .map_err(|(e, _)| anyhow::anyhow!("IMAP login failed: {e}"))?;

    session.select(&creds.folder)
        .context("Failed to select IMAP folder")?;

    let search_query = format!("SINCE {since_date}");
    let seq_set = session.search(&search_query)
        .context("IMAP SEARCH failed")?;

    if seq_set.is_empty() {
        let _ = session.logout();
        return Ok(vec![]);
    }

    // Fetch only the last 200 messages to bound the sync
    let mut sorted_ids: Vec<u32> = seq_set.into_iter().collect();
    sorted_ids.sort_unstable();
    let ids: Vec<String> = sorted_ids.iter()
        .rev()
        .take(200)
        .map(|n| n.to_string())
        .collect();
    let id_set = ids.join(",");

    let messages = session
        .fetch(&id_set, "(ENVELOPE UID)")
        .context("IMAP FETCH failed")?;

    let mut result = Vec::new();
    for msg in messages.iter() {
        if let Some(env) = msg.envelope() {
            if let Some(m) = parse_envelope(env) {
                result.push(m);
            }
        }
    }

    let _ = session.logout();
    Ok(result)
}

fn parse_envelope(env: &imap_proto::types::Envelope<'_>) -> Option<MailMessage> {
    let from_list = env.from.as_ref()?;
    let addr = from_list.first()?;

    let mailbox = addr.mailbox.as_ref()
        .map(|b| String::from_utf8_lossy(b).to_lowercase())
        .unwrap_or_default();
    let host = addr.host.as_ref()
        .map(|b| String::from_utf8_lossy(b).to_lowercase())
        .unwrap_or_default();
    let from_email = if mailbox.is_empty() || host.is_empty() {
        return None;
    } else {
        format!("{mailbox}@{host}")
    };

    let from_name = addr.name.as_ref()
        .map(|b| decode_mime(b))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| from_email.clone());

    let subject = env.subject.as_ref()
        .map(|b| decode_mime(b))
        .unwrap_or_else(|| "(no subject)".into());

    let date = env.date.as_ref()
        .map(|b| String::from_utf8_lossy(b).to_string())
        .unwrap_or_default();

    let message_id = env.message_id.as_ref()
        .map(|b| String::from_utf8_lossy(b).trim().to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    Some(MailMessage {
        id: message_id,
        from_name,
        from_email,
        subject,
        date,
        snippet: String::new(),
        provider: "imap".into(),
    })
}

fn decode_mime(bytes: &[u8]) -> String {
    // Basic: try UTF-8, strip encoded-word markers, fall back to lossy
    let raw = String::from_utf8_lossy(bytes).to_string();
    // Handle simple =?UTF-8?B?...?= and =?UTF-8?Q?...?= patterns
    if raw.contains("=?") {
        return decode_encoded_words(&raw);
    }
    raw
}

fn decode_encoded_words(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(start) = rest.find("=?") {
        out.push_str(&rest[..start]);
        rest = &rest[start + 2..];
        if let Some(end) = rest.find("?=") {
            let encoded = &rest[..end];
            rest = &rest[end + 2..];
            // charset?encoding?text
            let parts: Vec<&str> = encoded.splitn(3, '?').collect();
            if parts.len() == 3 {
                let encoding = parts[1].to_uppercase();
                let text = parts[2];
                let decoded = if encoding == "B" {
                    use base64::Engine as _;
                    base64::engine::general_purpose::STANDARD
                        .decode(text.replace(' ', "+"))
                        .ok()
                        .and_then(|b| String::from_utf8(b).ok())
                        .unwrap_or_else(|| text.to_string())
                } else if encoding == "Q" {
                    text.replace('_', " ").replace("=20", " ")
                } else {
                    text.to_string()
                };
                out.push_str(&decoded);
            } else {
                out.push_str(encoded);
            }
        } else {
            out.push_str(rest);
            rest = "";
        }
    }
    out.push_str(rest);
    out
}

fn ts_to_imap_date(ts: i64) -> String {
    let dt = chrono::DateTime::from_timestamp(ts, 0)
        .unwrap_or_else(|| chrono::DateTime::UNIX_EPOCH.into());
    dt.format("%-d-%b-%Y").to_string()  // e.g. "1-Jan-2024"
}

pub fn imap_presets() -> Vec<(&'static str, &'static str, u16)> {
    vec![
        ("Gmail",             "imap.gmail.com",          993),
        ("Outlook / Office",  "outlook.office365.com",   993),
        ("Yahoo Mail",        "imap.mail.yahoo.com",     993),
        ("Fastmail",          "imap.fastmail.com",       993),
        ("iCloud Mail",       "imap.mail.me.com",        993),
        ("ProtonMail Bridge", "127.0.0.1",               1143),
    ]
}
