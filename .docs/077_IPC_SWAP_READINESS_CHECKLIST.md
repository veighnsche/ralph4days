# IPC Swap Readiness Checklist (Make Current IPC “Perfect”)

Date: 2026-02-14

Goal: make the existing frontend-facing IPC contract (Tauri `invoke` + events) stable, typed, drift-proof, and transport-agnostic so it can be re-hosted behind `ralphd` with minimal/no UI changes.

## Current Snapshot (As Of 2026-02-14)
1. Backend portability (can we reuse “backend core” in both Tauri and `ralphd` today?): **No** (roughly **5/10**).
   1. Major blockers: business logic still lives in `src-tauri/src/commands/*`, and some subsystems still hard-depend on Tauri runtime types (notably `src-tauri/src/api_server.rs`).
2. IPC contract maturity (typed + stable + drift-tested enough to proxy 1:1): **Partial** (roughly **7.5/10**).
   1. Strongest area: terminal bridge contract (wire types + drift tests + buffering/replay semantics).
   2. Weakest area: protocol version handshake + command list drift testing.

## 0. Definitions
1. “IPC contract” = command names + request/response JSON shapes + event names + event payload shapes.
2. “Swap-ready” = you can run the same contract over a different transport (HTTP/WS/stdio) without rewriting product logic or UI behavior.

## 1. Contract Freeze + Inventory (Must-Have)
- [x] Declare the canonical command list owner (today: `src-tauri/src/lib.rs` `tauri::generate_handler![...]`).
- [ ] Add a drift test that asserts the canonical command list is stable (intentional changes require updating the test).
- [x] Declare the canonical event name owners per domain (now centralized in `crates/ralph-contracts/src/{terminal.rs,events.rs}`).
- [x] Add canonical event constants + drift tests for non-terminal events used by UI (done for `backend-diagnostic` in `crates/ralph-contracts/src/events.rs`; expand to other domains as needed).
- [ ] Add drift tests for every event name constant that the frontend listens to (terminal has this; others should match).
- [ ] Write down the “supported surface” for v1 `ralphd` parity:
  - Scope must be explicit: which commands/events are required for “remote UI parity” vs “nice-to-have”.

## 2. Protocol Versioning + Handshake (Must-Have)
- [x] Add a single canonical `PROTOCOL_VERSION` constant (Rust):
  - Owner: `crates/ralph-contracts/src/protocol.rs`
- [x] Add a “version” command that exists in local IPC too (so remote/local can be validated the same way):
  - Example: `protocol_version_get` and `health_get` (names are a decision).
- [x] Implement `protocol_version_get` in local Tauri IPC:
  - Owner: `src-tauri/src/commands/protocol.rs`
- [ ] Define the hard-fail policy:
  - If protocol mismatch: fail loudly with required server version + upgrade instruction.

## 3. Wire-Type Canonicalization (Must-Have)
- [ ] Ensure every IPC arg/result/event payload type is defined once in Rust and exported via `ts-rs`:
  - Pattern: `#[ipc_type]` + `#[serde(rename_all = "camelCase")]`.
  - Current status: event payload types are now shared via `crates/ralph-contracts` (terminal events + `backend-diagnostic`), and most command args/results are already exported via `#[ipc_type]` in `src-tauri/src/commands/*` and other crates.
- [ ] Ensure no required `Vec`/`HashMap` fields are omitted when empty:
  - No `skip_serializing_if = "Vec::is_empty"` / `HashMap::is_empty` on required collections.
  - Add serialization-shape tests for “high fan-out” types (tasks, disciplines, etc.) to lock this down.
- [ ] Decide and standardize 64-bit integer representation on the wire:
  - Today: Rust uses `u64` (serializes as JSON number); TS types generate `bigint`.
  - Make it explicit and consistent across domains (choose one):
    - Use `u32` where possible.
    - Or represent `u64` as string in JSON (recommended for correctness).
- [ ] Regeneration must be the only way types move:
  - `src/types/generated.ts` stays generated only (`just types`), never hand-edited.

## 4. Error Model (Must-Have)
- [ ] Decide whether IPC errors remain `Result<T, String>` or become a structured error envelope.
- [x] Define a standard string error format that is machine-parsable (at minimum: stable error code + message).
  - Current status: `crates/ralph-errors` already provides stable `[R-XXXX] message` formatting plus a parser (`parse_ralph_error`).
- [ ] Enforce the standard string error format across all IPC boundaries (eliminate raw `format!(...)` / `e.to_string()` errors escaping without a code).
- [ ] If moving to structured errors, define one error payload shape and migrate all commands at once (no compat shims).

## 5. Single Transport Adapter Boundary in Frontend (Swap Enabler)
- [x] Ensure *all* frontend command calls go through a single module boundary (`src/lib/tauri/invoke.ts` is the only `@tauri-apps/api/core` import in `src/**`).
- [x] Ensure *all* frontend event subscriptions go through a single module boundary (`src/lib/tauri/events.ts` is the only `@tauri-apps/api/event` import in `src/**`).
- [ ] Ban direct `@tauri-apps/api/*` usage outside that boundary (UI still uses `@tauri-apps/api/window` directly).

## 6. Single “Backend Service” Boundary in Rust (Swap Enabler)
- [ ] Move “real work” out of `#[tauri::command]` functions into a transport-agnostic service layer.
- [x] Define an injected event sink interface:
  - Local Tauri mode: sink emits Tauri events.
  - Remote mode: sink broadcasts WS events (or framed stream).
  - Current state:
    - Contract trait: `crates/ralph-contracts/src/transport.rs` (`EventSink`)
    - Tauri impl: `src-tauri/src/event_sink.rs`
- [x] Define an injected invoke-style RPC client interface (for proxying in remote mode):
  - Contract trait: `crates/ralph-contracts/src/transport.rs` (`RpcClient`)
- [ ] Replace remaining direct Tauri `AppHandle.emit(...)` usage with the sink interface (notably `src-tauri/src/api_server.rs` and the remaining direct emits in `src-tauri/src/commands/terminal_bridge.rs`).
- [ ] Keep Tauri command modules as thin adapters:
  - deserialize args
  - call backend service
  - return result

## 7. Streaming/Terminal Contract Hardening (Must-Have For Headless Parity)
- [x] Keep the current “withhold + replay” model authoritative in the backend (implemented in `src-tauri/src/terminal/manager.rs` + used by `src/lib/terminal/session.ts`).
- [ ] Document reconnection semantics as contract, not just implementation detail:
  - `sessionId` uniqueness rules
  - `seq` monotonicity rules
  - truncation signaling (`truncated`, `truncatedUntilSeq`)
  - replay limits + ordering guarantees
- [ ] Decide single-controller vs multi-attach policy for terminal sessions (v1 likely: hard-fail extra controllers).

## 8. Domain Policy Ownership Audit (Swap Enabler)
- [ ] For every piece of “domain policy” currently in frontend, decide: backend-owned or UI-owned.
  - Use `.docs/067_FRONTEND_LOGIC_BACKEND_AUDIT_CHECKLIST.md` as the working list.
- [ ] Move backend-owned policy behind IPC so all clients observe identical behavior.

## 9. Parity/Drift Test Suite (Must-Have)
- [ ] Add a contract test suite that runs without a GUI:
  - Serialize each IPC type and assert key names exist (camelCase, required collections).
  - Assert command list and event constants match expected snapshots.
- [x] Terminal drift tests exist (backend: `crates/ralph-contracts/src/terminal.rs`; frontend: `src/lib/terminal/terminalBridgeContract.test.ts`).
- [x] Backend diagnostic event drift tests exist (backend: `crates/ralph-contracts/src/events.rs`; frontend: `src/lib/tauri/eventsContract.test.ts`).
- [ ] Extend drift tests to other event domains and other command surfaces.
- [ ] Add a single “contract CI gate”:
  - fails if `just types` produces diffs
  - fails if command list changes without updating the snapshot test

## 10. “Ready For Swap” Definition Of Done
- [ ] Protocol version handshake exists in local IPC and is enforced.
- [ ] Commands/events are inventoried, drift-tested, and type-shared (Rust -> TS) across the board.
- [ ] Error semantics are stable and documented.
- [ ] All frontend IPC usage goes through one boundary module.
- [ ] All backend business logic is transport-agnostic and emits events through an injected sink.
- [ ] Terminal streaming contract is explicitly defined and tested (replay + truncation + reconnection).
