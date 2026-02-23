#!/usr/bin/env bash
set -euo pipefail

TARGET_HOST="${RALPH_IOS_E2E_TARGET_HOST:-127.0.0.1}"
TARGET_USER="${RALPH_IOS_E2E_TARGET_USERNAME:-$(id -un)}"
TARGET_NAME="${RALPH_IOS_E2E_TARGET_NAME:-macOS Target}"
TARGET_SSH_PORT="${RALPH_IOS_E2E_TARGET_SSH_PORT:-22}"
TARGET_RALPHD_PORT="${RALPH_IOS_E2E_TARGET_RALPHD_PORT:-9944}"
RESET_APP_STATE="${RALPH_IOS_E2E_RESET_APP_STATE:-0}"

echo "==> Provisioning macOS SSH target with fixture mocks + ralphd"
bash scripts/setup-ios-e2e-macos-target.sh

echo "==> Running iOS Appium harness against macOS SSH target profile"
RALPH_IOS_E2E_TARGET_NAME="${TARGET_NAME}" \
RALPH_IOS_E2E_TARGET_HOST="${TARGET_HOST}" \
RALPH_IOS_E2E_TARGET_USERNAME="${TARGET_USER}" \
RALPH_IOS_E2E_TARGET_SSH_PORT="${TARGET_SSH_PORT}" \
RALPH_IOS_E2E_TARGET_RALPHD_PORT="${TARGET_RALPHD_PORT}" \
RALPH_IOS_E2E_RESET_APP_STATE="${RESET_APP_STATE}" \
TAURI_E2E_SPEC="e2e-ios/remote-ssh.macos-target.ios.spec.js" \
bash scripts/run-ios-e2e-remote-ssh.sh
