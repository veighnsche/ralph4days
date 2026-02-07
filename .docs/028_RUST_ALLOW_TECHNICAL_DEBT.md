# 028 — Rust `#[allow(...)]` Technical Debt

Audit of all `#[allow(...)]` suppressions in `src-tauri/`. Each item is a lint bypass that should be resolved.

## `clippy::too_many_arguments`

All in `src-tauri/src/commands.rs`. Fix by replacing individual parameters with a single deserialized struct (Tauri commands support this).

- [x] `commands.rs:396` — `create_task` — needs a `CreateTaskParams` struct
- [x] `commands.rs:619` — `create_feature` — needs a `CreateFeatureParams` struct
- [x] `commands.rs:651` — `update_feature` — needs an `UpdateFeatureParams` struct
- [x] `commands.rs:758` — `update_task` — needs an `UpdateTaskParams` struct

## `dead_code` (unused modules)

Both in `src-tauri/src/lib.rs`. Either wire up or delete.

- [ ] `lib.rs:2` — `mod loop_engine` — entire module suppressed as dead code
- [ ] `lib.rs:5` — `mod types` — entire module suppressed as dead code

## `dead_code` (struct field)

- [ ] `terminal/session.rs:34` — `reader_handle` field on session struct. If stored intentionally to keep the thread alive, replace `#[allow(dead_code)]` with a WHY comment. If truly unused, remove it.
