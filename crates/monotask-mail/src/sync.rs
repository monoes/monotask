use anyhow::Result;
use automerge::{AutoCommit, ReadDoc, transaction::Transactable};
use std::collections::HashMap;
use std::path::Path;
use crate::{MailConfig, MailMessage};
use crate::gmail::GmailClient;
use crate::outlook::OutlookClient;
use crate::imap_client;

#[derive(Debug, serde::Serialize)]
pub struct SyncResult {
    pub contacts_created: usize,
    pub contacts_updated: usize,
    pub emails_added: usize,
}

pub async fn sync_board(
    doc: &mut AutoCommit,
    data_dir: &Path,
    config: &MailConfig,
    actor_pk: &[u8],
) -> Result<SyncResult> {
    let since_ts = config.last_sync.as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.timestamp())
        .unwrap_or_else(|| chrono::Utc::now().timestamp() - 30 * 86400);

    let mut all_messages: Vec<MailMessage> = Vec::new();

    if config.provider == "gmail" || config.provider == "both" {
        if let Some(ref cid) = config.gmail_client_id {
            let tok = crate::get_valid_token(data_dir, "gmail", cid, "common").await?;
            let msgs = GmailClient::new(&tok.access_token).fetch_since(since_ts).await?;
            all_messages.extend(msgs);
        }
    }
    if config.provider == "outlook" || config.provider == "both" {
        if let Some(ref cid) = config.outlook_client_id {
            let tok = crate::get_valid_token(data_dir, "outlook", cid, &config.outlook_tenant_id).await?;
            let msgs = OutlookClient::new(&tok.access_token).fetch_since(since_ts).await?;
            all_messages.extend(msgs);
        }
    }
    let uses_imap = config.provider == "imap"
        || config.provider.split(',').any(|p| p.trim() == "imap");
    if uses_imap {
        if let Ok(Some(creds)) = crate::load_imap_credentials(data_dir) {
            match imap_client::fetch_since(creds, since_ts).await {
                Ok(msgs) => all_messages.extend(msgs),
                Err(e) => eprintln!("IMAP fetch error: {e}"),
            }
        }
    }

    let field_ids = ensure_crm_fields(doc)?;
    let inbox_col = resolve_inbox_col(doc, config.inbox_col_id.as_deref())?;
    let author = hex::encode(actor_pk);

    let mut by_contact: HashMap<String, Vec<MailMessage>> = HashMap::new();
    for msg in all_messages {
        by_contact.entry(msg.from_email.clone()).or_default().push(msg);
    }

    let mut result = SyncResult { contacts_created: 0, contacts_updated: 0, emails_added: 0 };

    for (email_addr, mut messages) in by_contact {
        messages.sort_by(|a, b| b.date.cmp(&a.date));

        let (card_id, created) = upsert_contact(doc, &email_addr, &messages[0], &inbox_col, &field_ids, &author)?;
        if created { result.contacts_created += 1; } else { result.contacts_updated += 1; }

        let existing_ids = mail_comment_ids(doc, &card_id);
        let emails_to_add: Vec<&MailMessage> = messages.iter()
            .filter(|m| !existing_ids.contains_key(&m.id))
            .take(config.keep_last as usize)
            .collect();

        for msg in emails_to_add {
            let text = format!("**{}**\n{}\n\n{}", msg.subject, msg.date, msg.snippet);
            let comment = monotask_core::comment::add_comment(doc, &card_id, &text, &author, None, None, None)?;
            tag_comment_message_id(doc, &card_id, &comment.id, &msg.id)?;
            result.emails_added += 1;
        }

        prune_mail_comments(doc, &card_id, config.keep_last as usize)?;

        let last_seen = messages[0].date.clone();
        let count = existing_ids.len() + result.emails_added;
        let provider = messages[0].provider.clone();
        monotask_core::field::set_card_field(doc, &card_id, &field_ids.last_seen, &last_seen)?;
        monotask_core::field::set_card_field(doc, &card_id, &field_ids.email_count, &count.to_string())?;
        monotask_core::field::set_card_field(doc, &card_id, &field_ids.provider, &provider)?;
    }

    let now = chrono::Utc::now().to_rfc3339();
    doc.put(automerge::ROOT, "mail_last_sync", now.as_str())?;

    Ok(result)
}

struct FieldIds {
    email: String,
    last_seen: String,
    email_count: String,
    provider: String,
    labels: String,
}

fn ensure_crm_fields(doc: &mut AutoCommit) -> Result<FieldIds> {
    let defs = [
        ("mail_field_email",       "Email",       "text",   vec![]),
        ("mail_field_last_seen",   "Last Seen",   "date",   vec![]),
        ("mail_field_email_count", "Email Count", "number", vec![]),
        ("mail_field_provider",    "Provider",    "select", vec!["gmail","outlook"]),
        ("mail_field_labels",      "Labels",      "text",   vec![]),
    ];

    let mut ids = ["", "", "", "", ""].map(|s| s.to_string());

    for (i, (key, name, ftype, opts)) in defs.iter().enumerate() {
        if let Some(existing) = crate::get_mail_field_id(doc, key) {
            ids[i] = existing;
        } else {
            let ft = monotask_core::field::FieldType::from_str(ftype).unwrap_or(monotask_core::field::FieldType::Text);
            let options: Vec<String> = opts.iter().map(|s| s.to_string()).collect();
            let field = monotask_core::field::create_field(doc, name, ft, options, None, false)?;
            crate::set_mail_field_id(doc, key, &field.id)?;
            ids[i] = field.id;
        }
    }

    let [email, last_seen, email_count, provider, labels] = ids;
    Ok(FieldIds { email, last_seen, email_count, provider, labels })
}

fn resolve_inbox_col(doc: &AutoCommit, preferred: Option<&str>) -> Result<String> {
    let cols = monotask_core::column::list_columns(doc)?;
    if let Some(col_id) = preferred {
        if cols.iter().any(|c| c.id == col_id) {
            return Ok(col_id.to_string());
        }
    }
    cols.into_iter().next()
        .map(|c| c.id)
        .ok_or_else(|| anyhow::anyhow!("Board has no columns"))
}

fn upsert_contact(
    doc: &mut AutoCommit,
    email_addr: &str,
    first_msg: &MailMessage,
    col_id: &str,
    field_ids: &FieldIds,
    author: &str,
) -> Result<(String, bool)> {
    let cards_map = monotask_core::get_cards_map_readonly(doc)?;
    let all_ids: Vec<String> = doc.keys(&cards_map).map(|k| k.to_string()).collect();

    for cid in &all_ids {
        if let Ok(Some(val)) = monotask_core::field::get_card_field(doc, cid, &field_ids.email) {
            if val.to_lowercase() == email_addr {
                let card_obj = match doc.get(&cards_map, cid.as_str())? {
                    Some((_, o)) => o, None => continue,
                };
                let deleted = matches!(doc.get(&card_obj, "deleted")?,
                    Some((automerge::Value::Scalar(s), _)) if matches!(s.as_ref(), automerge::ScalarValue::Boolean(true)));
                if !deleted { return Ok((cid.clone(), false)); }
            }
        }
    }

    let actor_pk = hex::decode(author).unwrap_or_else(|_| vec![0u8; 32]);
    let members = vec![actor_pk.clone()];
    let title = if first_msg.from_name == email_addr {
        email_addr.to_string()
    } else {
        format!("{} <{}>", first_msg.from_name, email_addr)
    };
    let card = monotask_core::card::create_card(doc, col_id, &title, &actor_pk, &members)?;
    monotask_core::field::set_card_field(doc, &card.id, &field_ids.email, email_addr)?;
    monotask_core::field::apply_default_fields(doc, &card.id)?;
    let _ = author;
    Ok((card.id, true))
}

fn mail_comment_ids(doc: &AutoCommit, card_id: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let card_obj = match monotask_core::card::get_card_obj(doc, card_id) {
        Ok(o) => o, Err(_) => return map,
    };
    let comments = match monotask_core::comment::get_comments_list(doc, &card_obj) {
        Ok(l) => l, Err(_) => return map,
    };
    for i in 0..doc.length(&comments) {
        if let Ok(Some((_, c_obj))) = doc.get(&comments, i) {
            let deleted = matches!(doc.get(&c_obj, "deleted"),
                Ok(Some((automerge::Value::Scalar(s), _))) if matches!(s.as_ref(), automerge::ScalarValue::Boolean(true)));
            if deleted { continue; }
            if let (Ok(Some(cid)), Ok(Some(mid))) = (
                monotask_core::get_string(doc, &c_obj, "id"),
                monotask_core::get_string(doc, &c_obj, "mail_message_id"),
            ) {
                map.insert(mid, cid);
            }
        }
    }
    map
}

fn tag_comment_message_id(doc: &mut AutoCommit, card_id: &str, comment_id: &str, msg_id: &str) -> Result<()> {
    let card_obj = monotask_core::card::get_card_obj(doc, card_id)?;
    let comments = monotask_core::comment::get_comments_list(doc, &card_obj)?;
    for i in 0..doc.length(&comments) {
        if let Ok(Some((_, c_obj))) = doc.get(&comments, i) {
            if let Ok(Some(cid)) = monotask_core::get_string(doc, &c_obj, "id") {
                if cid == comment_id {
                    doc.put(&c_obj, "mail_message_id", msg_id)?;
                    return Ok(());
                }
            }
        }
    }
    Ok(())
}

fn prune_mail_comments(doc: &mut AutoCommit, card_id: &str, keep_last: usize) -> Result<()> {
    let card_obj = monotask_core::card::get_card_obj(doc, card_id)?;
    let comments = monotask_core::comment::get_comments_list(doc, &card_obj)?;
    let len = doc.length(&comments);

    let mut mail_entries: Vec<(usize, String, String)> = Vec::new(); // (idx, comment_id, created_at)
    for i in 0..len {
        if let Ok(Some((_, c_obj))) = doc.get(&comments, i) {
            let deleted = matches!(doc.get(&c_obj, "deleted"),
                Ok(Some((automerge::Value::Scalar(s), _))) if matches!(s.as_ref(), automerge::ScalarValue::Boolean(true)));
            if deleted { continue; }
            let has_mail_id = monotask_core::get_string(doc, &c_obj, "mail_message_id")
                .ok().flatten().is_some();
            if !has_mail_id { continue; }
            let cid = monotask_core::get_string(doc, &c_obj, "id").ok().flatten().unwrap_or_default();
            let ts = monotask_core::get_string(doc, &c_obj, "created_at").ok().flatten().unwrap_or_default();
            mail_entries.push((i, cid, ts));
        }
    }

    if mail_entries.len() <= keep_last { return Ok(()); }
    mail_entries.sort_by(|a, b| b.2.cmp(&a.2)); // newest first
    let to_delete: Vec<String> = mail_entries.into_iter().skip(keep_last).map(|(_, id, _)| id).collect();
    for cid in to_delete {
        let _ = monotask_core::comment::delete_comment(doc, card_id, &cid);
    }
    Ok(())
}
