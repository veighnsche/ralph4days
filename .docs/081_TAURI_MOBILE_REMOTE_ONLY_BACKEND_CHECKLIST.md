# 081 Tauri Mobile Remote-Only Backend Checklist

Goal: make the **Tauri mobile backend** (iOS/Android) **thin** and **remote-only**: it must *only* talk to `ralphd` over the existing WS wire protocol, and must not compile/link desktop-only local backend dependencies.

Notes:
- This checklist is intentionally scoped as a plan-only dump (implementation is expected to touch many files across adapters + contracts).
- Repo policy: fail-fast. No silent fallbacks for broken invariants.
- Progress updated: 2026-02-16.

## Decisions (Locked)
- [x] Canonical cross-platform IPC/domain DTO ownership lives in `crates/ralph-contracts` (single source of truth; strict decode; TS export owner).
- [x] Mobile registers the **full invoke command surface** (same command names as desktop) for max frontend reuse.
- [x] Mobile commands that require backend state **hard-fail unless connected** to `ralphd` (no local fallback on mobile).
- [x] Remote transport supports `ws://` and `wss://` (mobile-friendly TLS stack; prefer rustls).

## Success Criteria
- [x] `src-tauri` builds for desktop unchanged in behavior (hybrid local/remote remains on desktop).
- [ ] `src-tauri` builds for mobile targets without pulling `rusqlite`/`portable-pty`/`axum`/desktop windowing assumptions.
- [x] On mobile: calling any stateful command before `remote_connect` returns an explicit error telling the caller to connect first.
- [x] Protocol mismatch remains a hard failure on connect (already enforced in `RemoteWireFrameConnection::connect`).
- [x] `just verify` passes and `just types-check` confirms TS bindings are stable/up to date.

## Checklist

### 1) Inventory and Ownership Map
- [x] Enumerate which Rust types cross the IPC boundary today (command args/results + event payloads).
- [x] Identify all IPC-exposed DTOs that currently come from `sqlite-db` and `ralph-backend` modules.
- [x] Confirm which `src-tauri` modules are desktop-only: `api_server`, DB/session state, PTY, XDG/home-dir access, window lifecycle.

### 2) Extract Canonical DTOs into `ralph-contracts`
- [ ] Add a `crates/ralph-contracts/src/domain/` module tree for domain DTOs (tasks, subsystems, sessions, signals, etc).
- [ ] Move (or recreate with identical serialization) any IPC-relevant types currently defined in `crates/sqlite-db/src/types.rs`.
- [ ] Move command DTOs currently defined in `crates/ralph-backend/src/*_contract.rs` (or adjacent modules) into `ralph-contracts`.
- [ ] Ensure strict serde posture is preserved (`deny_unknown_fields` where appropriate; no defensive defaults for internal contracts).
- [ ] Ensure `ts-rs` export ownership is singular (types exported from `ralph-contracts`; avoid duplicate filenames).
- [ ] Update `src-tauri/tests/remote_strict_decode_contract_test.rs` to reference contract DTOs from `ralph-contracts`.

### 3) Refactor `sqlite-db` to Consume Contract DTOs
- [ ] Add dependency `ralph-contracts` to `crates/sqlite-db/Cargo.toml`.
- [ ] Replace internal `crate::types::*` usage with `ralph_contracts::domain::*` (or contract paths decided above).
- [ ] Keep DB implementation returning contract DTOs.
- [ ] If temporary re-exports are used for migration, keep them thin and clearly non-canonical.
- [ ] Remove/avoid TS export ownership for moved types from `sqlite-db` to prevent TS binding duplication.

### 4) Refactor `ralph-backend` to Consume Contract DTOs
- [ ] Change `ralph-backend` APIs to take/return `ralph-contracts` DTOs (not `sqlite-db` DTOs).
- [ ] Keep `ralph-backend` as the local implementation layer (it may still depend on `sqlite-db` internally on desktop/server).
- [ ] Ensure no DRY violations: the canonical type definitions live in `ralph-contracts` only.

### 5) Refactor `ralphd` (src-daemon) to Use Contract DTOs
- [ ] Update RPC decode/encode targets in `src-daemon/src/main.rs` to use `ralph-contracts` DTO paths.
- [ ] Keep RPC command names and payload shapes stable (`{ "args": ... }` for invoke payloads).
- [ ] Keep strict payload validation (unknown keys/fields should remain errors).

### 6) Make `src-tauri` Mobile Thin + Remote-Only

Cargo / feature gating:
- [ ] Split `src-tauri/Cargo.toml` dependencies into desktop-only vs mobile-compatible using `cfg(...)` target deps.
- [ ] Ensure mobile does not depend on `sqlite-db`, `prompt-builder`, `portable-pty`, `axum`, `tower*`, or desktop-only filesystem/XDG helpers.
- [x] Switch remote WS client to a TLS stack that works on mobile (prefer rustls; ensure `wss://` works).

Runtime init:
- [x] In `src-tauri/src/lib.rs`, gate desktop-only window creation, CLI plugin usage, and API server startup behind `#[cfg(not(mobile))]`.
- [x] Implement a mobile `run()` path that only initializes tracing, state, and invoke handlers (no explicit desktop windows).

State split:
- [x] Split `src-tauri/src/commands/state.rs` into `state_desktop.rs` and `state_mobile.rs`, with `state.rs` re-exporting `AppState` via `cfg`.
- [x] Mobile `AppState` contains only remote connection state and what’s required to re-emit events via the sink.

Command behavior:
- [x] Add a helper on mobile: `remote_rpc_client_required()` that fails with a clear "connect first" error if not connected.
- [x] For every command in `src-tauri/src/commands/*.rs`, apply this rule:
- [x] On mobile: always proxy to remote (`ralphd`) via `remote_invoke_*`.
- [x] On desktop: keep current behavior (proxy when connected; otherwise local implementation).
- [x] For desktop-only window commands (`window_splash_close`, `window_open_new`), keep names registered but return an explicit unsupported error on mobile.

### 7) Contract / Type Generation Gates
- [x] Run and keep passing: `just types-check` (or `just types` then verify `git diff` only changes expected generated TS, if any).
- [ ] Ensure no duplicate ts-rs output filenames (ownership invariant).

### 8) Test Plan
- [x] `cargo test -p ralph-contracts`
- [x] `cargo test --manifest-path src-tauri/Cargo.toml`
- [x] `cargo test --manifest-path src-daemon/Cargo.toml`
- [x] Run existing WS protocol/parity smoke tests under `src-daemon/tests/ws_*` and update them only for moved type paths.
- [ ] Add a mobile compile gate:
- [ ] `cargo check --manifest-path src-tauri/Cargo.toml --target <android/ios target>` (exact targets depend on toolchain availability).
- [ ] Add a `just check-mobile` recipe only if it can be made deterministic on this machine/toolchain.

### 10) Current Implementation Notes (2026-02-16)
- [x] `src-tauri/src/commands/state.rs` now enforces remote connection on mobile and fails loudly when disconnected/not connected.
- [x] `src-tauri/src/commands/state.rs` now re-exports platform slices: `state_desktop.rs` and `state_mobile.rs`.
- [x] `src-tauri/src/commands/state_mobile.rs` now has remote-only app state (remote connection state only).
- [x] `src-tauri/src/commands/mod.rs` now routes mobile to `*_mobile.rs` command modules that only proxy remote invokes.
- [x] `src-tauri/src/lib.rs` now has mobile/desktop bootstrap split (mobile skips local API server + desktop window/CLI flow).
- [x] `src-tauri/src/commands/project.rs` desktop-only window commands now return explicit mobile unsupported errors.
- [x] `src-tauri/Cargo.toml` now uses rustls-based WS transport and desktop-only `tauri-plugin-cli`.
- [x] `src-daemon` now handles `stacks_metadata_list` and `terminal_emit_system_message` so mobile remote-only commands keep parity.
- [x] `just types-check` currently passes.
- [x] `just verify` currently passes.
- [ ] Mobile compile gate is still blocked locally because Android/iOS rust targets are not installed on this machine.
- [ ] Remaining: full mobile dependency diet (target-specific Cargo deps) to remove desktop crates from mobile builds.
- [ ] Remaining: contract DTO migration from `sqlite-db`/`ralph-backend` into `ralph-contracts`.

### 9) Rollout / Coordination (Minimize Collisions)
- [ ] Land changes in a dedicated PR that is intentionally not overlapping the other agent’s adapter-parity edits (coordinate file-touch boundaries).
- [ ] Keep diffs focused: avoid touching generated files unless the change requires it.
- [ ] PR includes a short "Failure posture" note: mobile remote-only, no silent fallbacks, and protocol mismatch hard-fails.
