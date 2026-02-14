# Terminal Bridge Rust-TS Type Sharing Checklist

## Canonical design alignment

- `terminal_bridge` backend subsystem:
  - commands: `src-tauri/src/commands/terminal_bridge.rs`
  - event names + payload runtime structs: `crates/ralph-contracts/src/terminal.rs`
  - shared command arg contracts: `src-tauri/src/terminal/contract.rs`
- `terminal_bridge` frontend subsystem:
  - string constants: `src/lib/terminal/terminalBridgeContract.ts`
  - bridge client + listeners: `src/lib/terminal/terminalBridgeClient.ts`
  - session orchestration: `src/lib/terminal/session.ts`
- Intentional exclusions:
  - runtime-only helper functions in `terminal_bridge.rs` and `session.ts` are not shared types.
  - terminal UI component props/state are not wire-contract types.

## Source-of-truth decision

- Chosen option: **A**.
- Command/event strings stay hand-authored in `src/lib/terminal/terminalBridgeContract.ts`.
- Rust/TS sharing is used for payload and argument **types** only (via `#[ipc_type]` + `just types`).

## Checklist

- [x] 1. Define scope and freeze current contract.
- [x] 2. Create shared Rust contract types for `terminal_bridge`.
  - Added `#[ipc_type]` command arg structs in `src-tauri/src/terminal/contract.rs`.
  - Exported event payload types via `#[ipc_type]` in `crates/ralph-contracts/src/terminal.rs`.
- [x] 3. Export command/event string constants from one Rust location.
  - Backend event constants remain canonical in `crates/ralph-contracts/src/terminal.rs`.
  - TS constant generation intentionally not used (Option A).
- [x] 4. Replace inline backend command arg usage with shared structs where compatible.
  - `src-tauri/src/commands/terminal_bridge.rs` now routes command wrappers through shared arg structs.
  - Tauri command names and wire keys are unchanged.
- [x] 5. Wire frontend to generated types.
  - `src/lib/terminal/terminalBridgeClient.ts` imports terminal bridge wire types from `src/types/generated.ts`.
  - Removed local duplicated payload type declarations.
  - Invoke payloads now typed with generated command arg types.
- [x] 6. Decide string source of truth for command/event names.
  - Documented above (Option A).
- [x] 7. Add drift tests for command and event names.
  - Backend event-name stability test: `crates/ralph-contracts/src/terminal.rs`.
  - Frontend constant drift test: `src/lib/terminal/terminalBridgeContract.test.ts`.
- [x] 8. Regenerate shared types.
  - Ran `just types`.
  - Verified `src/types/generated.ts` now includes terminal bridge command arg + event types.
- [x] 9. Update terminal bridge frontend tests.
  - Updated `src/lib/terminal/terminalBridgeClient.test.ts`.
  - Updated `src/lib/terminal/session.test.ts`.
- [x] 10. Run focused validation.
  - `just test-terminal-bridge-backend`
  - `bun test:run src/lib/terminal/terminalBridgeClient.test.ts src/lib/terminal/session.test.ts`
- [x] 11. Run full type and lint checks.
  - `just types-check`
  - `just check-all`
  - `types-check` executed and correctly reports `src/types/generated.ts` is changed (expected until committed).
  - `check-all` executed and failed on missing tool `oxlint` in this environment.
- [x] 12. Add maintenance guardrails.
  - Added reference note in `docs/terminal_bridge_backend_testing.md`.
  - Added source comment in `src/lib/terminal/terminalBridgeClient.ts`.
- [x] 13. Acceptance criteria.
  - No duplicated terminal bridge payload types remain between frontend and backend.
  - Command/event name drift is test-detected.
  - Terminal bridge tests pass; full lint/check execution is environment-blocked as noted above.
