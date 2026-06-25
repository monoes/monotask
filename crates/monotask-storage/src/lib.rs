pub mod board;
pub mod card_number;
pub mod schema;
pub mod space;

use rusqlite::Connection;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("automerge error: {0}")]
    Automerge(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Extract (card_id, field_id, value_text, value_num, value_date) tuples for all
/// non-deleted cards in the doc. Uses field_definitions type map to populate the
/// correct typed column instead of guessing from value shape.
fn extract_custom_fields(
    doc: &automerge::AutoCommit,
) -> Vec<(String, String, String, Option<f64>, Option<String>)> {
    use automerge::ReadDoc;
    use monotask_core::field::FieldType;

    // Build field_id → FieldType lookup once
    let field_types: std::collections::HashMap<String, FieldType> = {
        let defs = match doc.get(automerge::ROOT, "field_definitions").ok().flatten() {
            Some((_, id)) => id,
            None => return vec![],
        };
        let keys: Vec<String> = doc.keys(&defs).map(|k| k.to_string()).collect();
        let mut m = std::collections::HashMap::new();
        for key in keys {
            if let Some((_, obj)) = doc.get(&defs, key.as_str()).ok().flatten() {
                if let Some(ts) = monotask_core::get_string(doc, &obj, "type").ok().flatten() {
                    if let Some(ft) = FieldType::from_str(&ts) {
                        m.insert(key, ft);
                    }
                }
            }
        }
        m
    };

    if field_types.is_empty() {
        return vec![];
    }

    let cards_map = match monotask_core::get_cards_map_readonly(doc) {
        Ok(id) => id,
        Err(_) => return vec![],
    };
    let card_ids: Vec<String> = doc.keys(&cards_map).map(|k| k.to_string()).collect();
    let mut result = Vec::new();

    for card_id in card_ids {
        let card_obj = match doc.get(&cards_map, card_id.as_str()).ok().flatten() {
            Some((_, id)) => id,
            None => continue,
        };
        let is_deleted = match doc.get(&card_obj, "deleted").ok().flatten() {
            Some((automerge::Value::Scalar(s), _)) => {
                matches!(s.as_ref(), automerge::ScalarValue::Boolean(true))
            }
            _ => false,
        };
        if is_deleted { continue; }

        let cf_map = match doc.get(&card_obj, "custom_fields").ok().flatten() {
            Some((_, id)) => id,
            None => continue,
        };

        let field_ids: Vec<String> = doc.keys(&cf_map).map(|k| k.to_string()).collect();
        for field_id in field_ids {
            let value_text = match doc.get(&cf_map, field_id.as_str()).ok().flatten() {
                Some((automerge::Value::Scalar(s), _)) => match s.as_ref() {
                    automerge::ScalarValue::Str(text) => text.to_string(),
                    _ => continue,
                },
                _ => continue,
            };

            let (value_num, value_date) = match field_types.get(&field_id) {
                Some(FieldType::Number) => (value_text.parse::<f64>().ok(), None),
                Some(FieldType::Date) => (None, Some(value_text.clone())),
                _ => (None, None),
            };

            result.push((card_id.clone(), field_id, value_text, value_num, value_date));
        }
    }
    result
}

/// Extract (card_id, number_string) pairs from an Automerge doc.
fn extract_card_numbers(doc: &automerge::AutoCommit) -> Vec<(String, String)> {
    use automerge::ReadDoc;
    let cards_map = match monotask_core::get_cards_map_readonly(doc) {
        Ok(id) => id,
        Err(_) => return vec![],
    };
    let all_keys: Vec<String> = doc.keys(&cards_map).map(|k| k.to_string()).collect();
    let result: Vec<(String, String)> = all_keys.into_iter()
        .filter_map(|card_id| {
            let card_obj = doc.get(&cards_map, &card_id).ok()?.map(|(_, id)| id)?;
            let is_deleted = match doc.get(&card_obj, "deleted").ok().flatten() {
                Some((automerge::Value::Scalar(s), _)) => {
                    matches!(s.as_ref(), automerge::ScalarValue::Boolean(true))
                }
                _ => false,
            };
            if is_deleted { return None; }
            let number = monotask_core::get_string(doc, &card_obj, "number").ok().flatten()?;
            Some((card_id, number))
        })
        .collect();
    result
}

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn open(dir: &Path) -> Result<Self, StorageError> {
        std::fs::create_dir_all(dir)?;
        let db_path = dir.join("kanban.db");
        let conn = Connection::open(&db_path)?;
        schema::run_migrations(&conn)?;
        schema::run_migrations_v2(&conn)?;
        schema::run_migrations_v3(&conn)?;
        schema::run_migrations_v4(&conn)?;
        schema::run_migrations_v5(&conn)?;
        schema::run_migrations_v6(&conn)?;
        schema::run_migrations_v7(&conn)?;
        Ok(Self { conn })
    }

    pub fn open_in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory()?;
        schema::run_migrations(&conn)?;
        schema::run_migrations_v2(&conn)?;
        schema::run_migrations_v3(&conn)?;
        schema::run_migrations_v4(&conn)?;
        schema::run_migrations_v5(&conn)?;
        schema::run_migrations_v6(&conn)?;
        schema::run_migrations_v7(&conn)?;
        Ok(Self { conn })
    }

    pub fn save_board(&mut self, board_id: &str, doc: &mut automerge::AutoCommit) -> Result<(), StorageError> {
        let cards = extract_card_numbers(doc);
        // Wrap board save + index rebuild in one transaction to prevent race
        // conditions from concurrent CLI/daemon processes hitting the UNIQUE
        // constraint on (board_id, number).
        let tx = self.conn.transaction().map_err(StorageError::Sqlite)?;
        let bytes = doc.save();
        tx.execute(
            "INSERT INTO boards (board_id, automerge_doc, last_modified)
             VALUES (?1, ?2, unixepoch())
             ON CONFLICT(board_id) DO UPDATE SET
                 automerge_doc = excluded.automerge_doc,
                 last_modified = excluded.last_modified",
            rusqlite::params![board_id, bytes],
        ).map_err(StorageError::Sqlite)?;
        tx.execute(
            "DELETE FROM card_number_index WHERE board_id = ?1",
            rusqlite::params![board_id],
        ).map_err(StorageError::Sqlite)?;
        for (card_id, number) in &cards {
            // ON CONFLICT DO NOTHING handles both the (board_id, card_id) PK and
            // the (board_id, number) unique index. The latter can be hit when two
            // peers concurrently assign the same sequential number to different
            // cards — the Automerge doc is correct but the index can only hold one
            // mapping per number. The first card inserted wins; callers that need
            // to resolve the number unambiguously should use the card UUID instead.
            tx.execute(
                "INSERT OR IGNORE INTO card_number_index (board_id, card_id, number)
                 VALUES (?1, ?2, ?3)",
                rusqlite::params![board_id, card_id, number],
            ).map_err(StorageError::Sqlite)?;
        }
        // Rebuild custom field index
        let custom_fields = extract_custom_fields(doc);
        tx.execute(
            "DELETE FROM card_custom_field_index WHERE board_id = ?1",
            rusqlite::params![board_id],
        ).map_err(StorageError::Sqlite)?;
        for (card_id, field_id, value_text, value_num, value_date) in &custom_fields {
            tx.execute(
                "INSERT OR IGNORE INTO card_custom_field_index
                     (board_id, card_id, field_id, value_text, value_num, value_date)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![board_id, card_id, field_id, value_text, value_num, value_date],
            ).map_err(StorageError::Sqlite)?;
        }
        tx.commit().map_err(StorageError::Sqlite)?;
        Ok(())
    }

    /// Save a board doc from raw bytes, setting the is_system flag.
    /// Used for system docs (e.g., chat docs) that don't need card number indexing.
    pub fn save_board_bytes(&self, board_id: &str, bytes: &[u8], is_system: bool) -> Result<(), StorageError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        self.conn.execute(
            "INSERT OR REPLACE INTO boards (board_id, automerge_doc, last_modified, is_system)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![board_id, bytes, now, is_system as i32],
        ).map_err(StorageError::Sqlite)?;
        Ok(())
    }

    pub fn load_board(&self, board_id: &str) -> Result<automerge::AutoCommit, StorageError> {
        board::load_board(&self.conn, board_id)
    }

    pub fn list_board_ids(&self) -> Result<Vec<String>, StorageError> {
        board::list_board_ids(&self.conn)
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Resolve a card reference: tries card number index first, falls back to UUID passthrough.
    pub fn resolve_card_ref(&self, board_id: &str, card_ref: &str) -> Result<String, StorageError> {
        card_number::resolve_card_ref(&self.conn, board_id, card_ref)
    }

    pub fn delete_board(&self, board_id: &str) -> Result<(), StorageError> {
        let tx = self.conn.unchecked_transaction().map_err(StorageError::Sqlite)?;
        tx.execute(
            "DELETE FROM card_search_index WHERE board_id = ?1",
            rusqlite::params![board_id],
        ).map_err(StorageError::Sqlite)?;
        tx.execute(
            "DELETE FROM card_number_index WHERE board_id = ?1",
            rusqlite::params![board_id],
        ).map_err(StorageError::Sqlite)?;
        tx.execute(
            "DELETE FROM card_custom_field_index WHERE board_id = ?1",
            rusqlite::params![board_id],
        ).map_err(StorageError::Sqlite)?;
        tx.execute(
            "DELETE FROM boards WHERE board_id = ?1",
            rusqlite::params![board_id],
        ).map_err(StorageError::Sqlite)?;
        tx.commit().map_err(StorageError::Sqlite)?;
        Ok(())
    }

    /// Query card IDs matching a custom field filter.
    /// `op` is one of: `=`, `!=`, `>`, `>=`, `<`, `<=`, `~` (contains).
    /// Returns a `Vec<String>` of card IDs that match.
    pub fn query_cards_by_field(
        &self,
        board_id: &str,
        field_id: &str,
        field_type: &monotask_core::field::FieldType,
        op: &str,
        value: &str,
    ) -> Result<Vec<String>, StorageError> {
        use monotask_core::field::FieldType;
        let sql_op = match op {
            "=" => "=",
            "!=" => "!=",
            ">" => ">",
            ">=" => ">=",
            "<" => "<",
            "<=" => "<=",
            "~" => "LIKE",
            _ => "=",
        };

        let sql = match field_type {
            FieldType::Number => format!(
                "SELECT card_id FROM card_custom_field_index
                 WHERE board_id = ?1 AND field_id = ?2 AND value_num {} ?3",
                sql_op
            ),
            FieldType::Date => format!(
                "SELECT card_id FROM card_custom_field_index
                 WHERE board_id = ?1 AND field_id = ?2 AND value_date {} ?3",
                sql_op
            ),
            _ => format!(
                "SELECT card_id FROM card_custom_field_index
                 WHERE board_id = ?1 AND field_id = ?2 AND value_text {} ?3",
                sql_op
            ),
        };

        let like_value = if op == "~" {
            format!("%{}%", value)
        } else {
            value.to_string()
        };

        let mut stmt = self.conn.prepare(&sql).map_err(StorageError::Sqlite)?;
        let rows = match field_type {
            FieldType::Number => {
                let num: f64 = value.parse().unwrap_or(0.0);
                stmt.query_map(rusqlite::params![board_id, field_id, num], |r| r.get(0))
                    .map_err(StorageError::Sqlite)?
                    .filter_map(|r| r.ok())
                    .collect()
            }
            _ => stmt
                .query_map(rusqlite::params![board_id, field_id, like_value], |r| r.get(0))
                .map_err(StorageError::Sqlite)?
                .filter_map(|r| r.ok())
                .collect(),
        };
        Ok(rows)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use automerge::AutoCommit;

    #[test]
    fn save_and_load_board_round_trips() {
        let tmp = tempfile::tempdir().unwrap();
        let mut storage = Storage::open(tmp.path()).unwrap();
        let mut doc = AutoCommit::new();
        monotask_core::init_doc(&mut doc).unwrap();
        storage.save_board("board1", &mut doc).unwrap();
        let loaded = storage.load_board("board1").unwrap();
        assert!(automerge::ReadDoc::get(&loaded, automerge::ROOT, "columns").unwrap().is_some());
    }

    #[test]
    fn list_boards_returns_saved_boards() {
        let mut storage = Storage::open_in_memory().unwrap();
        let mut doc = AutoCommit::new();
        monotask_core::init_doc(&mut doc).unwrap();
        storage.save_board("board-a", &mut doc).unwrap();
        storage.save_board("board-b", &mut doc).unwrap();
        let ids = storage.list_board_ids().unwrap();
        assert_eq!(ids.len(), 2);
    }
}
