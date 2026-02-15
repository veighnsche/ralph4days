# Tauri Mobile (iOS + Android) Enablement Dump

Date: 2026-02-14
Updated: 2026-02-15

## 0. Direct Answer (With Our Current Architecture)
Yes, we can ship iOS and Android apps from this codebase because we are already on **Tauri 2** with a **React + TypeScript** frontend and a **Rust** backend (`tauri` 2.x in `src-tauri/Cargo.toml`, `@tauri-apps/api` 2.x in `package.json`).

However, the *current product behavior* is **desktop-host dependent**:
- Terminal tabs are backed by PTYs + subprocesses (`portable-pty`, external CLIs).
- Project discovery/locking assumes access to an arbitrary desktop filesystem (`project_scan`, "open recent projects", etc.).
- Multi-window assumptions exist (main + splash, "open new window" spawns a new process).

Those assumptions do not hold on mobile (especially iOS). So "mobile app" is feasible, but **what it does** must be decided up front.

## 1. Decision (As Of 2026-02-15): Remote-Only Mobile Client
Mobile builds are "remote mode only":
- The iOS/Android app is a UI client.
- A headless daemon (`ralphd`) owns the project dir, SQLite DB, PTYs/subprocesses, and agent execution.
- The mobile app never runs the orchestrator locally.

Constraints:
- `ralphd` is **Linux-only for now** (server/runtime target constraint; independent of where we build the iOS app).
- Transport is **private-network + SSH**:
  - Tailscale/VPN gets the phone onto the same private network.
  - SSH is the security envelope (auth + encryption).
  - The app protocol rides inside SSH (tunnel + RPC + event streaming, or SSH stdio framing).
- Mobile `src-tauri` is a **proxy-only backend**:
  - It exists to host the WebView and provide IPC to the React UI.
  - It forwards the existing `invoke(...)` command surface to `ralphd` (no local "real work").
  - It re-emits remote events as local Tauri events so the UI stays transport-agnostic.

Failure posture (mandatory):
- No connection means **every invoke fails loudly** with a deterministic "not connected" error.
- Protocol/version mismatch is a hard error with a deterministic remediation path.
- No "offline mode" that pretends work is happening.

Canonical references in this repo (do not duplicate their contents here):
- `.docs/073_REMOTE_HEADLESS_RALPH_VIA_SSH.md` (overall architecture + posture)
- `.docs/074_IPC_CONTRACT_REFACTOR_AND_RALPHD_LAYERING_DUMP.md` (contract rules + channel layering options)
- `.docs/075_REMOTE_HEADLESS_RALPH_IPC_AUDIT.md` (command/event surface audit)

Non-goal:
- "On-device runner" mode is explicitly out of scope for this direction.

## 2. Platform Tooling Requirements
### 2.1 Android (Can Be Done On Linux)
Prereqs (developer machine / CI):
- Android Studio (SDK + emulator)
- Android SDK + NDK configured and discoverable by Tauri tooling
- JDK (typically installed with Android Studio)
- Rust Android targets (exact targets depend on what ABIs we ship)

Tauri CLI entrypoints (from this repo's toolchain):
- `bunx tauri android init` (generates Android project scaffolding)
- `bunx tauri android dev` (dev on emulator/device)
- `bunx tauri android build` (APK/AAB for release)

### 2.2 iOS (Requires A macOS Build Host)
iOS builds require:
- macOS + Xcode toolchain
- Apple Developer Program membership for device testing and App Store distribution
- Code signing identities and provisioning profiles

Important practical note:
- Tauri's iOS CLI subcommands are **macOS-only**. On Linux you should expect `tauri ios ...` to be unavailable.

## 3. Code/Config Work Needed (To Compile + Run On Mobile)
This is the "what must change in *our* repo" list.

### 3.1 Tauri Entry Point Must Be Split (Desktop vs Mobile)
Current `src-tauri/src/lib.rs` assumes desktop:
- creates multiple windows (`main`, `splash`)
- reads CLI args (project lock, `--no-splash`)
- spawns a new app process for `window_open_new`
- applies Linux-only WebKitGTK env var workaround

For mobile we need a `cfg(mobile)` path that:
- creates the single mobile webview (no splash window)
- does not parse CLI args
- does not spawn a new process

This must be done without adding silent fallbacks. Unsupported actions should be explicit errors.

### 3.2 Backend Diet: Mobile Must Not Link Desktop-Host Dependencies
In remote-only mobile mode, these must not exist as "local implementations":
- PTY terminal subsystem (`src-tauri/src/terminal/*`, `commands/terminal_bridge.rs` local PTY ownership)
- local SQLite project DB ownership (`sqlite-db` usage as canonical state)
- local prompt-builder ownership (`prompt-builder` usage as canonical state)
- local agent execution / external process spawning
- desktop window/process behaviors (`window_open_new`, splash window, CLI args)
- local Axum server used for MCP ingress (belongs in `ralphd`, not the mobile proxy)

Instead:
- The mobile backend proxies these semantics to the remote Linux `ralphd`.
- If the proxy is not connected, it fails loudly (see Section 1).

Important: this is a build-time diet, not a runtime feature flag.
- Mobile builds should not compile/link the desktop-host crates and subsystems at all.

Implication for the React UI:
- Any UI action that implies "pick a local directory" or "scan local disk" must be removed/hidden on mobile, or it will hard-fail by design.

### 3.3 Capabilities/Permissions Need Mobile-Specific Definitions
Current capability file (`src-tauri/capabilities/default.json`) is desktop-scoped:
- schema is `desktop-schema.json`
- window list includes `main` + `splash`

For mobile we need:
- capability file(s) using the correct mobile schema
- a window list that matches the mobile windowing model (likely a single window)
- explicit permissions for whatever plugins/features we keep on mobile (network, opener/dialog, etc.)

### 3.4 Storage Model Must Be Explicit
Remote-only decision:
- canonical DB/project state is remote (Linux `ralphd`)
- local storage is limited to connection profiles (host, user, key reference, known-hosts pin, last connected)

The code must have one canonical owner for "what remote am I connected to" and "what remote project is locked", with hard-fail behavior on ambiguity.

### 3.5 Networking (Remote Mode)
Remote-only mobile requires:
- a remote transport for "invoke-like" RPC calls and event streams
- explicit auth and encryption (SSH, reached over Tailscale/VPN)
- strict version/protocol gating (no "best effort")
- a single transport adapter boundary (UI must not contain ad-hoc SSH/RPC logic)

Do not bolt this into the UI in an ad-hoc way. Keep one transport adapter boundary and keep contracts canonical in `crates/ralph-contracts`.

Current repo state (already implemented):
- Remote transport today is a single WebSocket carrying `RemoteWireFrame` (RPC + events) and is already implemented in the local Tauri backend (`src-tauri/src/remote.rs`) and in the headless server (`src-daemon/src/main.rs`).
- The Tauri proxy connect surface currently takes a `wsUrl` directly (`remote_connect`). SSH port-forwarding / tunnel management is intentionally not part of that layer yet; mobile will need a "connection manager" that establishes SSH (over Tailscale/VPN) and then supplies a local `ws://127.0.0.1:<forwardedPort>` to `remote_connect`.

## 4. Side-Quest Development Already In Repo (Remote Proxy + Ralphd)
This section exists to prevent re-work. The following modules/crates were introduced specifically to make "remote-only mobile" viable.

Contracts and protocol (Rust canonical owner):
- `crates/ralph-contracts/src/protocol.rs`: `PROTOCOL_VERSION` + `ProtocolVersionInfo` (typed, deny-unknown-fields).
- `crates/ralph-contracts/src/terminal.rs`: terminal event payloads + canonical event name constants (`terminal:output`, `terminal:closed`) with drift tests.
- `crates/ralph-contracts/src/events.rs`: non-terminal event payloads + canonical event name constants (for example `backend-diagnostic`) with drift tests.
- `crates/ralph-contracts/src/transport.rs`: transport traits (`EventSink`, `RpcClient`) and the one-channel WS framing contract (`RemoteWireFrame`, `RemoteEventFrame`).

Backend extraction (shared by Tauri + ralphd):
- `crates/ralph-backend/src/{project.rs,session.rs,tasks.rs,...}`: Tauri-free domain logic (the long-term intent is: Tauri command modules become thin adapters).

Headless Linux server (remote authority):
- `src-daemon/src/main.rs`: WebSocket server that speaks `RemoteWireFrame` and currently implements a parity subset:
  - `protocol_version_get`
  - `project_validate_path`
  - `project_lock_{set,get}`
  - `tasks_*` (CRUD + signals/comments subset)
- Status: terminal/PTY parity and broader command parity are not complete yet.

Desktop proxy plumbing (what mobile will reuse, then "diet down"):
- `src-tauri/src/remote.rs`: `RemoteWireFrameConnection` WS client, protocol handshake (hard-fail on mismatch), event pump into an injected sink, invoke-style RPC client.
- `src-tauri/src/commands/remote.rs`: `remote_connect`, `remote_disconnect`, `remote_status_get`.
- `src-tauri/src/commands/remote_proxy.rs`: helpers that enforce canonical invoke payload shape (`{ args: ... }`) when proxying.
- `src-tauri/src/event_sink.rs`: `TauriEventSink` implements `EventSink` by re-emitting remote events as local Tauri events.
- `src-tauri/tests/invoke_command_list_contract_test.rs`: drift test that hard-fails if the invoke command list changes unintentionally.

Frontend IPC boundaries (swap enablers):
- `src/lib/tauri/invoke.ts`: the single frontend boundary for `invoke(...)` calls; enforces the `{ args: ... }` envelope.
- `src/lib/tauri/events.ts`: the single frontend boundary for `listen(...)` subscriptions.
- `src/lib/tauri/eventsContract.test.ts` + `src/lib/terminal/terminalBridgeContract.test.ts`: drift tests for event name constants the UI listens to.

UI work that landed (not required for the backend side-quest):
- A "mobile-first responsive shell + workspace toggle" exists in the frontend history. This can help iOS/Android ergonomics, but is not the core requirement for remote-only proxy mode.

Git breadcrumbs (so we can find this work in history later; as of 2026-02-15):
- `feat(contracts): introduce ralph-contracts crate`
- `refactor(backend): route terminal + diagnostics events via EventSink`
- `refactor(frontend): centralize Tauri event subscriptions`
- `docs(remote): add headless ralph IPC audit + readiness checklist`
- `docs(mobile): add Tauri mobile enablement dump`
- `feat(remote): add RemoteWireFrame remote-mode adapter`
- `feat(remote): proxy terminal bridge commands in remote mode`
- `feat(remote): proxy core command surfaces in remote mode`
- `feat(ralphd): add RemoteWireFrame ws skeleton`
- `refactor(backend): extract project lock + tasks domains`
- `feat(ralphd): serve project lock + tasks via ralph-backend`
- `docs(mobile): note remote-only proxy approach`

## 5. Distribution Requirements (High-Level)
Android:
- debug: emulator/device installs (APK)
- release: Play Store (AAB), signing keys, Play Console configuration

iOS:
- signing + provisioning
- TestFlight/App Store Connect distribution
- macOS runner / Mac build host as a hard dependency

## 6. Testing/Verification Requirements
Minimum gates to claim "mobile support exists":
- A compile gate for mobile targets (Android at least; iOS on macOS CI/host).
- Emulator smoke run: app boots, loads the UI, and can perform a representative workflow.
- Contract tests that prove mobile can speak to `ralphd` and handle version mismatch as a hard error.
- Tests that assert "disconnected proxy" hard-fails (no silent no-ops).

## 7. Explicitly Out Of Scope (Per Request)
- GUI responsiveness and small-screen layout work (beyond whatever is already landed).
- UX re-design for touch, keyboard avoidance, and compact navigation.
