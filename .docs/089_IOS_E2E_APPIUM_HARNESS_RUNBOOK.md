# 089 iOS E2E Appium Harness Runbook

## Scope
- Runs the real Tauri iOS simulator app (Rust IPC + frontend) and drives SSH mobile UI flows end to end.
- Captures deterministic screenshots for key states in the SSH connection panel.
- Keeps XCTest harness as a fallback runner for environments where Appium is not preferred.

## Entrypoints
- `just test-ios-e2e-remote-ssh "iPhone 17 Pro"`: default Appium runner.
- `just test-ios-e2e-remote-ssh-macos-target "iPhone 17 Pro"`: provisions a macOS SSH target (fixture + `ralphd`) then runs a target-profile setup spec.
- `just test-ios-e2e-remote-ssh-xctest "iPhone 17 Pro"`: fallback XCTest runner.
- `bun run test:ios-e2e-remote-ssh`: npm-style alias for default runner.
- `bun run test:ios-e2e-remote-ssh:macos-target`: npm-style alias for macOS target provisioning + spec run.

## Dependency Preflight
- Script: `scripts/preflight-ios-e2e.sh`
- Hard-fails when:
  - Host OS is not macOS.
  - Required commands are missing (`bun`, `xcodebuild`, `xcrun`, `curl`, `appium`).
  - Appium XCUITest driver is not installed.
  - Requested simulator device is not present.

## Runtime Flow (Default Appium Runner)
1. Build simulator app via `bun tauri ios build --debug --target aarch64-sim --ci` with `VITE_E2E_FORCE_REMOTE_PANEL=1`.
2. Resolve built `ralph4days_iOS.app` path from Tauri/Xcode outputs.
3. Boot target simulator and reset installed app for deterministic state.
4. Start Appium server and wait for `/status`.
5. Run `wdio` with `wdio.ios.appium.conf.js` and spec `e2e-ios/remote-ssh.ios.spec.js`.
6. Write screenshots to `RALPH_IOS_E2E_SCREENSHOT_DIR` (default `/tmp/ralph-ios-e2e`).

## Runtime Flow (macOS Target Provisioned Runner)
1. Reset local fixture mocks (`scripts/reset-mock.sh`).
2. Build release `ralphd` binary (`cargo build -p ralphd --release`).
3. Install `ralphd` + fixture project on the macOS target (`/tmp/ralph-ios-e2e-target` by default).
4. Start `ralphd` on target loopback (`127.0.0.1:${RALPH_IOS_E2E_TARGET_RALPHD_PORT:-9944}`).
5. Run iOS Appium harness with spec `e2e-ios/remote-ssh.macos-target.ios.spec.js`.
6. Assert first screen is SSH config and save a profile with configured macOS target host/user/ports.

## Environment Knobs
- `RALPH_IOS_E2E_RUNNER`: `appium` (default) or `xctest`.
- `RALPH_IOS_E2E_DEVICE`: simulator display name.
- `RALPH_IOS_E2E_UDID`: optional explicit simulator UDID.
- `RALPH_IOS_E2E_APP_PATH`: optional explicit `.app` path override.
- `RALPH_IOS_E2E_SCREENSHOT_DIR`: screenshot output directory.
- `RALPH_IOS_E2E_APPIUM_HOST`, `RALPH_IOS_E2E_APPIUM_PORT`: Appium endpoint.
- `RALPH_IOS_E2E_TARGET_HOST`: macOS SSH target host (default `127.0.0.1`).
- `RALPH_IOS_E2E_TARGET_USERNAME`: SSH username for target (default current user).
- `RALPH_IOS_E2E_TARGET_SSH_PORT`: SSH port (default `22`).
- `RALPH_IOS_E2E_TARGET_RALPHD_PORT`: `ralphd` bind port on target loopback (default `9944`).
- `RALPH_IOS_E2E_TARGET_FIXTURE`: fixture directory name copied to target (default `04-desktop-dev`).
- `RALPH_IOS_E2E_TARGET_BASE_DIR`: install root on target (default `/tmp/ralph-ios-e2e-target`).

## Failure Posture
- No silent fallback to web-only runtime.
- No silent runner switching.
- Missing app build artifacts, missing simulator, Appium startup failure, or selector drift all fail loud with explicit messages.
