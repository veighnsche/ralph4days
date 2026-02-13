# TESTING STRATEGY — TAURI WEBDRIVER (NATIVE-ONLY)

## Current State (ENFORCED)

Ralph4days has three test layers with a strict native boundary:

| Test Type | Tooling | Scope |
|-----------|---------|-------|
| Rust Backend | `cargo test` | Unit and backend behavior |
| Frontend Unit | Vitest (`bun test:run`) | React components/hooks |
| E2E Integration | `wdio` + `tauri-driver` | Compiled Tauri desktop app |

## Hard Rule

E2E checks must run against the native Tauri window. Non-native UI automation is forbidden.

## Why Playwright is not used

Playwright tests were used in an earlier draft on the web frontend only and cannot validate:

- real Tauri IPC calls
- PTY lifecycle
- Rust process and file-system-backed behavior
- native window-level interactions

Those gaps caused false confidence in integration readiness.

## Current Native Flow

- Build debug Tauri app (`bun tauri build --debug --no-bundle`)
- Start `tauri-driver`
- Run `wdio` specs against `wdio: wry` / `tauri:options`

Commands wired in this repo:

- `test:e2e` -> `bunx wdio run wdio.conf.js`
- `test:e2e-terminal` -> `bunx wdio run wdio.conf.js --spec e2e-tauri/terminal.spec.js`
- `audit:no-playwright` -> hard-fails on any runtime Playwright mention in active harness surface

## Native E2E Command Surface

`just test-e2e` and `just test-e2e-terminal` are the only canonical e2e tasks.
They both:

1. validate mock project readiness for `.ralph/db/ralph.db`
2. invoke the Playwright-guard audit
3. launch WebdriverIO + `tauri-driver`

## Implemented Coverage

- `e2e-tauri/terminal.spec.js` verifies terminal host appears after opening a new terminal in the native app
- `wdio.conf.js` enforces:
  - native capabilities
  - debug binary existence
  - explicit `--project` argument
  - `tauri-driver` availability

## References

- Tauri testing guide: https://tauri.app/v1/guides/testing/webdriver/introduction
- WebdriverIO: https://webdriver.io/docs/what-is-webdriverio
