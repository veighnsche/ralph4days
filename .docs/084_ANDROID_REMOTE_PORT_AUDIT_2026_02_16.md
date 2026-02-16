# 084 Android Remote Port Audit (2026-02-16)

Date: 2026-02-16
Auditor: Codex
Scope: Android mobile port readiness for remote-only `ralphd` backend, with emphasis on automation quality (unit/integration/E2E) and anti-reward-hacking posture.

## Audit Method
- Static code/config review across `src-tauri`, `src-daemon`, frontend, Android Gradle project, and CI workflow.
- Gate execution:
- `just verify` (pass)
- `just verify-swap` (pass)
- `just check-mobile` (pass)
- Dependency inspection:
- `cargo tree --manifest-path src-tauri/Cargo.toml --target aarch64-linux-android -e normal`

## High-Severity Findings

### 1) No user-facing mobile connection flow to `ralphd` (functional blocker)
Evidence:
- Mobile hard-requires remote connection before stateful calls:
  - `src-tauri/src/commands/state_mobile.rs:21`
  - `src-tauri/src/commands/state_mobile.rs:42`
- Frontend starts by calling `project_lock_get` immediately:
  - `src/App.tsx:42`
- No frontend call sites for remote connect/status/disconnect:
  - `src` search contains no `remote_connect`/`remote_status_get`/`remote_disconnect` calls.
- Remote commands exist only in Tauri backend:
  - `src-tauri/src/commands/remote.rs:8`
  - `src-tauri/src/commands/remote.rs:56`

Impact:
- On mobile, app behavior is effectively blocked unless connection is done manually outside normal UX (for example, devtools/console or external scripting).

Required fix:
- Add a first-class connection manager UI/flow (connect, status, reconnect, disconnect) and route app startup through it.

### 2) Android automated test stack is not implemented
Evidence:
- No Android instrumentation or local unit test source roots:
  - `src-tauri/gen/android/app/src/androidTest` missing
  - `src-tauri/gen/android/app/src/test` missing
- Android Gradle config includes test dependencies but no instrumentation runner/orchestrator setup:
  - `src-tauri/gen/android/app/build.gradle.kts:66`
  - `src-tauri/gen/android/app/build.gradle.kts:67`
  - Missing `defaultConfig.testInstrumentationRunner`
  - Missing `testOptions.execution = "ANDROIDX_TEST_ORCHESTRATOR"`
- CI has only contract verification workflow; no Android build/test jobs:
  - `.github/workflows/verify-contract.yml:1`

Impact:
- No automated Android E2E confidence path exists today; regressions will be detected late and manually.

Required fix:
- Implement instrumentation test module(s), managed-emulator CI job(s), and real-device matrix nightly gate.

## Medium-Severity Findings

### 3) Open checklist invariant: `tower*` still in mobile dependency graph
Evidence:
- Open item explicitly tracked:
  - `.docs/081_TAURI_MOBILE_REMOTE_ONLY_BACKEND_CHECKLIST.md:59`
  - `.docs/081_TAURI_MOBILE_REMOTE_ONLY_BACKEND_CHECKLIST.md:109`
- Current mobile gate excludes `sqlite-db|prompt-builder|ralph-backend|portable-pty|axum` but not `tower*`:
  - `justfile:94`
- Actual Android target tree still includes `reqwest`/`tower*` transitive chain.

Impact:
- Remaining deviation from declared dependency target policy for mobile thinness.

Required fix:
- Decide policy outcome:
- Accept `tower*` as framework-transitive and update checklist/policy text.
- Or hard-enforce exclusion (if technically feasible) and make gate fail on presence.

### 4) Mobile proxy adapters are untyped `serde_json::Value` pass-throughs
Evidence:
- Mobile command adapters decode to `serde_json::Value` instead of typed DTOs:
  - `src-tauri/src/commands/project_mobile.rs:10`
  - `src-tauri/src/commands/tasks_mobile.rs:9`
- Current notes acknowledge this decoupling approach:
  - `.docs/081_TAURI_MOBILE_REMOTE_ONLY_BACKEND_CHECKLIST.md:95`

Impact:
- Less compile-time contract safety in mobile adapter layer; errors move to runtime decode paths.

Required fix:
- Keep this only if intentional and tested.
- Add parity/strictness tests specifically for mobile adapter payload shapes.

### 5) Remote host transport management still external/manual
Evidence:
- `remote_connect` takes raw `wsUrl`:
  - `src-tauri/src/commands/remote.rs:11`
- Architecture notes explicitly state SSH tunnel/connection manager is not implemented in this layer:
  - `.docs/078_TAURI_MOBILE_IOS_ANDROID_ENABLEMENT_DUMP.md:132`

Impact:
- End-user connect/reconnect lifecycle to another machine is not production-ready without external setup.

Required fix:
- Implement connection manager ownership boundary (SSH/Tailscale integration + local forwarded endpoint lifecycle).

## Low-Severity Findings

### 6) Contract/test gates are strong, but Android-side gates are absent
Evidence:
- Positive:
  - Command surface drift gate:
    - `src-tauri/tests/invoke_command_list_contract_test.rs:58`
  - Strict decode guard:
    - `src-tauri/tests/remote_strict_decode_contract_test.rs:7`
  - Protocol mismatch hard-fail:
    - `src-tauri/src/remote.rs:293`
  - WS parity smoke:
    - `src-daemon/tests/ws_parity_smoke_test.rs:165`
- Gap:
  - None of these are Android instrumentation E2E tests.

Impact:
- Backend/protocol contract confidence is good; device-level confidence remains weak.

## What Is Working Well
- Mobile backend is structurally thin and remote-required by design.
- Desktop-only deps are largely gated from mobile target in `src-tauri/Cargo.toml`.
- Remote WS protocol handshake is strict and fail-fast.
- Swap/readiness contracts and ralphd WS smoke tests are in place and passing.
- `just verify`, `just verify-swap`, and `just check-mobile` pass as of this audit.

## Priority Remediation Checklist
- [ ] Add mobile connection UX flow (connect/status/reconnect/disconnect) and make app boot path explicit.
- [ ] Add Android instrumentation harness with `AndroidJUnitRunner`.
- [ ] Add `Espresso-Web` coverage for key WebView/Tauri UI flows.
- [ ] Enable Android Test Orchestrator and deterministic fixture reset.
- [ ] Add PR Android emulator gate (managed device).
- [ ] Add nightly real-device matrix gate (Firebase Test Lab or equivalent).
- [ ] Resolve `tower*` policy mismatch (either enforce or explicitly accept and document).
- [ ] Add mobile adapter payload parity tests if keeping `serde_json::Value` pass-through pattern.

## Verdict
- Architecture direction: strong.
- Protocol/contract quality: strong.
- Android production test posture: incomplete.
- Remote mobile UX readiness: blocked pending connection manager flow.
