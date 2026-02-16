# 085 Entire Audit: Android + Remote-Only Mobile Backend (2026-02-16)

Date: 2026-02-16
Auditor: Codex
Scope: current `main` HEAD audit for Tauri mobile remote-only backend posture, Android readiness, CI/testing completeness, and checklist drift.

## Evidence Run
- `just verify` (pass)
- `just verify-swap` (pass)
- `just check-mobile` (pass)
- `bun test:run src/lib/tauri/mobileGate.test.ts` (pass)
- `cargo tree --manifest-path src-tauri/Cargo.toml --target aarch64-linux-android -e normal`
- `cargo tree --manifest-path src-tauri/Cargo.toml --target aarch64-linux-android -e normal -i tower`

## High-Severity Findings

### 1) Android automated test stack is still not implemented
Evidence:
- Android test source roots are missing:
  - `src-tauri/gen/android/app/src/androidTest` (missing)
  - `src-tauri/gen/android/app/src/test` (missing)
- Gradle config has test deps but no instrumentation runner/orchestrator wiring:
  - `src-tauri/gen/android/app/build.gradle.kts:19`
  - `src-tauri/gen/android/app/build.gradle.kts:65`
  - Missing `defaultConfig.testInstrumentationRunner`
  - Missing `testOptions.execution = "ANDROIDX_TEST_ORCHESTRATOR"`
- CI only runs contract verification and has no Android build/test job:
  - `.github/workflows/verify-contract.yml:1`

Impact:
- Android regressions are still detected manually and late.

Required fix:
- Add instrumentation test module(s), runner config, orchestrator config, managed emulator CI gate, and nightly real-device matrix.

## Medium-Severity Findings

### 2) Mobile connection lifecycle is now present but not complete (connect-only UI)
Status change vs prior audit:
- Resolved: there is now a first-class connection gate UI before project lock load.

Evidence:
- Mobile connection gate exists and is wired into app boot path:
  - `src/App.tsx:118`
  - `src/App.tsx:191`
  - `src/components/app-shell/RemoteConnectionPanel.tsx:34`
- Backend supports `remote_disconnect`:
  - `src-tauri/src/commands/remote.rs:46`
- Frontend has no disconnect flow (`remote_disconnect` is not called anywhere under `src/`).
- Remote status is fetched once without polling/subscription:
  - `src/App.tsx:125`
  - `src/hooks/api/useInvoke.ts:47`
  - Invalidations happen on connect only: `src/App.tsx:182`

Impact:
- Initial mobile connect works.
- Mid-session disconnect/reconnect UX is weak and can become stale until user action triggers refresh.

Required fix:
- Add explicit disconnect/reconnect UI and status refresh strategy (polling or push event).

### 3) Checklist policy drift: `tower*` remains in mobile graph, gate does not enforce it
Evidence:
- Checklist still tracks `tower*` as disallowed target:
  - `.docs/081_TAURI_MOBILE_REMOTE_ONLY_BACKEND_CHECKLIST.md:59`
  - `.docs/081_TAURI_MOBILE_REMOTE_ONLY_BACKEND_CHECKLIST.md:109`
- Current mobile dependency gate does not check `tower*`:
  - `justfile:94`
- Actual Android target graph includes `tower`/`tower-http` via `tauri -> reqwest`.

Impact:
- Policy says one thing, enforcement says another.

Required fix:
- Either explicitly allow framework-transitive `tower*` and update checklist text, or enforce it in `check-mobile` and make gate fail.

### 4) Mobile adapter layer remains untyped (`serde_json::Value`) and lacks dedicated parity tests
Evidence:
- Mobile command adapters forward `serde_json::Value` payloads/results:
  - `src-tauri/src/commands/project_mobile.rs:10`
  - `src-tauri/src/commands/tasks_mobile.rs:9`
- Strict decode tests exist, but they are contract-level and not adapter-shape focused:
  - `src-tauri/tests/remote_strict_decode_contract_test.rs:7`

Impact:
- Adapter layer relies on runtime failures instead of compile-time DTO checks.

Required fix:
- Add mobile adapter payload-shape tests or migrate selected high-risk commands back to typed DTOs.

## Low-Severity Findings

### 5) Android release cleartext policy + default ws URL can mislead in non-debug setups
Evidence:
- Default URL in connection panel is cleartext localhost:
  - `src/components/app-shell/RemoteConnectionPanel.tsx:11`
- Release build disables cleartext by default:
  - `src-tauri/gen/android/app/build.gradle.kts:20`

Impact:
- Debug/dev path is fine; release users must use `wss://` and may fail if they copy the default `ws://` pattern.

Required fix:
- Add validation/hinting in UI based on build mode or URL scheme; keep clear error guidance.

## Strengths / What Is Working
- Mobile backend is structurally remote-only and hard-fails when not connected:
  - `src-tauri/src/commands/state_mobile.rs:16`
- Mobile bootstrap skips local backend setup:
  - `src-tauri/src/lib.rs:96`
- Full command surface is still registered for frontend reuse:
  - `src-tauri/src/lib.rs:187`
- Protocol mismatch handling is strict and fail-fast:
  - `src-tauri/src/remote.rs:293`
- Swap/parity contract coverage is active:
  - `src-daemon/tests/ws_parity_smoke_test.rs:165`

## Updated Priority Checklist
- [ ] Add Android instrumentation harness (`AndroidJUnitRunner`) and first critical-path instrumentation tests.
- [ ] Enable Android Test Orchestrator and deterministic isolation in Gradle.
- [ ] Add CI Android emulator gate on PRs.
- [ ] Add nightly/release real-device matrix gate.
- [ ] Add frontend disconnect/reconnect controls using `remote_disconnect`.
- [ ] Add remote status refresh policy (polling or push-based invalidation).
- [ ] Resolve `tower*` policy mismatch (accept+document or enforce in gate).
- [ ] Add mobile adapter payload parity tests for `serde_json::Value` bridge commands.

## Verdict
- Remote-only backend architecture: solid.
- Mobile boot/connect gate: now present and functional for first connect.
- Android production test posture: still insufficient.
- Remaining blockers are mostly test automation and lifecycle hardening, not core protocol architecture.
