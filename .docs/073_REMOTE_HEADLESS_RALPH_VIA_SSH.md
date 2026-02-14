# Remote Headless Ralph via SSH (VS Code Remote-SSH Model)

Date: 2026-02-14

## 1. Problem Statement
1. We need **remote access** to Ralph running on a different machine (typically on the same local network, but SSH reachable).
2. We need a **headless Ralph** that can run without a GUI/WebView and still provide full backend functionality.
3. The **Tauri GUI is the user interface** and should not be redesigned. The goal is to preserve workflows while swapping transport from local IPC to remote RPC.

## 2. Constraints / Non-Goals
1. Tauri `invoke` IPC is a local bridge between the WebView and the Rust host; it is **not** a remote transport.
2. We will not expose the headless server publicly by default. Default posture is **bind to localhost** on the remote host and rely on SSH for auth + encryption.
3. We will not add “best-effort” silent fallbacks. If a remote invariant is broken (version mismatch, protocol mismatch, missing capability), it fails loudly with remediation steps.

## 3. Feasibility Summary
1. This is feasible and aligns with the proven pattern used by VS Code Remote-SSH:
   1. A local GUI manages an SSH connection.
   2. It installs a server component on the remote host (first connect) and keeps it up to date.
   3. It establishes tunnels and talks to the server over a stable protocol.
2. “Headless” does not mean “CLI only”. It means **no windowing/WebView requirement** on the machine that executes tasks.

## 4. Why Not “Just Use IPC”
1. Current frontend/backend communication is implemented via:
   1. Frontend: `@tauri-apps/api/core` `invoke(...)`.
   2. Backend: `#[tauri::command]` + `.invoke_handler(...)`.
   3. Push/streaming: Tauri events (`emit` in Rust, `listen` in TS).
2. This works because the UI runs in an embedded WebView controlled by the same desktop app instance. It does not generalize to remote execution.

## 5. “SSH vs mosh vs modern options”
1. `mosh` is excellent for **interactive terminal sessions** over unreliable links, but it is not a general-purpose secure RPC + port-forwarding substrate.
2. The practical replacements for “SSH as the security envelope” are:
   1. A mesh VPN (Tailscale/WireGuard/ZeroTier) and then plain TCP to a private address.
   2. A tunnel broker (Cloudflare Tunnel, etc.) with explicit auth.
3. For v1, SSH is the best default because it is ubiquitous and already provides:
   1. Authentication (keys, agent forwarding, known_hosts).
   2. Encryption.
   3. Port forwarding.

## 6. Proposed Architecture (Decision-Complete Shape)
1. Introduce a new headless daemon: `ralphd`.
   1. Runs on the remote host (the machine that executes tasks).
   2. Owns subprocess/PTY management, DB access, task execution sequencing, and event production.
2. Introduce a stable remote protocol:
   1. HTTP JSON for request/response commands (invoke-like).
   2. WebSocket for push events and terminal output streaming (event-like).
3. Keep the Tauri GUI as-is, but add a “transport adapter” boundary:
   1. Local mode: current Tauri IPC.
   2. Remote mode: HTTP + WebSocket to `ralphd` over an SSH tunnel.

## 7. Transport Model (VS Code-Style)
1. On the remote host, `ralphd` binds to `127.0.0.1:<port>` (localhost only).
2. The GUI establishes an SSH local port forward:
   1. `ssh -N -L <localPort>:127.0.0.1:<remotePort> user@host`
3. The GUI talks to:
   1. `http://127.0.0.1:<localPort>` (HTTP RPC)
   2. `ws://127.0.0.1:<localPort>` (WS streams)

## 8. Remote Install/Update Flow (First Connect)
1. The GUI provides “Connect via SSH”.
2. The local app runs a deterministic install script over SSH:
   1. Detect remote OS/arch: `uname -s`, `uname -m`.
   2. Resolve the correct `ralphd` artifact for that platform.
   3. Download or upload the artifact.
   4. Verify checksum (SHA256).
   5. Install into a versioned directory and flip an atomic `current` symlink.
3. Service management:
   1. Install a `systemd --user` unit (preferred baseline).
   2. Enable and start `ralphd`.
   3. If `systemd --user` is not available, fail with explicit requirements rather than silently doing `nohup`.

## 9. Versioning and Compatibility Rules
1. Define an explicit `protocol_version`.
2. The GUI must hard-fail if:
   1. `protocol_version` mismatches.
   2. The remote server reports “unsupported platform” or missing capabilities.
3. Auto-update path is explicit:
   1. If mismatch detected, run the install/update flow to replace `ralphd`.

## 10. Protocol Surface (What Needs to Exist Remotely)
1. HTTP endpoints:
   1. `GET /health` for liveness.
   2. `GET /version` returns server + protocol info.
   3. `POST /rpc/<command>` for invoke-equivalents. Unknown commands are errors.
2. WebSocket:
   1. `GET /ws/events` for push events that currently use Tauri events (diagnostics, task changes, terminal output/closed, etc.).
      1. Recommendation: multiplex *all* server-push events (including terminal) onto this one WS using `RemoteEventFrame`.
   2. `GET /ws/terminal` is optional (only if we later decide to split terminal streaming onto a separate WS):
      1. stdout/stderr frames
      2. exit status
      3. resize
      4. stdin input
3. Event framing contract (receive-side):
   1. Each WS message is one JSON object with the shape:
      1. `{ "event": "<event-name>", "payload": { ... } }`
   2. Canonical Rust owner:
      1. `crates/ralph-contracts/src/transport.rs` (`RemoteEventFrame`)
   3. Policy:
      1. Unknown `event` values are protocol errors (fail loudly, no silent drops).
      2. Payloads are strict-decoded (`deny_unknown_fields`) to catch protocol drift early.
4. Optional one-channel framing contract (RPC + events over one WS):
   1. Each WS message is one JSON object with the shape:
      1. `{ "type": "rpc-request", "id": 1, "command": "<invoke-command>", "payload": { ... } }`
      2. `{ "type": "rpc-ok", "id": 1, "result": { ... } }`
      3. `{ "type": "rpc-err", "id": 1, "error": "[R-XXXX] ..." }`
      4. `{ "type": "event", "event": "<event-name>", "payload": { ... } }`
   2. Canonical Rust owner:
      1. `crates/ralph-contracts/src/transport.rs` (`RemoteWireFrame`)

## 11. Refactoring Strategy (So We Don’t Fork Logic)
1. Extract backend “real work” from Tauri command handlers into a Tauri-agnostic Rust crate (shared library).
2. Replace direct Tauri event emission calls with an injected interface:
   1. Local Tauri implementation emits via `AppHandle.emit(...)`.
   2. `ralphd` implementation broadcasts via WebSocket.
3. Goal: one canonical owner for task execution logic, terminal manager, and DB operations.

## 12. Security Posture (v1)
1. Authentication:
   1. Use SSH identity (keys) as the primary authentication.
   2. Bind `ralphd` to localhost so it is not reachable except through SSH forwarding.
2. Data exposure:
   1. Events and RPC responses must be scoped to the locked project/session, same as current “one project per session” model.
3. Logs:
   1. Failures include enough context to diagnose (remote command, stderr, exit code), but no secrets in logs.

## 13. Testing and Acceptance Criteria
1. Contract tests:
   1. For each existing `invoke` command, there is a remote RPC equivalent with the same schema.
   2. Failure schemas are stable and machine-parsable.
2. Terminal streaming:
   1. Output frames render identically in the GUI as local mode.
   2. Resize/input behavior matches local mode.
3. Upgrade/mismatch:
   1. Protocol mismatch triggers a clear error and a deterministic upgrade path.
4. Hard-fail invariants:
   1. Unknown commands/events are explicit errors (no silent drops).

## 14. Related Existing Code (Current State)
1. Tauri command surface is wired in `src-tauri/src/lib.rs` via `.invoke_handler(...)`.
2. There is already a local Axum HTTP server in `src-tauri/src/api_server.rs` (currently bound to `127.0.0.1:0`) used for MCP signal ingress.
   1. This is not the remote protocol yet, but it shows Axum is already in the stack.

## 15. Terminal Streaming Ownership (Withholding + Replay)
1. Current behavior (local mode) already supports “withhold stream when terminal tab is inactive”:
   1. Backend keeps an in-memory replay buffer per terminal session and appends every output chunk to it.
   2. Backend only emits live output events when the session stream mode is `Live`.
   3. When the session stream mode is `Buffered`, output is still collected, but not pushed to the frontend (it must be replayed).
2. This is implemented in:
   1. Backend buffering + stream mode: `src-tauri/src/terminal/manager.rs` (`SessionStreamMode`, replay buffer, seq, truncation).
   2. Frontend runtime mode switching: `src/lib/terminal/session.ts` (set `buffered` when inactive; on activation do `buffered -> replay -> live -> replay`).
3. Ownership decision for remote mode:
   1. `ralphd` should own the terminal process lifecycle (PTY/subprocess) AND the replay buffer and stream-withholding logic.
   2. The GUI (or a thin local “remote backend proxy”) should remain dumb: it requests stream mode changes and replays output, but does not maintain the canonical output buffer.
4. Why `ralphd` should own withholding/buffering:
   1. It avoids duplicating an already-complex streaming subsystem in the client.
   2. It preserves the key headless/remote property: reconnect is possible. If the GUI disconnects, `ralphd` continues running and retains the replay buffer so the GUI can resubscribe and replay by `session_id` + `after_seq`.
5. Caveat for future multi-attach:
   1. The current model is effectively “one subscriber controlling stream mode per session”. If multiple GUIs attach to the same terminal session, a global `Live/Buffered` flag becomes contentious.
   2. If multi-attach is required later, evolve stream gating to be per-subscriber while keeping one canonical buffer in `ralphd`, or hard-fail additional controllers in v1.

## 16. Migration Complexity Score
1. Complexity score (1-10): **8/10 (high)** for “full parity headless `ralphd` + VS Code-style SSH install/update + streaming parity + reconnect”.
2. Why it’s high:
   1. Backend decoupling: extracting “real work” out of Tauri `#[tauri::command]` handlers into shared crates and replacing Tauri-only event emission with an injected sink.
   2. Streaming parity: terminal sessions are bidirectional and stateful (session IDs, replay buffers, truncation, mode switching, resize/input). Getting reconnection semantics right is non-trivial.
   3. Remote lifecycle: platform detection, artifact distribution, checksum verification, remote service installation, and version mismatch handling (hard-fail + deterministic remediation).
   4. Cross-platform surface: remote host OS/arch may not match the GUI machine, so the install story must select the correct binary and fail loudly if unsupported.
3. What would lower the score:
   1. Reduce scope to “tasks + execution controls + events” but postpone “full terminal interactivity” (drops to ~6/10).
   2. Require a single supported remote platform for v1 (for example Linux x86_64 only) (drops ~1 point).
   3. Skip auto-install and require manual `ralphd` install + service setup (drops ~1-2 points, but worsens UX).

## 17. IPC Parity Contract (Keep Frontend IPC Untouched)
1. Goal: keep the **existing Tauri IPC contract** as close to unchanged as possible:
   1. Same command names (`invoke('...')`).
   2. Same request/response JSON shapes.
   3. Same event names + payload schemas (for terminal output, diagnostics, task updates, etc.).
2. Strategy: in “remote mode”, treat the local Tauri backend as a **thin proxy**:
   1. `#[tauri::command]` handlers remain the frontend-facing API.
   2. In local mode, handlers call the in-process backend implementation (current behavior).
   3. In remote mode, handlers forward the same request payloads to `ralphd` (minimal/no translation) and return the remote response as-is.
3. Events in remote mode:
   1. Local Tauri backend subscribes to `ralphd` streams (WS).
   2. It decodes WS frames as `RemoteEventFrame` (Rust) then re-emits them as Tauri events with identical names/payloads, so the React UI does not need transport-specific logic.
4. DRY contract ownership:
   1. Define the canonical command arg/result structs and event payload structs once in Rust (“wire contract” types).
   2. Derive `serde` + `ts-rs` so `src/types/generated.ts` stays the single frontend owner of wire types.
   3. Both Tauri handlers and `ralphd` handlers use the same Rust types to ensure compile-time parity.
5. Parity enforcement:
   1. Maintain a single canonical command list used to register both Tauri invoke handlers and `ralphd` routes.
   2. Add a test/CI gate that fails if a new command exists on one side but not the other.

## 18. Complexity Reduction Strategy (Modularize First, Then Ship Remote)
1. Principle: the overall migration is inherently high-complexity, but we can reduce risk and cognitive load by making each unit **small, testable, and transport-agnostic**.
2. Module decomposition (recommended):
   1. `crates/ralph-contracts/` (new)
      1. Owns the canonical “wire contract” types: command args/results and event payload structs/enums.
      2. Owns `PROTOCOL_VERSION` (explicit constant).
      3. Derives `serde` + `ts-rs` so `just types` continues to generate `src/types/generated.ts` from the same canonical owner.
   2. `crates/ralph-backend/` (new)
      1. Owns the actual backend behavior currently living behind `#[tauri::command]` modules:
         1. task CRUD and execution controls
         2. project locking/validation
         3. terminal session lifecycle (PTY manager)
         4. prompt builder integration
      2. Has no dependency on Tauri.
      3. Emits events through a trait (see below), never directly via `tauri::AppHandle`.
   3. `src-tauri/src/commands/*` (existing, becomes thin adapters)
      1. Keeps the frontend IPC API stable.
      2. Delegates to `ralph-backend` in local mode.
      3. Later: delegates to a remote client in remote mode (but the UI never sees this).
   4. `crates/ralphd/` or `src-tauri/src/bin/ralphd.rs` (new headless server)
      1. HTTP/WS server that exposes the same `ralph-contracts` API.
      2. Calls `ralph-backend` for the real work.
   5. `crates/ralph-remote/` (optional, later)
      1. Owns SSH install/update, tunnel management, and reconnection policy.
      2. Used by the local Tauri backend to “connect remote”, but keeps SSH concerns out of the GUI and out of `ralph-backend`.
3. Core interface to enable transport-agnostic backend:
   1. Define an event sink trait in `ralph-backend` (or `ralph-contracts`) for domain events.
   2. Implementations:
      1. Tauri adapter emits Tauri events.
      2. `ralphd` adapter broadcasts via WS.
   3. Hard-fail rule: if an internal event cannot be delivered due to an invariant violation, treat that as a bug (do not silently drop).
4. Shipping phases (each phase keeps the app working; reduces “8/10 all-at-once”):
   1. Phase 0: Contract freeze
      1. Identify the command/event surface area that must be supported by `ralphd`.
      2. Move the corresponding structs into `ralph-contracts`.
   2. Phase 1: Extract backend core
      1. Move logic out of `src-tauri/src/commands/*` into `ralph-backend`.
      2. Keep Tauri command names and TS usage unchanged.
      3. Add deterministic Rust tests for core behavior (this is where correctness hardens).
   3. Phase 2: Make terminal streaming transport-agnostic
      1. Replace direct Tauri emits in the terminal subsystem with the event sink trait.
      2. Preserve existing buffering/withholding/replay semantics (see Section 15).
   4. Phase 3: Stand up `ralphd` locally (no SSH yet)
      1. Implement `/health`, `/version`, minimal RPC + WS.
      2. Add contract tests that run the GUI against a local `ralphd` (or run a thin Rust client against `ralphd`) to enforce parity.
   5. Phase 4: Add SSH install + tunnel + remote mode toggle
      1. All SSH concerns live in one place (`ralph-remote` or a small module in `src-tauri`), not scattered across commands.
      2. Once connected, local Tauri backend simply proxies the existing IPC API to `ralphd`.
5. Definition of done for “modularization complete” (before investing in SSH UX):
   1. Tauri commands are thin adapters (no duplicated business logic).
   2. Terminal manager + execution engine do not depend on Tauri types.
   3. `src/types/generated.ts` is generated from the same Rust contract types used by both local and `ralphd` implementations.
