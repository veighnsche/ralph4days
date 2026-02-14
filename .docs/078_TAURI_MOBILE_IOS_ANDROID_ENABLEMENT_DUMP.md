# Tauri Mobile (iOS + Android) Enablement Dump

Date: 2026-02-14

## 0. Direct Answer (With Our Current Architecture)
Yes, we can ship iOS and Android apps from this codebase because we are already on **Tauri 2** with a **React + TypeScript** frontend and a **Rust** backend (`tauri` 2.x in `src-tauri/Cargo.toml`, `@tauri-apps/api` 2.x in `package.json`).

However, the *current product behavior* is **desktop-host dependent**:
- Terminal tabs are backed by PTYs + subprocesses (`portable-pty`, external CLIs).
- Project discovery/locking assumes access to an arbitrary desktop filesystem (`project_scan`, "open recent projects", etc.).
- Multi-window assumptions exist (main + splash, "open new window" spawns a new process).

Those assumptions do not hold on mobile (especially iOS). So "mobile app" is feasible, but **what it does** must be decided up front.

## 1. Non-Negotiable Product Decision (Pick One)
### Option A: Mobile As A Remote Client (Recommended)
Ship iOS/Android as a UI client that connects to a **headless backend** running on a real dev machine/server that actually has:
- the project directory,
- the ability to spawn PTYs/subprocesses,
- Claude CLI (or whichever agent runtime we target),
- stable storage for the SQLite DB.

This preserves the "Ralph as orchestrator on a workstation" model and avoids trying to cram a dev environment into a phone.

Canonical references in this repo (do not duplicate their contents here):
- `.docs/073_REMOTE_HEADLESS_RALPH_VIA_SSH.md` (overall architecture + posture)
- `.docs/074_IPC_CONTRACT_REFACTOR_AND_RALPHD_LAYERING_DUMP.md`
- `.docs/075_REMOTE_HEADLESS_RALPH_IPC_AUDIT.md`

**Fail-fast rule for remote mode:** protocol mismatch, missing capability, or auth failures must be explicit hard errors with deterministic remediation. No silent "offline mode" that pretends work is happening.

### Option B: On-Device Runner (Not Recommended For Current Feature Set)
On-device can only support the subset that is truly mobile-compatible:
- local CRUD over an on-device SQLite DB (app-private storage),
- prompt preview/editing (pure prompt-builder logic),
- note-taking / task list UI.

It cannot provide "run task in a terminal with Claude CLI" in a way that matches the desktop product.

**Fail-fast rule for on-device mode:** any desktop-only command (PTY/terminal, project scanning, "open new window", etc.) must fail loudly on mobile with an error like "not supported on mobile" and a pointer to the remote-client path.

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

### 3.2 Desktop-Only Backend Surfaces Must Be Isolated
At minimum, the following must be treated as "desktop-only unless remote-backed":
- PTY terminal subsystem (`src-tauri/src/terminal/*`, `commands/terminal_bridge.rs`)
- "open new window" (`window_open_new` uses `std::process::Command`)
- project scanning / arbitrary path locking (`project_scan`, `project_lock_*`)

Mobile-compatible implementation choices:
- Remote-client approach: keep the frontend contract stable and proxy these commands to `ralphd`.
- On-device approach: reject these commands on mobile with explicit errors.

### 3.3 Capabilities/Permissions Need Mobile-Specific Definitions
Current capability file (`src-tauri/capabilities/default.json`) is desktop-scoped:
- schema is `desktop-schema.json`
- window list includes `main` + `splash`

For mobile we need:
- capability file(s) using the correct mobile schema
- a window list that matches the mobile windowing model (likely a single window)
- explicit permissions for whatever plugins/features we keep on mobile (network, opener/dialog, etc.)

### 3.4 Storage Model Must Be Explicit
Decide what is canonical on mobile:
- Remote-client: canonical DB is remote; local storage is cache-only (or none).
- On-device: canonical DB is local to the app sandbox; no "pick arbitrary directory" UX.

Either way, the code must have one canonical owner for "where is the DB and what project is locked", with hard-fail behavior on ambiguity.

### 3.5 Networking (Remote Mode)
If we do remote-client mobile, we need:
- a remote transport for "invoke-like" RPC calls and event streams
- explicit auth and encryption (SSH tunnel / private network)
- strict version/protocol gating (no "best effort")

Do not bolt this into the UI in an ad-hoc way. Keep one transport adapter boundary and keep contracts canonical in `crates/ralph-contracts`.

## 4. Distribution Requirements (High-Level)
Android:
- debug: emulator/device installs (APK)
- release: Play Store (AAB), signing keys, Play Console configuration

iOS:
- signing + provisioning
- TestFlight/App Store Connect distribution
- macOS runner / Mac build host as a hard dependency

## 5. Testing/Verification Requirements
Minimum gates to claim "mobile support exists":
- A compile gate for mobile targets (Android at least; iOS on macOS CI/host).
- Emulator smoke run: app boots, loads the UI, and can perform a representative workflow.
- If remote-client: contract tests that prove mobile can speak to `ralphd` and handle version mismatch as a hard error.
- If on-device: tests that assert desktop-only commands hard-fail on mobile (no silent no-ops).

## 6. Explicitly Out Of Scope (Per Request)
- GUI responsiveness and small-screen layout work.
- UX re-design for touch, keyboard avoidance, and compact navigation.
