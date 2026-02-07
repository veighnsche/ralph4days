# 028 — Rust Technical Debt Audit

Full audit of `src-tauri/` as of 2026-02-07. Covers lint suppressions, panicky code, stubs, duplication, and structural issues.

---

## HIGH PRIORITY

### Unvalidated production `.unwrap()`

- [ ] `commands.rs:1107` — `.unwrap()` on `instruction_override` with no prior validation. Will panic if `None`.

### Not-implemented stubs returning errors at runtime

- [ ] `commands.rs:293` — `start_loop` returns `Err("Not implemented")`
- [ ] `commands.rs:299` — `pause_loop` returns `Err("Not implemented")`
- [ ] `commands.rs:305` — `resume_loop` returns `Err("Not implemented")`
- [ ] `commands.rs:308` — `stop_loop` returns `Err("Not implemented")`
- [ ] `commands.rs:314` — `get_loop_state` returns `Err("Not implemented")`
- [ ] `loop_engine.rs` — empty struct (7 lines), only has `new()`

### Dead modules behind `#[allow(dead_code)]`

- [ ] `lib.rs:2` — `mod loop_engine` — entire module suppressed
- [ ] `lib.rs:5` — `mod types` — entire module suppressed

---

## MEDIUM PRIORITY

### `clippy::too_many_arguments` (need param structs)

All in `commands.rs`. Replace individual params with a single deserialized struct.

- [ ] `commands.rs:396` — `create_task` (12 params)
- [ ] `commands.rs:619` — `create_feature` (9 params)
- [ ] `commands.rs:651` — `update_feature` (9 params)
- [ ] `commands.rs:758` — `update_task` (13 params)

### Duplicated `get_db` + `.unwrap()` pattern (25 instances)

Every command repeats this two-liner:
```rust
let guard = get_db(&state)?;
let db = guard.as_ref().unwrap();
```
- [ ] Refactor `get_db()` to return `&YamlDb` directly, eliminating 25 `.unwrap()` calls in one shot

### Duplicated `map_err(|e| e.to_string())` pattern (15 instances)

- [ ] `commands.rs` — 9 occurrences
- [ ] `terminal/manager.rs` — 6 occurrences
- [ ] Extract a helper trait or extension method

### Large functions

- [ ] `commands.rs:199` — `initialize_ralph_project` (59 lines)
- [ ] `commands.rs:318` — `scan_for_ralph_projects` (69 lines, contains nested `scan_recursive` fn that should be extracted)

### `commands.rs` file size

- [ ] 1,257 lines in a single file. Consider splitting into submodules (db commands, loop commands, project commands, terminal commands).

---

## LOW PRIORITY

### `#[allow(dead_code)]` on struct field

- [ ] `terminal/session.rs:34` — `reader_handle` field. If kept for thread ownership, replace allow with a WHY comment. If unused, delete.

### Duplicated struct-to-struct field cloning

- [ ] `commands.rs:494-513` — discipline transformation, field-by-field clone
- [ ] `commands.rs:603-614` — feature transformation, field-by-field clone
- [ ] Consider `From` trait impls to centralize these conversions

### Repeated `.ralph` path construction

- [ ] `commands.rs` lines 1006, 1020, 1037, 1142, 1168, 1182, 1196 — `project_path.join(".ralph").join(...)` repeated throughout
- [ ] Extract a path helper or constants

### Discarded `Result` values

Acceptable in context but worth noting:
- [ ] `commands.rs:59` — `let _ = std::fs::remove_dir_all(...)` in `Drop` impl
- [ ] `terminal/manager.rs:104,121` — `let _ = app_clone.emit(...)` in background threads

---

## CLEAN (no issues found)

- No `unsafe` blocks
- No `todo!()` / `unimplemented!()` / `unreachable!()` macros
- No `panic!()` calls
- No commented-out code
- No deprecated API usage
- No stale `#[cfg]` attributes
- No `String` params where `&str` would suffice (Tauri boundary requires `String`)
- Test `.unwrap()` usage (90 in `generate_fixtures.rs`, 6 in terminal tests) — acceptable
