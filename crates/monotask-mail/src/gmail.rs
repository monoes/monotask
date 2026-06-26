use anyhow::{Context, Result};
use crate::MailMessage;

pub struct GmailClient {
    token: String,
    http: reqwest::Client,
}

impl GmailClient {
    pub fn new(access_token: &str) -> Self {
        Self {
            token: access_token.to_string(),
            http: reqwest::Client::new(),
        }
    }

    pub async fn fetch_since(&self, since_ts: i64) -> Result<Vec<MailMessage>> {
        let q = format!("in:inbox after:{since_ts}");
        let ids = self.list_message_ids(&q, 200).await?;
        let mut messages = Vec::new();
        for id in &ids {
            match self.get_metadata(id).await {
                Ok(m) => messages.push(m),
                Err(_) => continue,
            }
        }
        Ok(messages)
    }

    async fn list_message_ids(&self, q: &str, max: u32) -> Result<Vec<String>> {
        let resp: serde_json::Value = self.http
            .get("https://gmail.googleapis.com/gmail/v1/users/me/messages")
            .bearer_auth(&self.token)
            .query(&[("q", q), ("maxResults", &max.to_string()), ("fields", "messages(id)")])
            .send().await?
            .json().await?;

        Ok(resp["messages"].as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
            .collect())
    }

    async fn get_metadata(&self, msg_id: &str) -> Result<MailMessage> {
        let resp: serde_json::Value = self.http
            .get(format!("https://gmail.googleapis.com/gmail/v1/users/me/messages/{msg_id}"))
            .bearer_auth(&self.token)
            .query(&[
                ("format", "metadata"),
                ("metadataHeaders", "From,Subject,Date,Message-ID"),
                ("fields", "id,snippet,payload(headers)"),
            ])
            .send().await?
            .json().await?;

        let headers = resp["payload"]["headers"].as_array()
            .context("missing headers")?;

        let get = |name: &str| -> String {
            headers.iter()
                .find(|h| h["name"].as_str().map(|n| n.eq_ignore_ascii_case(name)).unwrap_or(false))
                .and_then(|h| h["value"].as_str())
                .unwrap_or("")
                .to_string()
        };

        let from_raw = get("From");
        let (from_name, from_email) = parse_from(&from_raw);
        let subject = get("Subject");
        let subject = if subject.is_empty() { "(no subject)".into() } else { subject };
        let date = get("Date");
        let message_id = get("Message-ID");
        let snippet = resp["snippet"].as_str().unwrap_or("").to_string();

        Ok(MailMessage {
            id: if message_id.is_empty() { msg_id.to_string() } else { message_id },
            from_name,
            from_email,
            subject,
            date,
            snippet,
            provider: "gmail".into(),
        })
    }
}

fn parse_from(raw: &str) -> (String, String) {
    // Formats: "Name <email>" or just "email"
    if let Some(angle) = raw.rfind('<') {
        let name = raw[..angle].trim().trim_matches('"').to_string();
        let email = raw[angle+1..].trim_end_matches('>').trim().to_lowercase();
        (if name.is_empty() { email.clone() } else { name }, email)
    } else {
        let email = raw.trim().to_lowercase();
        (email.clone(), email)
    }
}
