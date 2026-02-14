# Remote/Headless Ralph + IPC Audit (Current Code) + Updated Complexity Rating

Date: 2026-02-14

This doc is a “dump for later” capturing feasibility conclusions, the SSH security model clarification, and a concrete audit of what the repo already has today (so we can rate migration complexity based on real code, not vibes).

Related docs (deeper design dumps):
1. `.docs/073_REMOTE_HEADLESS_RALPH_VIA_SSH.md`
2. `.docs/074_IPC_CONTRACT_REFACTOR_AND_RALPHD_LAYERING_DUMP.md`
3. `.docs/077_IPC_SWAP_READINESS_CHECKLIST.md`

## 1. Current Reality (Why IPC Works Today)
1. Today Ralph4days is a Tauri desktop app:
   1. Frontend (React in WebView) calls Rust via Tauri IPC `invoke(...)`.
   2. Backend registers commands via `#[tauri::command]` and `tauri::generate_handler!` in `src-tauri/src/lib.rs`.
   3. “Server push” is done via Tauri events (`emit` in Rust, `listen` in TS).
2. This IPC exists because the WebView and the Rust host are packaged as one desktop app instance; it is not a remote protocol.
3. The existing frontend assumes “invoke-style request/response” semantics and “event stream” semantics, which is why it feels more like WebSockets than REST.

## 2. Goal (Remote + Headless)
1. Remote access from a local machine to a host running Ralph.
2. A headless server process (`ralphd`) that can run without GUI/WebView and still do:
   1. DB access + project locking.
   2. Task/disciplines/subsystems commands.
   3. Terminal/PTY sessions for agents (Claude/Codex/Shell).
   4. Streaming output + reconnection + replay (terminal tab can close/open without losing output).
3. Keep the existing IPC contract as unchanged as possible for parity.

## 3. SSH “Security Envelope” Clarification
1. SSH is not “one of three random channels”. It is the envelope that can provide:
   1. Authentication (keys, known_hosts, agent).
   2. Encryption.
   3. Tunneling/port forwarding.
2. The app protocol (RPC/streams) rides inside SSH.
3. A common best-practice posture for dev daemons is:
   1. `ralphd` binds to `127.0.0.1` on the remote host (not publicly reachable).
   2. Access happens only via SSH port-forward (or SSH stdio).
4. This means “security” is: if you cannot SSH to the host, you cannot talk to `ralphd` at all.

## 4. “Why 3 Channels?” (SSH vs RPC vs WS)
1. People say “SSH + RPC + WS” because they’re describing layers:
   1. SSH control plane: connect/auth + start server + port-forward.
   2. RPC data plane: request/response calls (invoke-like).
   3. Stream data plane: server push (terminal output, state events).
2. For our use case, the *app layer* can be consolidated:
   1. One WebSocket carrying both:
      1. RPC frames (request/response with ids).
      2. Event frames (server push).
   2. Or one HTTP/2 protocol (gRPC) with unary + streaming.
3. Even if you consolidate the app layer to “one channel”, SSH still exists as the security/tunnel layer unless you deliberately replace it with a different auth/encryption story.
4. There’s also a literal “one channel” option:
   1. `ssh host ralphd --stdio`
   2. Speak a framed protocol over stdin/stdout (no remote port at all).

## 5. Ownership Decision: Who Owns Agent Processes?
1. Model A (recommended): `ralphd` owns agent/PTY processes.
   1. Pros:
      1. Reconnect is possible.
      2. Server can buffer output canonically and serve replay.
      3. One authority for session lifecycle.
   2. Cons:
      1. `ralphd` is “real server software” (process supervision, session persistence).
2. Model B (not recommended for parity): client backend SSH-execs agents directly.
   1. Pros:
      1. `ralphd` can be thinner or non-existent.
   2. Cons:
      1. Disconnect is effectively fatal (reattach is hard without tmux/screen-like tricks).
      2. Output buffering becomes client-owned (more fragile).

## 6. Buffering/Withholding: We Already Implemented This Locally
1. Local mode already supports “terminal tab not open” without losing output:
   1. Backend has `SessionStreamMode` (`Live` vs `Buffered`).
   2. Backend keeps an in-memory replay buffer with seq ids + truncation bookkeeping.
   3. Frontend flips to `buffered` when tab is inactive, then does replay + restore to `live` on activation.
2. Concrete code owners (today):
   1. Canonical invoke arg wrapper shape: `src/lib/tauri/invoke.ts`.
   2. Terminal buffering + replay: `src-tauri/src/terminal/manager.rs`.
   3. Terminal command arg wire types: `src-tauri/src/terminal/contract.rs` + generated TS in `src/types/generated.ts`.
   4. Terminal commands + emits: `src-tauri/src/commands/terminal_bridge.rs`.
   5. Frontend stream-mode orchestration: `src/lib/terminal/session.ts`.
3. Implication for remote:
   1. If we want parity and reconnection, `ralphd` should own the buffering and replay, not the client.

## 7. Audit: Current IPC Contract Quality (As Of This Repo State)
1. Strengths (already reduces remote complexity):
   1. Uniform invoke shape in TS: every command payload is `{ args: ... }` via `src/lib/tauri/invoke.ts`.
   2. Type sharing exists: Rust `#[ipc_type]` macro injects `ts-rs` exports; TS types land in `src/types/generated.ts`.
   3. Terminal bridge is already “remote-friendly” in behavior:
      1. explicit session ids
      2. explicit output seq ids
      3. replay endpoint with truncation signals
      4. stream gating (`live`/`buffered`)
   4. HTTP server is already in the stack: local Axum server in `src-tauri/src/api_server.rs` (currently for MCP signal ingress).
2. Weaknesses (increase remote complexity):
   1. Protocol versioning exists (`crates/ralph-contracts/src/protocol.rs` + `protocol_version_get`), but hard-fail mismatch enforcement is not implemented yet.
   2. Most command implementations are Tauri-bound entrypoints (`#[tauri::command]` functions) rather than a transport-agnostic service layer.
   3. Event emission is Tauri-coupled in several places (for example terminal output uses `AppHandle.emit`), so `ralphd` needs an alternate event sink/broadcast mechanism.
   4. Parity enforcement is uneven:
      1. Terminal has drift tests and shared types.
      2. Other domains do not yet have the same “contract drift detection” rigor.
   5. Some “domain policy” still lives in frontend (see `.docs/067_FRONTEND_LOGIC_BACKEND_AUDIT_CHECKLIST.md`), which becomes a problem when there are multiple clients (local UI, remote UI, CLI, etc.).

## 8. Updated Ratings (Based On Current Code)
1. “IPC contract maturity for remote parity” (1-10, higher is better): **7.5/10**
   1. Why not higher:
      1. protocol versioning exists but mismatch enforcement is not implemented yet
      2. incomplete parity/drift testing outside terminal bridge
      3. business logic still lives inside Tauri command modules
2. “Remote/headless migration complexity” (1-10, higher is harder): **8/10**
   1. Why it stays high even with good IPC types:
      1. remote process supervision + reconnection semantics
      2. server protocol + auth story + versioning
      3. terminal streaming is bidirectional and stateful
      4. SSH install/update/tunnel UX (VS Code-style) is real engineering
3. “How close can it become to a transport-only swap?” (1-10, higher is closer): **5/10 today**
   1. What raises this score:
      1. extract a transport-agnostic backend service layer (shared by Tauri and `ralphd`)
      2. make event emission go through an injected sink interface
      3. add a protocol version + canonical contract registry used by both servers

## 9. Concrete Complexity Reducers (Modularization Path)
1. Extract a transport-agnostic “backend core” (so Tauri and `ralphd` don’t fork logic).
2. Define a single canonical “contracts” owner in Rust for:
   1. command args/results
   2. event payload types
   3. protocol version constant
   4. transport adapter traits (`EventSink`, `RpcClient`)
3. Add parity/drift tests for non-terminal command surfaces (at least: command list + key payload shapes).
4. Move frontend-owned domain policy to backend (prompt registry, launch resolution policy, naming invariants).
5. Only then: build `ralphd` + SSH connection manager (install/update/tunnel).

## 10. Migration Checklist (Merged)
This section was merged from the now-removed `.docs/076_RALPHD_REMOTE_HEADLESS_MIGRATION_CHECKLIST.md` to keep a single canonical owner.

Goal: ship **headless remote Ralph** while keeping the existing frontend-facing IPC contract as unchanged as practical (parity), with SSH as the default security envelope.

Non-goal: redesign the UI or rewrite the app around a web backend.

### Phase 0: Scope Freeze + Inventory
1. [ ] Freeze the “must support” command list for parity (source of truth: `src-tauri/src/lib.rs`).
2. [ ] Freeze the “must support” event list for parity (source of truth: terminal events + any other emitted events).
3. [ ] Freeze wire shape rules (camelCase, `{ args: ... }` invoke wrapper, error envelope policy).
4. [ ] Decide v1 remote platform support set (recommended: Linux x86_64 only for first ship).
5. [ ] Decide v1 multi-attach policy (recommended: single controller per terminal session; hard-fail additional controllers).

### Phase 1: Contract Hardening (Before Any Ralphd Code)
1. [x] Introduce explicit `PROTOCOL_VERSION` and a local version payload (hard-fail on mismatch in remote mode).
2. [ ] Ensure all IPC-exported structs serialize required collections even when empty (no omitted required `Vec`/`HashMap` keys).
3. [ ] Add contract drift tests for non-terminal domains:
   1. [ ] command list parity test (one canonical registry, used by both servers)
   2. [ ] key payload shape tests (serde roundtrip / snapshot JSON keys)
4. [ ] Decide app-layer framing for remote:
   1. [ ] Option A (recommended): single WebSocket multiplexing RPC + events
   2. [ ] Option B: HTTP JSON RPC + separate WS for events/terminal
   3. [ ] Option C: SSH stdio framed protocol (`ssh host ralphd --stdio`)

### Phase 2: Extract A Transport-Agnostic Backend Core
1. [ ] Move “real work” out of `#[tauri::command]` handlers into a Tauri-free crate (for example `crates/ralph-backend/`).
2. [x] Replace direct `tauri::AppHandle.emit(...)` usage with an injected event sink trait (partial adoption; remaining emits still exist).
3. [ ] Keep `src-tauri/src/commands/*` as thin adapters:
   1. [ ] local mode: call backend core directly
   2. [ ] remote mode: forward to remote client without translation
4. [ ] Move frontend-owned domain policy into backend where it impacts correctness across clients (see `.docs/067_FRONTEND_LOGIC_BACKEND_AUDIT_CHECKLIST.md`).

### Phase 3: Stand Up Ralphd Locally (No SSH Yet)
1. [ ] Add a headless binary `ralphd` (Rust).
2. [ ] Bind to `127.0.0.1` by default (no public listener).
3. [ ] Implement version endpoints:
   1. [ ] `GET /health`
   2. [ ] `GET /version` (includes `PROTOCOL_VERSION`, server build/version, capabilities)
4. [ ] Implement RPC surface mirroring current invoke commands:
   1. [ ] same command names
   2. [ ] same request/response JSON shapes (including `{ args: ... }` if we keep that)
   3. [ ] unknown command is an error (no silent drop)
5. [ ] Implement event streaming:
   1. [ ] terminal output/closed streams
   2. [ ] any other domain events needed for UI parity
6. [ ] Port the existing terminal replay/buffering semantics into `ralphd`:
   1. [ ] session ids
   2. [ ] seq ids
   3. [ ] truncation reporting
   4. [ ] `live` vs `buffered` gating

### Phase 4: Build The Remote Client/Proxy Layer (Keep UI Untouched)
1. [ ] Add a “remote transport adapter” in the local Tauri backend:
   1. [ ] in remote mode, `#[tauri::command]` handlers forward to `ralphd` over the chosen transport
   2. [ ] in remote mode, subscribe to `ralphd` event stream and re-emit as Tauri events with identical names/payloads
2. [ ] Ensure reconnection semantics:
   1. [ ] on reconnect, client re-subscribes
   2. [ ] terminal tab activation triggers replay from last seen seq
3. [ ] Hard-fail on protocol mismatch with a deterministic remediation message (“update ralphd”).

### Phase 5: SSH Envelope (VS Code Remote-SSH Model)
1. [ ] Add SSH connect flow (local machine):
   1. [ ] host selection + identity selection
   2. [ ] known_hosts behavior (hard-fail or explicit trust prompt; no silent trust)
2. [ ] Add remote install/update flow for `ralphd`:
   1. [ ] detect remote OS/arch
   2. [ ] fetch/upload correct artifact
   3. [ ] checksum verify (SHA256)
   4. [ ] versioned install dir + atomic `current` switch
3. [ ] Add service management (preferred: `systemd --user`):
   1. [ ] install unit
   2. [ ] enable/start
   3. [ ] hard-fail with explicit requirements if unsupported (no `nohup` fallback)
4. [ ] Add tunnel management:
   1. [ ] establish local port forward to remote `127.0.0.1:<ralphdPort>`
   2. [ ] monitor tunnel liveness
   3. [ ] teardown on disconnect

### Phase 6: Parity Gates + Testing
1. [ ] Add a parity test harness that runs against:
   1. [ ] local Tauri backend (current)
   2. [ ] `ralphd` direct
   3. [ ] Tauri proxy -> `ralphd` (remote mode)
2. [ ] Terminal parity tests:
   1. [ ] output frames identical rendering
   2. [ ] resize/input works
   3. [ ] buffering/replay works across disconnect/reconnect
3. [ ] Failure posture tests:
   1. [ ] unknown command/event hard-fails
   2. [ ] protocol mismatch hard-fails with upgrade path
   3. [ ] missing capability hard-fails (capabilities reported by `/version`)

### Phase 7: Operational Hardening (v1)
1. [ ] Audit for secrets in logs (especially SSH command stderr/stdout capture).
2. [ ] Add bounded memory policy for server-side buffers (terminal replay buffer already has truncation; ensure all streams have limits).
3. [ ] Add explicit timeouts/retries policy for remote calls (no infinite hang).
4. [ ] Add “capabilities” reporting so the GUI can explain why something is unavailable (but still fail loudly when required).

### Definition Of Done (v1 Remote/Headless)
1. [ ] A local GUI can connect to a remote host via SSH and use Ralph normally.
2. [ ] The remote host runs `ralphd` headless and owns agent processes.
3. [ ] Terminal sessions survive GUI disconnect and can be replayed after reconnect.
4. [ ] Protocol mismatch results in a deterministic update flow (or explicit instructions).
5. [ ] The React UI does not need transport-specific code paths (Tauri proxy re-emits events + keeps invoke API stable).
