use anyhow::Result;
use crate::MailMessage;

pub struct OutlookClient {
    token: String,
    http: reqwest::Client,
}

impl OutlookClient {
    pub fn new(access_token: &str) -> Self {
        Self {
            token: access_token.to_string(),
            http: reqwest::Client::new(),
        }
    }

    pub async fn fetch_since(&self, since_ts: i64) -> Result<Vec<MailMessage>> {
        let since = chrono::DateTime::from_timestamp(since_ts, 0)
            .unwrap_or_else(|| chrono::DateTime::UNIX_EPOCH.into())
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();

        let resp: serde_json::Value = self.http
            .get("https://graph.microsoft.com/v1.0/me/messages")
            .bearer_auth(&self.token)
            .header("Accept", "application/json")
            .query(&[
                ("$select", "id,internetMessageId,from,subject,receivedDateTime,bodyPreview"),
                ("$top", "200"),
                ("$filter", &format!("receivedDateTime ge {since}")),
                ("$orderby", "receivedDateTime desc"),
            ])
            .send().await?
            .json().await?;

        if let Some(err) = resp.get("error") {
            anyhow::bail!("Microsoft Graph API error: {}", err);
        }

        let items = resp["value"].as_array()
            .cloned()
            .unwrap_or_default();

        Ok(items.into_iter().filter_map(|item| self.parse_message(item)).collect())
    }

    fn parse_message(&self, item: serde_json::Value) -> Option<MailMessage> {
        let from_email = item["from"]["emailAddress"]["address"]
            .as_str()?.to_lowercase();
        let from_name = item["from"]["emailAddress"]["name"]
            .as_str()
            .unwrap_or(&from_email)
            .to_string();
        let from_name = if from_name.is_empty() { from_email.clone() } else { from_name };
        let subject = item["subject"].as_str().unwrap_or("(no subject)").to_string();
        let date = item["receivedDateTime"].as_str().unwrap_or("").to_string();
        let snippet = item["bodyPreview"].as_str().unwrap_or("").to_string();
        let id = item["internetMessageId"].as_str()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| item["id"].as_str().unwrap_or(""))
            .to_string();

        if from_email.is_empty() { return None; }

        Some(MailMessage {
            id,
            from_name,
            from_email,
            subject,
            date,
            snippet,
            provider: "outlook".into(),
        })
    }
}
