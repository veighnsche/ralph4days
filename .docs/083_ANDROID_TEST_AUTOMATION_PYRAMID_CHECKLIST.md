# 083 Android Test Automation Pyramid Checklist

Date: 2026-02-16
Scope: `ralph4days` mobile port (Tauri mobile + remote-only backend to `ralphd`)

## Goal
- Build a deterministic automated test stack with strong signal quality:
- Unit tests first.
- Integration tests second.
- Android E2E tests last (small, high-value critical paths only).

Policy alignment:
- Fail-fast only. Broken invariants must fail loudly.
- No silent fallbacks in tests or app logic.

## Best-Practice Answer (Direct)
- Primary Android E2E stack: native Android instrumentation tests.
- Use `AndroidJUnitRunner` + `Espresso` for in-app flows.
- Use `Espresso-Web` for WebView assertions (important for Tauri UI content).
- Use `UI Automator` only where system UI interaction is required (permissions/app switching).
- Use Android Test Orchestrator for test isolation and reduced flake.
- Run emulator E2E on PRs (fast gate), then real-device matrix nightly/release.

## Test Pyramid (Target Split)
- Unit: 70-80%
- Integration: 15-25%
- E2E: 5-10%

Rationale:
- E2E is the most expensive and flaky layer.
- Most behavior should be validated below E2E where failures are faster to diagnose.

## Layer 1: Unit Tests Checklist
- [ ] Frontend unit tests cover pure UI logic, stores, reducers, and adapter boundaries (`vitest`).
- [ ] Rust unit tests cover contract encode/decode, remote wire framing, and hard-fail error paths.
- [ ] Every mobile-only command proxy path has unit tests for:
- [ ] connected state
- [ ] disconnected state
- [ ] protocol mismatch/error propagation
- [ ] Unit tests do not require emulator/device startup.

## Layer 2: Integration Tests Checklist
- [ ] Add backend integration tests for command proxy parity between desktop and mobile command surfaces.
- [ ] Validate remote payload envelope invariants (`{ "args": ... }`) at adapter boundaries.
- [ ] Validate event re-emission (`terminal:output`, `terminal:closed`, diagnostics) through integration tests.
- [ ] Use seeded deterministic fixtures for remote state so failures are reproducible.
- [ ] Include negative-path integration tests (malformed payloads, unknown fields, disconnected transport).

## Layer 3: Android E2E (Instrumentation) Checklist
- [ ] Keep E2E scope to critical journeys only:
- [ ] app boot + backend connection
- [ ] lock/select project
- [ ] view/update core task/subsystem flows
- [ ] terminal output stream render + closed event handling
- [ ] Use `androidx.test.runner.AndroidJUnitRunner`.
- [ ] Use `Espresso` for app UI behavior.
- [ ] Use `Espresso-Web` to assert WebView DOM state for Tauri-hosted frontend.
- [ ] Use `UI Automator` only for system-level steps.
- [ ] Enable Android Test Orchestrator with `clearPackageData` for test isolation.
- [ ] Collect screenshots, logcat, and test reports on failure.

## CI/CD Strategy Checklist
- [ ] PR gate (fast):
- [ ] unit + integration suite
- [ ] Android emulator E2E on Gradle Managed Devices (single API level/device profile)
- [ ] Nightly/release gate (broad):
- [ ] Firebase Test Lab real-device matrix (API levels + hardware vendors)
- [ ] locale/timezone variations for regression detection
- [ ] Fail pipeline on flaky threshold breach; do not silently quarantine forever.

## Flake-Control Rules
- [ ] No fixed sleeps as primary synchronization strategy.
- [ ] Synchronize on observable state (view idle, DOM ready signal, backend event).
- [ ] One test = one scenario = one assertion cluster.
- [ ] Isolate test data per test run (orchestrator + explicit fixture reset).
- [ ] Retry policy only for external lab instability, never for deterministic assertion failures.

## Performance + Robustness Gates
- [ ] Add Android Macrobenchmark coverage for startup and critical render paths.
- [ ] Add Baseline Profile generation/validation for release builds.
- [ ] Track boot/connect time and first interactive screen as explicit KPIs.

## Repo-Specific Guardrails (Ralph Mobile)
- [ ] Keep mobile backend remote-only. No local desktop fallback paths.
- [ ] Keep strict decode posture (`deny_unknown_fields`) for IPC contracts.
- [ ] Keep protocol mismatch as hard failure.
- [ ] Keep `just check-mobile` as compile/dependency gate.
- [ ] Expand contract parity tests before adding new E2E cases.

## Initial Implementation Plan (Execution Order)
- [ ] Phase 1: stabilize/add missing unit tests for mobile proxy behavior.
- [ ] Phase 2: add integration tests for remote adapter/event propagation invariants.
- [ ] Phase 3: add Android instrumentation skeleton in `src-tauri/gen/android` test modules.
- [ ] Phase 4: add first 2-3 critical E2E flows only.
- [ ] Phase 5: wire PR emulator gate + nightly real-device matrix.

## Definition of Done
- [ ] Unit + integration tests pass locally and in CI.
- [ ] Android emulator E2E passes on every PR.
- [ ] Nightly real-device run is green for target API/device matrix.
- [ ] Failure artifacts are attached automatically.
- [ ] No silent fallback behavior introduced for test stability.

## References
- Android testing strategy: https://developer.android.com/training/testing/fundamentals/strategies
- Gradle Managed Devices: https://developer.android.com/studio/test/managed-devices
- AndroidJUnitRunner + AndroidX test libs: https://developer.android.com/training/testing/instrumented-tests/androidx-test-libraries/runner
- Espresso: https://developer.android.com/training/testing/espresso
- Espresso-Web: https://developer.android.com/training/testing/espresso/web
- UI Automator: https://developer.android.com/training/testing/other-components/ui-automator
- Firebase Test Lab instrumentation: https://firebase.google.com/docs/test-lab/android/instrumentation-test
- Macrobenchmark: https://developer.android.com/topic/performance/benchmarking/macrobenchmark-overview
- Tauri testing docs: https://v2.tauri.app/develop/tests/
