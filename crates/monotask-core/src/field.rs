use automerge::{AutoCommit, ObjType, ReadDoc, transaction::Transactable};
use serde::{Deserialize, Serialize};
use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    Text,
    Number,
    Date,
    Select,
    MultiSelect,
    Checkbox,
}

impl FieldType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Number => "number",
            Self::Date => "date",
            Self::Select => "select",
            Self::MultiSelect => "multi_select",
            Self::Checkbox => "checkbox",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "text" => Some(Self::Text),
            "number" => Some(Self::Number),
            "date" => Some(Self::Date),
            "select" => Some(Self::Select),
            "multi_select" => Some(Self::MultiSelect),
            "checkbox" => Some(Self::Checkbox),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub id: String,
    pub name: String,
    pub field_type: FieldType,
    pub options: Vec<String>,
    pub order: u64,
    pub archived: bool,
    pub default_value: Option<String>,
    pub auto_apply: bool,
}

/// Get or lazily create the `field_definitions` map at the board root (mutable).
fn get_or_create_field_defs_map(doc: &mut AutoCommit) -> Result<automerge::ObjId> {
    match doc.get(automerge::ROOT, "field_definitions")? {
        Some((_, id)) => Ok(id),
        None => Ok(doc.put_object(automerge::ROOT, "field_definitions", ObjType::Map)?),
    }
}

fn read_field_def(doc: &AutoCommit, obj: &automerge::ObjId) -> Result<Option<FieldDefinition>> {
    let id = match crate::get_string(doc, obj, "id")? {
        Some(s) => s,
        None => return Ok(None),
    };
    let name = crate::get_string(doc, obj, "name")?.unwrap_or_default();
    let type_str = crate::get_string(doc, obj, "type")?.unwrap_or_default();
    let field_type = FieldType::from_str(&type_str).unwrap_or(FieldType::Text);
    let default_value = crate::get_string(doc, obj, "default_value")?;

    let archived = match doc.get(obj, "archived")? {
        Some((automerge::Value::Scalar(s), _)) => {
            matches!(s.as_ref(), automerge::ScalarValue::Boolean(true))
        }
        _ => false,
    };
    let auto_apply = match doc.get(obj, "auto_apply")? {
        Some((automerge::Value::Scalar(s), _)) => {
            matches!(s.as_ref(), automerge::ScalarValue::Boolean(true))
        }
        _ => false,
    };
    let order = match doc.get(obj, "order")? {
        Some((automerge::Value::Scalar(s), _)) => match s.as_ref() {
            automerge::ScalarValue::Uint(n) => *n,
            automerge::ScalarValue::Int(n) => (*n).max(0) as u64,
            _ => 0,
        },
        _ => 0,
    };
    let options = match doc.get(obj, "options")? {
        Some((_, list_id)) => {
            let mut v = Vec::new();
            for i in 0..doc.length(&list_id) {
                if let Some((automerge::Value::Scalar(s), _)) = doc.get(&list_id, i)? {
                    if let automerge::ScalarValue::Str(text) = s.as_ref() {
                        v.push(text.to_string());
                    }
                }
            }
            v
        }
        None => vec![],
    };

    Ok(Some(FieldDefinition { id, name, field_type, options, order, archived, default_value, auto_apply }))
}

pub fn create_field(
    doc: &mut AutoCommit,
    name: &str,
    field_type: FieldType,
    options: Vec<String>,
    default_value: Option<String>,
    auto_apply: bool,
) -> Result<FieldDefinition> {
    let field_id = uuid::Uuid::new_v4().to_string();
    let defs = get_or_create_field_defs_map(doc)?;
    let order = doc.keys(&defs).count() as u64;

    let field_obj = doc.put_object(&defs, &field_id, ObjType::Map)?;
    doc.put(&field_obj, "id", field_id.as_str())?;
    doc.put(&field_obj, "name", name)?;
    doc.put(&field_obj, "type", field_type.as_str())?;
    doc.put(&field_obj, "order", order)?;
    doc.put(&field_obj, "archived", false)?;
    doc.put(&field_obj, "auto_apply", auto_apply)?;

    let opts_list = doc.put_object(&field_obj, "options", ObjType::List)?;
    for (i, opt) in options.iter().enumerate() {
        doc.insert(&opts_list, i, opt.as_str())?;
    }
    if let Some(ref dv) = default_value {
        doc.put(&field_obj, "default_value", dv.as_str())?;
    }

    Ok(FieldDefinition {
        id: field_id,
        name: name.to_string(),
        field_type,
        options,
        order,
        archived: false,
        default_value,
        auto_apply,
    })
}

pub fn list_fields(doc: &AutoCommit) -> Result<Vec<FieldDefinition>> {
    let defs = match doc.get(automerge::ROOT, "field_definitions")? {
        Some((_, id)) => id,
        None => return Ok(vec![]),
    };
    let keys: Vec<String> = doc.keys(&defs).map(|k| k.to_string()).collect();
    let mut fields = Vec::new();
    for key in keys {
        if let Some((_, obj)) = doc.get(&defs, key.as_str())? {
            if let Some(def) = read_field_def(doc, &obj)? {
                fields.push(def);
            }
        }
    }
    fields.sort_by_key(|f| f.order);
    Ok(fields)
}

pub fn get_field_by_id(doc: &AutoCommit, field_id: &str) -> Result<Option<FieldDefinition>> {
    let defs = match doc.get(automerge::ROOT, "field_definitions")? {
        Some((_, id)) => id,
        None => return Ok(None),
    };
    match doc.get(&defs, field_id)? {
        Some((_, obj)) => read_field_def(doc, &obj),
        None => Ok(None),
    }
}

/// Resolve a field reference: exact UUID first, then case-insensitive name match.
/// Returns Err if ambiguous.
pub fn resolve_field_ref(doc: &AutoCommit, field_ref: &str) -> Result<Option<FieldDefinition>> {
    if let Some(def) = get_field_by_id(doc, field_ref)? {
        return Ok(Some(def));
    }
    let lower = field_ref.to_lowercase();
    let matches: Vec<_> = list_fields(doc)?
        .into_iter()
        .filter(|f| !f.archived && f.name.to_lowercase() == lower)
        .collect();
    match matches.len() {
        0 => Ok(None),
        1 => Ok(Some(matches.into_iter().next().unwrap())),
        _ => Err(crate::Error::NotFound(format!(
            "ambiguous field name '{}' — use field UUID instead",
            field_ref
        ))),
    }
}

pub fn rename_field(doc: &mut AutoCommit, field_id: &str, new_name: &str) -> Result<()> {
    let defs = get_or_create_field_defs_map(doc)?;
    match doc.get(&defs, field_id)? {
        Some((_, obj)) => {
            doc.put(&obj, "name", new_name)?;
            Ok(())
        }
        None => Err(crate::Error::NotFound(field_id.into())),
    }
}

pub fn update_field_default(
    doc: &mut AutoCommit,
    field_id: &str,
    default_value: Option<&str>,
    auto_apply: Option<bool>,
) -> Result<()> {
    let defs = get_or_create_field_defs_map(doc)?;
    match doc.get(&defs, field_id)? {
        Some((_, obj)) => {
            if let Some(dv) = default_value {
                doc.put(&obj, "default_value", dv)?;
            }
            if let Some(aa) = auto_apply {
                doc.put(&obj, "auto_apply", aa)?;
            }
            Ok(())
        }
        None => Err(crate::Error::NotFound(field_id.into())),
    }
}

pub fn archive_field(doc: &mut AutoCommit, field_id: &str) -> Result<()> {
    let defs = get_or_create_field_defs_map(doc)?;
    match doc.get(&defs, field_id)? {
        Some((_, obj)) => {
            doc.put(&obj, "archived", true)?;
            Ok(())
        }
        None => Err(crate::Error::NotFound(field_id.into())),
    }
}

// ---------- Card field value operations ----------

pub fn set_card_field(doc: &mut AutoCommit, card_id: &str, field_id: &str, value: &str) -> Result<()> {
    let card_obj = crate::card::get_card_obj(doc, card_id)?;
    let cf_map = match doc.get(&card_obj, "custom_fields")? {
        Some((_, id)) => id,
        None => doc.put_object(&card_obj, "custom_fields", ObjType::Map)?,
    };
    doc.put(&cf_map, field_id, value)?;
    Ok(())
}

pub fn get_card_field(doc: &AutoCommit, card_id: &str, field_id: &str) -> Result<Option<String>> {
    let card_obj = crate::card::get_card_obj(doc, card_id)?;
    match doc.get(&card_obj, "custom_fields")? {
        Some((_, cf_map)) => crate::get_string(doc, &cf_map, field_id),
        None => Ok(None),
    }
}

pub fn clear_card_field(doc: &mut AutoCommit, card_id: &str, field_id: &str) -> Result<()> {
    let card_obj = crate::card::get_card_obj(doc, card_id)?;
    if let Some((_, cf_map)) = doc.get(&card_obj, "custom_fields")? {
        let _ = doc.delete(&cf_map, field_id);
    }
    Ok(())
}

pub fn list_card_fields(doc: &AutoCommit, card_id: &str) -> Result<Vec<(String, String)>> {
    let card_obj = crate::card::get_card_obj(doc, card_id)?;
    match doc.get(&card_obj, "custom_fields")? {
        Some((_, cf_map)) => {
            let keys: Vec<String> = doc.keys(&cf_map).map(|k| k.to_string()).collect();
            let mut result = Vec::new();
            for key in keys {
                if let Some(val) = crate::get_string(doc, &cf_map, &key)? {
                    result.push((key, val));
                }
            }
            Ok(result)
        }
        None => Ok(vec![]),
    }
}

/// Apply auto_apply defaults onto a newly-created card.
/// Only writes for fields not already explicitly set.
pub fn apply_default_fields(doc: &mut AutoCommit, card_id: &str) -> Result<()> {
    let fields_to_apply: Vec<(String, String)> = {
        let defs_obj = match doc.get(automerge::ROOT, "field_definitions")? {
            Some((_, id)) => id,
            None => return Ok(()),
        };
        let keys: Vec<String> = doc.keys(&defs_obj).map(|k| k.to_string()).collect();
        let mut to_apply = Vec::new();
        for key in &keys {
            if let Some((_, obj)) = doc.get(&defs_obj, key.as_str())? {
                let auto_apply = match doc.get(&obj, "auto_apply")? {
                    Some((automerge::Value::Scalar(s), _)) => {
                        matches!(s.as_ref(), automerge::ScalarValue::Boolean(true))
                    }
                    _ => false,
                };
                if !auto_apply { continue; }
                let archived = match doc.get(&obj, "archived")? {
                    Some((automerge::Value::Scalar(s), _)) => {
                        matches!(s.as_ref(), automerge::ScalarValue::Boolean(true))
                    }
                    _ => false,
                };
                if archived { continue; }
                if let Some(dv) = crate::get_string(doc, &obj, "default_value")? {
                    to_apply.push((key.clone(), dv));
                }
            }
        }
        to_apply
    };

    let card_obj = crate::card::get_card_obj(doc, card_id)?;
    let cf_map = match doc.get(&card_obj, "custom_fields")? {
        Some((_, id)) => id,
        None => doc.put_object(&card_obj, "custom_fields", ObjType::Map)?,
    };
    for (field_id, dv) in fields_to_apply {
        // Explicit value already set? Skip.
        if doc.get(&cf_map, field_id.as_str())?.is_some() {
            continue;
        }
        doc.put(&cf_map, field_id.as_str(), dv.as_str())?;
    }
    Ok(())
}

/// Backfill defaults onto all non-deleted cards in the board for a specific field.
/// Returns the count of cards that were updated.
pub fn backfill_field_defaults(doc: &mut AutoCommit, field_id: &str) -> Result<usize> {
    let default_value = {
        let defs = match doc.get(automerge::ROOT, "field_definitions")? {
            Some((_, id)) => id,
            None => return Err(crate::Error::NotFound(field_id.into())),
        };
        let field_obj = match doc.get(&defs, field_id)? {
            Some((_, obj)) => obj,
            None => return Err(crate::Error::NotFound(field_id.into())),
        };
        match crate::get_string(doc, &field_obj, "default_value")? {
            Some(dv) => dv,
            None => return Ok(0),
        }
    };

    let cards_map = crate::get_cards_map_readonly(doc)?;
    let card_ids: Vec<String> = doc.keys(&cards_map).map(|k| k.to_string()).collect();

    let mut needs_default: Vec<String> = Vec::new();
    for card_id in &card_ids {
        let card_obj = match doc.get(&cards_map, card_id.as_str())? {
            Some((_, id)) => id,
            None => continue,
        };
        let is_deleted = match doc.get(&card_obj, "deleted")? {
            Some((automerge::Value::Scalar(s), _)) => {
                matches!(s.as_ref(), automerge::ScalarValue::Boolean(true))
            }
            _ => false,
        };
        if is_deleted { continue; }
        let already_set = match doc.get(&card_obj, "custom_fields")? {
            Some((_, cf_map)) => doc.get(&cf_map, field_id)?.is_some(),
            None => false,
        };
        if !already_set {
            needs_default.push(card_id.clone());
        }
    }

    let count = needs_default.len();
    for card_id in needs_default {
        let card_obj = crate::card::get_card_obj(doc, &card_id)?;
        let cf_map = match doc.get(&card_obj, "custom_fields")? {
            Some((_, id)) => id,
            None => doc.put_object(&card_obj, "custom_fields", ObjType::Map)?,
        };
        doc.put(&cf_map, field_id, default_value.as_str())?;
    }
    Ok(count)
}

/// Validate a value against a field's declared type and options.
pub fn validate_field_value(field: &FieldDefinition, value: &str) -> std::result::Result<(), String> {
    match field.field_type {
        FieldType::Number => {
            if value.parse::<f64>().is_err() {
                return Err(format!(
                    "field '{}' expects a number, got '{}'",
                    field.name, value
                ));
            }
        }
        FieldType::Date => {
            let parts: Vec<&str> = value.split('-').collect();
            let valid = parts.len() == 3
                && parts[0].len() == 4
                && parts[1].len() == 2
                && parts[2].len() == 2
                && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()));
            if !valid {
                return Err(format!(
                    "field '{}' expects a date (YYYY-MM-DD), got '{}'",
                    field.name, value
                ));
            }
        }
        FieldType::Checkbox => {
            if value != "true" && value != "false" {
                return Err(format!(
                    "field '{}' expects 'true' or 'false', got '{}'",
                    field.name, value
                ));
            }
        }
        FieldType::Select => {
            if !field.options.is_empty() && !field.options.iter().any(|o| o == value) {
                return Err(format!(
                    "field '{}': '{}' is not a valid option. valid options: {}",
                    field.name,
                    value,
                    field.options.join(", ")
                ));
            }
        }
        FieldType::MultiSelect => {
            if !field.options.is_empty() {
                for part in value.split('\x1f') {
                    if !field.options.iter().any(|o| o == part) {
                        return Err(format!(
                            "field '{}': '{}' is not a valid option. valid options: {}",
                            field.name,
                            part,
                            field.options.join(", ")
                        ));
                    }
                }
            }
        }
        FieldType::Text => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use automerge::AutoCommit;

    fn fresh_doc() -> AutoCommit {
        let mut doc = AutoCommit::new();
        crate::init_doc(&mut doc).unwrap();
        doc
    }

    #[test]
    fn create_and_list_field() {
        let mut doc = fresh_doc();
        let f = create_field(&mut doc, "Stage", FieldType::Select,
            vec!["Lead".into(), "Qualified".into()], Some("Lead".into()), true).unwrap();
        assert_eq!(f.name, "Stage");
        assert_eq!(f.options, vec!["Lead", "Qualified"]);
        assert_eq!(f.default_value, Some("Lead".into()));

        let list = list_fields(&doc).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, f.id);
    }

    #[test]
    fn create_field_on_existing_board_without_field_defs() {
        // Simulate a board that was created before field_definitions existed
        let mut doc = AutoCommit::new();
        // Only put columns/cards — no field_definitions
        doc.put_object(automerge::ROOT, "columns", ObjType::List).unwrap();
        doc.put_object(automerge::ROOT, "cards", ObjType::Map).unwrap();
        // Must succeed via lazy creation
        let f = create_field(&mut doc, "Priority", FieldType::Text, vec![], None, false).unwrap();
        assert_eq!(f.name, "Priority");
    }

    #[test]
    fn resolve_field_by_name_and_uuid() {
        let mut doc = fresh_doc();
        let f = create_field(&mut doc, "Company", FieldType::Text, vec![], None, false).unwrap();
        let by_uuid = resolve_field_ref(&doc, &f.id).unwrap().unwrap();
        assert_eq!(by_uuid.id, f.id);
        let by_name = resolve_field_ref(&doc, "company").unwrap().unwrap(); // case-insensitive
        assert_eq!(by_name.id, f.id);
    }

    #[test]
    fn set_and_get_card_field() {
        let mut doc = fresh_doc();
        let col_id = crate::column::create_column(&mut doc, "Todo").unwrap();
        let actor_pk = vec![1u8; 32];
        let card = crate::card::create_card(&mut doc, &col_id, "Deal", &actor_pk, &[actor_pk.clone()]).unwrap();
        let field = create_field(&mut doc, "Amount", FieldType::Number, vec![], None, false).unwrap();

        set_card_field(&mut doc, &card.id, &field.id, "15000").unwrap();
        let val = get_card_field(&doc, &card.id, &field.id).unwrap();
        assert_eq!(val, Some("15000".into()));
    }

    #[test]
    fn auto_apply_default_on_card_creation() {
        let mut doc = fresh_doc();
        let col_id = crate::column::create_column(&mut doc, "Todo").unwrap();
        let actor_pk = vec![1u8; 32];
        let field = create_field(&mut doc, "Stage", FieldType::Select,
            vec!["Lead".into()], Some("Lead".into()), true).unwrap();
        let card = crate::card::create_card(&mut doc, &col_id, "Acme Corp", &actor_pk, &[actor_pk.clone()]).unwrap();
        apply_default_fields(&mut doc, &card.id).unwrap();
        let val = get_card_field(&doc, &card.id, &field.id).unwrap();
        assert_eq!(val, Some("Lead".into()));
    }

    #[test]
    fn explicit_value_beats_auto_apply_default() {
        let mut doc = fresh_doc();
        let col_id = crate::column::create_column(&mut doc, "Todo").unwrap();
        let actor_pk = vec![1u8; 32];
        let field = create_field(&mut doc, "Stage", FieldType::Select,
            vec!["Lead".into(), "Qualified".into()], Some("Lead".into()), true).unwrap();
        let card = crate::card::create_card(&mut doc, &col_id, "Acme", &actor_pk, &[actor_pk.clone()]).unwrap();
        set_card_field(&mut doc, &card.id, &field.id, "Qualified").unwrap();
        apply_default_fields(&mut doc, &card.id).unwrap(); // must not overwrite
        let val = get_card_field(&doc, &card.id, &field.id).unwrap();
        assert_eq!(val, Some("Qualified".into()));
    }

    #[test]
    fn backfill_updates_only_unset_cards() {
        let mut doc = fresh_doc();
        let col_id = crate::column::create_column(&mut doc, "Todo").unwrap();
        let actor_pk = vec![1u8; 32];
        let field = create_field(&mut doc, "Stage", FieldType::Select,
            vec!["Lead".into()], Some("Lead".into()), false).unwrap();
        let c1 = crate::card::create_card(&mut doc, &col_id, "C1", &actor_pk, &[actor_pk.clone()]).unwrap();
        let c2 = crate::card::create_card(&mut doc, &col_id, "C2", &actor_pk, &[actor_pk.clone()]).unwrap();
        // Explicitly set c2 to something different
        set_card_field(&mut doc, &c2.id, &field.id, "Qualified").unwrap();

        let count = backfill_field_defaults(&mut doc, &field.id).unwrap();
        assert_eq!(count, 1); // Only c1 should get the default

        assert_eq!(get_card_field(&doc, &c1.id, &field.id).unwrap(), Some("Lead".into()));
        assert_eq!(get_card_field(&doc, &c2.id, &field.id).unwrap(), Some("Qualified".into())); // unchanged
    }

    #[test]
    fn validate_select_rejects_invalid_option() {
        let field = FieldDefinition {
            id: "f1".into(), name: "Stage".into(), field_type: FieldType::Select,
            options: vec!["Lead".into(), "Qualified".into()], order: 0,
            archived: false, default_value: None, auto_apply: false,
        };
        assert!(validate_field_value(&field, "Lead").is_ok());
        let err = validate_field_value(&field, "Qualifed").unwrap_err();
        assert!(err.contains("valid options"));
    }

    #[test]
    fn validate_number_rejects_non_numeric() {
        let field = FieldDefinition {
            id: "f1".into(), name: "Amount".into(), field_type: FieldType::Number,
            options: vec![], order: 0, archived: false, default_value: None, auto_apply: false,
        };
        assert!(validate_field_value(&field, "1500.50").is_ok());
        assert!(validate_field_value(&field, "abc").is_err());
    }

    #[test]
    fn validate_date_rejects_wrong_format() {
        let field = FieldDefinition {
            id: "f1".into(), name: "Close Date".into(), field_type: FieldType::Date,
            options: vec![], order: 0, archived: false, default_value: None, auto_apply: false,
        };
        assert!(validate_field_value(&field, "2026-09-01").is_ok());
        assert!(validate_field_value(&field, "09/01/2026").is_err());
    }
}
