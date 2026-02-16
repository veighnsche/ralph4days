# macOS Apple Silicon Desktop Audit (2026-02-16)

## Scope

- Included: desktop Tauri app (`src/`, `src-tauri/`, workspace crates used by desktop flow), build/test/dev scripts.
- Excluded: `ralphd` runtime behavior and websocket swap readiness.

## Host Baseline

- Date: 2026-02-16
- OS: macOS 26.1.0 (`Darwin 25.1.0`)
- Arch: `arm64` (Apple M4)
- Toolchain: Rust 1.90.0, Cargo 1.90.0, Bun 1.3.8, Node 24.11.0
- Xcode: 26.2 + Command Line Tools installed

## Validation Performed

- Static platform/config scan across `justfile`, scripts, `tauri.conf`, Rust cfg gates.
- `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`
- `bunx oxlint src`
- `bunx biome lint src`
- `bunx biome format src`
- `bunx tsc --noEmit`
- types generation/check flow (manual equivalent of `just types-check`)
- contract tests:
  - `cargo test -p core-contracts`
  - `cargo test --manifest-path src-tauri/Cargo.toml --test invoke_command_list_contract_test`
  - `bun test:run src/lib/terminal/terminalBridgeContract.test.ts src/lib/tauri/eventsContract.test.ts src/lib/tauri/tauriImportBoundary.test.ts`
- desktop build checks:
  - `bun tauri build --debug --no-bundle`
  - `bun tauri build`
- desktop dev runtime smoke:
  - `bun tauri dev -- -- --no-splash` (confirmed startup + API server boot on macOS after fix)

## Findings and Fixes Applied

1. Bundle target config was Linux-only.
- Previous: `"targets": ["deb", "rpm", "appimage"]`
- Fix: set to `"targets": "all"` in `src-tauri/tauri.conf.json` so mac builds produce native bundle targets.

2. Dev commands always injected Linux-only WebKit env var.
- Previous: `WEBKIT_DISABLE_DMABUF_RENDERER=1 bun tauri dev` for all OSes.
- Fix: `justfile` now applies that env var only on Linux (`dev` and `dev-mock`).

3. macOS release recipes were missing.
- Fix: added `just release-macos` and `just release-macos-native`.
- `release-macos-native` intentionally uses `RUSTFLAGS='-C target-cpu=native'` for local Apple Silicon optimized builds.

4. Utility recipes were Linux-only.
- Fix: `check-mold` and `sysinfo` are now OS-aware and provide valid macOS output paths.

5. `just types` used `mapfile` (bash 4+), but macOS system bash is 3.2.
- Fix: replaced `mapfile` with a bash-3-compatible null-delimited read loop in `justfile`.

6. Desktop e2e preflight message was Linux-only and misleading on macOS.
- Fix: `scripts/preflight-tauri-e2e.sh` now fails immediately on macOS with explicit "unsupported on macOS" guidance and docs link.

7. README prerequisites/build instructions were Linux-biased.
- Fix: documented `just` install, macOS Xcode/CLT prerequisite, and macOS release commands.

8. Desktop startup panicked on macOS due missing `state_dir`.
- Symptom: panic on launch with `Failed to resolve XDG directories: ... No XDG state directory`.
- Cause: `dirs::state_dir()` can be `None` on macOS while app state initialization treated it as mandatory.
- Fix: `crates/service-runtime/src/xdg.rs` now resolves state dir with explicit precedence:
  - `state_dir()`
  - `data_local_dir()`
  - `data_dir()`
  and emits warnings when fallback is used.
- Added tests for each resolution branch.

## Residual Risk / Known Constraint

- Tauri desktop WebDriver is currently unsupported on macOS. This is an upstream/platform constraint, not a local project bug.

## Result

- Desktop Tauri app builds and validates on Apple Silicon macOS with corrected platform defaults and mac-specific developer/release guidance.
