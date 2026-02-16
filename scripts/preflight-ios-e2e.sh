#!/usr/bin/env bash
set -euo pipefail

DEVICE="${RALPH_IOS_E2E_DEVICE:-iPhone 17 Pro}"
OS_NAME="$(uname -s)"

if [ "${OS_NAME}" != "Darwin" ]; then
  echo "❌ iOS e2e harness requires macOS. Current OS: ${OS_NAME}"
  exit 1
fi

required_commands=(
  bun
  xcodebuild
  xcrun
  curl
)

for command_name in "${required_commands[@]}"; do
  if ! command -v "${command_name}" >/dev/null 2>&1; then
    echo "❌ Missing required command: ${command_name}"
    exit 1
  fi
done

if command -v appium >/dev/null 2>&1; then
  APPIUM_CMD=(appium)
else
  APPIUM_CMD=(bunx --bun appium)
fi

if ! "${APPIUM_CMD[@]}" --version >/dev/null 2>&1; then
  echo "❌ Appium command is unavailable."
  if [ "${APPIUM_CMD[0]}" = "appium" ]; then
    echo "Install Appium globally or ensure it is in PATH."
  else
    echo "Unable to execute 'bunx --bun appium'."
  fi
  exit 1
fi

if ! "${APPIUM_CMD[@]}" driver list --installed --json 2>/dev/null | grep -iq '"xcuitest"'; then
  echo "❌ Appium XCUITest driver is not installed."
  if [ "${APPIUM_CMD[0]}" = "appium" ]; then
    echo "Install with: appium driver install xcuitest"
  else
    echo "Install with: bunx --bun appium driver install xcuitest"
  fi
  exit 1
fi

if ! xcrun simctl list devices available | grep -F " ${DEVICE} (" >/dev/null 2>&1; then
  echo "❌ iOS Simulator device not found: ${DEVICE}"
  echo "Available devices:"
  xcrun simctl list devices available
  exit 1
fi

echo "✓ iOS e2e preflight passed"
