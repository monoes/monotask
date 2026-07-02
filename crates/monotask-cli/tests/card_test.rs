//! Comprehensive integration tests for `card` CLI commands.

use std::process::Command;

fn cli(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_monotaskcli"))
        .args(["--data-dir", dir.to_str().unwrap()])
        .args(args)
        .output()
        .expect("failed to run monotaskcli")
}

fn json(out: &std::process::Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "CLI command failed.\nstdout: {stdout}\nstderr: {stderr}");
    serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("invalid JSON: {e}\nraw stdout: {stdout}"))
}

fn make_space(dir: &std::path::Path) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_monotaskcli"))
        .args(["--data-dir", dir.to_str().unwrap()])
        .args(["space", "create", "TestSpace"])
        .output()
        .expect("failed to create space");
    let text = String::from_utf8_lossy(&out.stdout);
    let start = text.rfind('(').expect("no '(' in space create output");
    let end = text.rfind(')').expect("no ')' in space create output");
    text[start + 1..end].trim().to_string()
}

/// Create a space, board, and one column; return (board_id, col_id).
fn setup_board_col(tmp: &std::path::Path) -> (String, String) {
    let sid = make_space(tmp);
    let board_id = json(&cli(tmp, &["board", "create", "TestBoard", "--space", &sid, "--json"]))["id"]
        .as_str().unwrap().to_string();
    let col_id = json(&cli(tmp, &["column", "create", &board_id, "Backlog", "--json"]))["id"]
        .as_str().unwrap().to_string();
    (board_id, col_id)
}

// ---------------------------------------------------------------------------
// card create
// ---------------------------------------------------------------------------

#[test]
fn card_create_returns_id_title_board_id_and_number() {
    let tmp = tempfile::tempdir().unwrap();
    let (board_id, col_id) = setup_board_col(tmp.path());

    let out = cli(tmp.path(), &["card", "create", &board_id, &col_id, "My Task", "--json"]);
    let card = json(&out);

    let id = card["id"].as_str().expect("card.id should be a string");
    assert!(!id.is_empty(), "card.id should not be empty");
    assert_eq!(card["title"], "My Task", "card.title mismatch");
    assert_eq!(card["board_id"], board_id.as_str(), "card.board_id mismatch");

    let number_str = card["number"].as_str().expect("card.number should be a string");
    assert!(!number_str.is_empty());
    assert!(number_str.contains('-'), "card.number should be 'prefix-seq' format, got: {number_str}");
    let seq_str = number_str.rsplit('-').next().unwrap();
    let seq: u64 = seq_str.parse()
        .unwrap_or_else(|_| panic!("seq part of number is not an integer: {number_str}"));
    assert!(seq >= 1, "first card seq should be >= 1");
}

// ---------------------------------------------------------------------------
// card view
// ---------------------------------------------------------------------------

#[test]
fn card_view_returns_full_card_details() {
    let tmp = tempfile::tempdir().unwrap();
    let (board_id, col_id) = setup_board_col(tmp.path());

    let created = json(&cli(tmp.path(), &["card", "create", &board_id, &col_id, "View Me", "--json"]));
    let card_id = created["id"].as_str().unwrap().to_string();

    let card = json(&cli(tmp.path(), &["card", "view", &board_id, &card_id, "--json"]));

    assert_eq!(card["id"].as_str().unwrap(), card_id);
    assert_eq!(card["title"], "View Me");

    let number = &card["number"];
    assert!(!number.is_null(), "card.number should not be null; got: {card}");
    let prefix = number["prefix"].as_str().expect("card.number.prefix should be a string");
    assert!(!prefix.is_empty());
    let seq = number["seq"].as_u64().expect("card.number.seq should be an unsigned integer");
    assert!(seq >= 1);

    assert_eq!(card["deleted"], false);
    assert_eq!(card["archived"], false);
}

// ---------------------------------------------------------------------------
// Sequential card numbers
// ---------------------------------------------------------------------------

#[test]
fn sequential_card_numbers_increase_on_same_board() {
    let tmp = tempfile::tempdir().unwrap();
    let (board_id, col_id) = setup_board_col(tmp.path());

    let c1 = json(&cli(tmp.path(), &["card", "create", &board_id, &col_id, "First",  "--json"]));
    let c2 = json(&cli(tmp.path(), &["card", "create", &board_id, &col_id, "Second", "--json"]));
    let c3 = json(&cli(tmp.path(), &["card", "create", &board_id, &col_id, "Third",  "--json"]));

    let id1 = c1["id"].as_str().unwrap().to_string();
    let id2 = c2["id"].as_str().unwrap().to_string();
    let id3 = c3["id"].as_str().unwrap().to_string();

    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
    assert_ne!(id1, id3);

    let v1 = json(&cli(tmp.path(), &["card", "view", &board_id, &id1, "--json"]));
    let v2 = json(&cli(tmp.path(), &["card", "view", &board_id, &id2, "--json"]));
    let v3 = json(&cli(tmp.path(), &["card", "view", &board_id, &id3, "--json"]));

    let seq1 = v1["number"]["seq"].as_u64().expect("card1 must have number.seq");
    let seq2 = v2["number"]["seq"].as_u64().expect("card2 must have number.seq");
    let seq3 = v3["number"]["seq"].as_u64().expect("card3 must have number.seq");

    assert!(seq1 >= 1);
    assert!(seq2 > seq1, "seq2 ({seq2}) should be > seq1 ({seq1})");
    assert!(seq3 > seq2, "seq3 ({seq3}) should be > seq2 ({seq2})");

    let pfx1 = v1["number"]["prefix"].as_str().unwrap();
    let pfx2 = v2["number"]["prefix"].as_str().unwrap();
    let pfx3 = v3["number"]["prefix"].as_str().unwrap();
    assert_eq!(pfx1, pfx2);
    assert_eq!(pfx2, pfx3);
}

#[test]
fn sequential_numbers_start_at_one_for_first_card() {
    let tmp = tempfile::tempdir().unwrap();
    let (board_id, col_id) = setup_board_col(tmp.path());

    let card = json(&cli(tmp.path(), &["card", "create", &board_id, &col_id, "Solo", "--json"]));
    let card_id = card["id"].as_str().unwrap().to_string();
    let view = json(&cli(tmp.path(), &["card", "view", &board_id, &card_id, "--json"]));

    let seq = view["number"]["seq"].as_u64().expect("number.seq must be present");
    assert_eq!(seq, 1, "first card on a fresh board should have seq = 1");
}

// ---------------------------------------------------------------------------
// Card numbers are per-board
// ---------------------------------------------------------------------------

#[test]
fn card_numbers_are_independent_per_board() {
    let tmp = tempfile::tempdir().unwrap();
    let sid = make_space(tmp.path());

    let board_a = json(&cli(tmp.path(), &["board", "create", "BoardA", "--space", &sid, "--json"]))["id"]
        .as_str().unwrap().to_string();
    let col_a = json(&cli(tmp.path(), &["column", "create", &board_a, "Col", "--json"]))["id"]
        .as_str().unwrap().to_string();

    let board_b = json(&cli(tmp.path(), &["board", "create", "BoardB", "--space", &sid, "--json"]))["id"]
        .as_str().unwrap().to_string();
    let col_b = json(&cli(tmp.path(), &["column", "create", &board_b, "Col", "--json"]))["id"]
        .as_str().unwrap().to_string();

    json(&cli(tmp.path(), &["card", "create", &board_a, &col_a, "A-one", "--json"]));
    json(&cli(tmp.path(), &["card", "create", &board_a, &col_a, "A-two", "--json"]));

    let b1 = json(&cli(tmp.path(), &["card", "create", &board_b, &col_b, "B-one", "--json"]));
    let b1_id = b1["id"].as_str().unwrap().to_string();
    let b1_view = json(&cli(tmp.path(), &["card", "view", &board_b, &b1_id, "--json"]));
    let b1_seq = b1_view["number"]["seq"].as_u64().expect("B card must have seq");

    assert_eq!(b1_seq, 1, "first card on board B should have seq=1 regardless of board A's count");
}

// ---------------------------------------------------------------------------
// card create without --json
// ---------------------------------------------------------------------------

#[test]
fn card_create_without_json_flag_exits_successfully() {
    let tmp = tempfile::tempdir().unwrap();
    let (board_id, col_id) = setup_board_col(tmp.path());

    let out = cli(tmp.path(), &["card", "create", &board_id, &col_id, "Plain Output"]);
    assert!(out.status.success(), "card create without --json should succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Plain Output"), "plain output should mention card title; got: {stdout}");
}

// ---------------------------------------------------------------------------
// card view: title round-trips exactly
// ---------------------------------------------------------------------------

#[test]
fn card_view_title_matches_creation_title_exactly() {
    let tmp = tempfile::tempdir().unwrap();
    let (board_id, col_id) = setup_board_col(tmp.path());

    let title = "Exact Title With Spaces";
    let created = json(&cli(tmp.path(), &["card", "create", &board_id, &col_id, title, "--json"]));
    let card_id = created["id"].as_str().unwrap().to_string();

    let viewed = json(&cli(tmp.path(), &["card", "view", &board_id, &card_id, "--json"]));
    assert_eq!(viewed["title"].as_str().unwrap(), title);
}
