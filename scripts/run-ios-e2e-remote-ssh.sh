#!/usr/bin/env bash
set -euo pipefail

WORKSPACE="src-tauri/gen/apple/ralph4days.xcodeproj/project.xcworkspace"
SCHEME="ralph4days_iOS"
TEST_ID="ralph4days_iOSUITests/RemoteSshFlowUITests/testRemoteSshCrudAndDialogFlow"
DEVICE="${RALPH_IOS_E2E_DEVICE:-iPhone 17 Pro}"
SCREENSHOT_DIR="${RALPH_IOS_E2E_SCREENSHOT_DIR:-/tmp/ralph-ios-e2e}"
APP_BUNDLE_ID="${RALPH_IOS_E2E_BUNDLE_ID:-com.vince.ralph}"
RUNNER="${RALPH_IOS_E2E_RUNNER:-appium}"
APPIUM_HOST="${RALPH_IOS_E2E_APPIUM_HOST:-127.0.0.1}"
APPIUM_PORT="${RALPH_IOS_E2E_APPIUM_PORT:-4723}"
APPIUM_LOG="${RALPH_IOS_E2E_APPIUM_LOG:-/tmp/ralph-ios-e2e-appium.log}"
SPEC_PATH="${TAURI_E2E_SPEC:-e2e-ios/remote-ssh.ios.spec.js}"
ARCHIVE_DIR="src-tauri/gen/apple/build/ralph4days_iOS.xcarchive"

if [ "${RUNNER}" != "appium" ] && [ "${RUNNER}" != "xctest" ]; then
  echo "❌ Unsupported RALPH_IOS_E2E_RUNNER: ${RUNNER} (expected: appium or xctest)"
  exit 1
fi

if command -v appium >/dev/null 2>&1; then
  APPIUM_CMD=(appium)
else
  APPIUM_CMD=(bunx --bun appium)
fi

resolve_simulator_udid() {
  local line
  line="$(xcrun simctl list devices available | grep -F " ${DEVICE} (" | head -n 1 || true)"
  if [ -z "${line}" ]; then
    echo ""
    return 0
  fi
  echo "${line}" | sed -E 's/.*\(([0-9A-F-]+)\).*/\1/'
}

resolve_app_path() {
  local candidates=(
    "src-tauri/gen/apple/build/Build/Products/debug-iphonesimulator/ralph4days_iOS.app"
    "src-tauri/gen/apple/build/Build/Products/Debug-iphonesimulator/ralph4days_iOS.app"
    "src-tauri/gen/apple/build/ralph4days_iOS.xcarchive/Products/Applications/ralph4days_iOS.app"
    "src-tauri/gen/apple/build/ralph4days_iOS.xcarchive/Products/Applications/ralph.app"
  )

  local candidate
  for candidate in "${candidates[@]}"; do
    if [ -d "${candidate}" ]; then
      echo "${candidate}"
      return 0
    fi
  done

  local discovered
  discovered="$(
    find src-tauri/gen/apple/build -type d \( -name "ralph4days_iOS.app" -o -name "ralph.app" \) | head -n 1 || true
  )"
  if [ -n "${discovered}" ]; then
    echo "${discovered}"
    return 0
  fi

  echo ""
}

wait_for_appium_ready() {
  local status_url="http://${APPIUM_HOST}:${APPIUM_PORT}/status"
  local i
  for i in $(seq 1 60); do
    if curl -fsS "${status_url}" >/dev/null 2>&1; then
      return 0
    fi
    if ! kill -0 "${APPIUM_PID}" >/dev/null 2>&1; then
      echo "❌ Appium exited before becoming ready. Log: ${APPIUM_LOG}"
      cat "${APPIUM_LOG}" || true
      exit 1
    fi
    sleep 1
  done
  echo "❌ Timed out waiting for Appium at ${status_url}"
  cat "${APPIUM_LOG}" || true
  exit 1
}

cleanup() {
  if [ -n "${APPIUM_PID:-}" ] && kill -0 "${APPIUM_PID}" >/dev/null 2>&1; then
    kill "${APPIUM_PID}" >/dev/null 2>&1 || true
    wait "${APPIUM_PID}" 2>/dev/null || true
  fi
}

trap cleanup EXIT INT TERM

mkdir -p "${SCREENSHOT_DIR}"
rm -rf "${ARCHIVE_DIR}"

echo "==> Running iOS e2e preflight"
bash scripts/preflight-ios-e2e.sh

echo "==> Building iOS simulator app with Tauri (real IPC runtime)"
build_log="$(mktemp -t ralph-ios-build.XXXXXX.log)"
set +e
VITE_E2E_FORCE_REMOTE_PANEL=1 bun tauri ios build --debug --target aarch64-sim --ci 2>&1 | tee "${build_log}"
build_status=${PIPESTATUS[0]}
set -e

if [ "${build_status}" -ne 0 ]; then
  provisional_app_path="$(resolve_app_path)"
  if [ -d "${provisional_app_path}" ] && grep -F "failed to rename app" "${build_log}" >/dev/null 2>&1; then
    echo "⚠️  tauri ios build reported archive rename failure; continuing with produced simulator app: ${provisional_app_path}"
  else
    echo "❌ tauri ios build failed. See log: ${build_log}"
    exit "${build_status}"
  fi
fi

APP_PATH="${RALPH_IOS_E2E_APP_PATH:-$(resolve_app_path)}"
if [ -z "${APP_PATH}" ] || [ ! -d "${APP_PATH}" ]; then
  echo "❌ Failed to resolve simulator app bundle path after build."
  echo "Set RALPH_IOS_E2E_APP_PATH explicitly to ralph4days_iOS.app."
  exit 1
fi

SIMULATOR_UDID="${RALPH_IOS_E2E_UDID:-$(resolve_simulator_udid)}"
if [ -z "${SIMULATOR_UDID}" ]; then
  echo "❌ Failed to resolve simulator UDID for device '${DEVICE}'."
  exit 1
fi

if [ "${RUNNER}" = "xctest" ]; then
  echo "==> Running XCTest UI harness and capturing screenshots"
  RALPH_IOS_E2E_SCREENSHOT_DIR="${SCREENSHOT_DIR}" \
  RALPH_IOS_E2E_PREBUILT_RUST=1 \
  xcodebuild \
    -workspace "${WORKSPACE}" \
    -scheme "${SCHEME}" \
    -destination "id=${SIMULATOR_UDID}" \
    -only-testing:"${TEST_ID}" \
    test

  echo "==> XCTest screenshots written to: ${SCREENSHOT_DIR}"
  exit 0
fi

echo "==> Booting simulator ${DEVICE} (${SIMULATOR_UDID})"
xcrun simctl boot "${SIMULATOR_UDID}" >/dev/null 2>&1 || true
xcrun simctl bootstatus "${SIMULATOR_UDID}" -b

echo "==> Resetting app install for deterministic Appium session"
xcrun simctl uninstall "${SIMULATOR_UDID}" "${APP_BUNDLE_ID}" >/dev/null 2>&1 || true
xcrun simctl install "${SIMULATOR_UDID}" "${APP_PATH}"

echo "==> Starting Appium on ${APPIUM_HOST}:${APPIUM_PORT}"
rm -f "${APPIUM_LOG}"
"${APPIUM_CMD[@]}" --address "${APPIUM_HOST}" --port "${APPIUM_PORT}" --base-path / >"${APPIUM_LOG}" 2>&1 &
APPIUM_PID=$!
wait_for_appium_ready

echo "==> Running Appium + WebDriverIO iOS harness"
RALPH_IOS_E2E_SCREENSHOT_DIR="${SCREENSHOT_DIR}" \
RALPH_IOS_E2E_DEVICE="${DEVICE}" \
RALPH_IOS_E2E_UDID="${SIMULATOR_UDID}" \
RALPH_IOS_E2E_BUNDLE_ID="${APP_BUNDLE_ID}" \
RALPH_IOS_E2E_APP_PATH="${APP_PATH}" \
RALPH_IOS_E2E_APPIUM_HOST="${APPIUM_HOST}" \
RALPH_IOS_E2E_APPIUM_PORT="${APPIUM_PORT}" \
bun x wdio run wdio.ios.appium.conf.js --spec "${SPEC_PATH}"

echo "==> Appium screenshots written to: ${SCREENSHOT_DIR}"
