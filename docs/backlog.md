# Monotask Feature Backlog

Planned features — status as of 2026-07-01 after iroh migration (v1.3.0).

---

## Legend

- ✅ Done — shipped, tested
- 🔧 Partial — backend/schema exists, surface missing
- ❌ Not started

---

## 1. Comment Author Identity  `small`

**Status:** ✅ Done (v1.3.0)  
`identity.public_key_hex()` now passed as `author_key` in `card comment add`.

---

## 2. HLC Remote Clock Advancement  `small`

**Status:** ✅ Done (v1.3.0)  
Added `clock::observe_remote(hlc: &str)` to `monotask-core`. Advances logical counter from any remote HLC string. Tests added.

---

## 3. CLI Undo/Redo Commands  `small`

**Status:** ✅ Done (v1.3.0)  
`monotask board undo <BOARD_ID>` and `monotask board redo <BOARD_ID>` added. Uses same undo/redo stack tables as the Tauri UI.

---

## 4. Protocol Version Negotiation  `medium`

**Status:** ✅ Done (v1.3.0)  
`VersionHello { major: u16 }` / `VersionReject` exchange on the first bi-stream of every new iroh connection. `PROTOCOL_MAJOR = 1`. Incompatible peers are rejected with a log warning and disconnected.

---

## 5. Deep Link URL Scheme  `small`

**Status:** ✅ Done (v1.3.0)  
`monotask://` registered in `tauri.conf.json` via `tauri-plugin-deep-link`. Deep link events forwarded to JS as `monotask://open`. JS listener navigates to the board/card. `monotask app open <url>` CLI command added.

---

## 6. Mention Scanning & Index  `medium`

**Status:** ✅ Done (v1.3.0) — Tauri side wired  
`index_mentions()` helper scans card descriptions for `@token` patterns and upserts into `mention_index` after every `update_card_cmd`. CLI mention query commands are not yet added (low priority; `get_mention_suggestions_cmd` serves the Tauri UI).

---

## 7. Card Linking  `medium`

**Status:** ✅ Done (v1.3.0)  
`add_card_link`, `remove_card_link`, `list_card_links` in `monotask-core::card`. CLI: `monotask card link add/list/remove`. Links stored as `links` Map on the card Automerge object.

---

## 8. Peer Presence Heartbeats  `medium`

**Status:** ✅ Done (v1.3.0)  
`SyncRequest::Presence { pubkey, status, display_name }` added to protocol. 30s `presence_tick` broadcasts local status to all connected peers via a new bi-stream. Recipients update `space_members.presence` in storage.

---

## 9. Chat CLI Commands  `small`

**Status:** ✅ Done (v1.3.0)  
`monotask chat send <SPACE_ID> <TEXT>` and `monotask chat list <SPACE_ID> [--limit N]` added.

---

## 10. Card Image Attachments UI  `medium`

**Status:** ✅ Already done (pre-v1.3.0)  
Edit/Preview tabs, `triggerImageAttach()`, `renderAttachmentList()`, inline `img:id` rendering in `renderMarkdown()` were all already implemented in `index.html`.

---

## Implementation Order

| # | Item | Size | Dependency |
|---|------|------|------------|
| 1 | Comment author identity | tiny | — |
| 2 | HLC remote clock | tiny | — |
| 3 | CLI undo/redo | small | — |
| 4 | Deep link scheme + CLI open | small | — |
| 9 | Chat CLI | small | — |
| 6 | Mention scanning + CLI | medium | — |
| 5 | Protocol version negotiation | medium | — |
| 7 | Card linking | medium | — |
| 8 | Peer presence heartbeats | medium | — |
| 10 | Card image attachments UI | medium | — |
