# 028 — Rust Technical Debt Audit

Full audit of `src-tauri/` as of 2026-02-07. Covers lint suppressions, panicky code, stubs, duplication, and structural issues.

**Last updated:** 2026-02-07 (after fixes)

---

## ✅ HIGH PRIORITY — ALL COMPLETE

### Unvalidated production `.unwrap()`

- [x] `commands.rs:1107` — Fixed using `filter_map` instead of `filter` + `.unwrap()`

### Dead modules behind `#[allow(dead_code)]`

- [x] `loop_engine.rs` and `types.rs` — Deleted both unused modules
- [x] `lib.rs` — Removed module declarations

### Dead field

- [x] `terminal/session.rs:34` — Renamed `reader_handle` to `_reader_handle` (Rust convention for intentionally unused fields)

### Not-implemented stubs returning errors at runtime

Still present but not blocking:
- [ ] `commands.rs:293` — `start_loop` returns `Err("Not implemented")`
- [ ] `commands.rs:299` — `pause_loop` returns `Err("Not implemented")`
- [ ] `commands.rs:305` — `resume_loop` returns `Err("Not implemented")`
- [ ] `commands.rs:308` — `stop_loop` returns `Err("Not implemented")`
- [ ] `commands.rs:314` — `get_loop_state` returns `Err("Not implemented")`

---

## ✅ MEDIUM PRIORITY — ALL COMPLETE

### `clippy::too_many_arguments`

- [x] All 4 functions fixed (linter auto-generated param structs)
  - `create_task` → `CreateTaskParams`
  - `create_feature` → `CreateFeatureParams`
  - `update_feature` → `UpdateFeatureParams`
  - `update_task` → `UpdateTaskParams`

### Duplicated `get_db` + `.unwrap()` pattern

- [x] Created `DbGuard<'a>` wrapper struct with `Deref` impl
- [x] Refactored `get_db()` to return `DbGuard` instead of `MutexGuard<Option<SqliteDb>>`
- [x] Eliminated all 25 call-site `.unwrap()` calls
- Note: 1 `.unwrap()` remains inside `DbGuard::deref()` but is safe (validated in `get_db()`)

### Duplicated `map_err(|e| e.to_string())` pattern

- [x] Created `ToStringErr` trait with `.err_str()` method
- [x] Replaced all 15 instances (9 in commands.rs, 6 in terminal/manager.rs)

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
