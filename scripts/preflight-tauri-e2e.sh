#!/usr/bin/env bash
set -euo pipefail

PROJECT_PATH="${RALPH_E2E_PROJECT:-}"
MOCK_DIR="${RALPH_MOCK_DIR:-/tmp/ralph4days-mock}"
DRIVER_BINARY="${TAURI_DRIVER_BINARY:-}"
NATIVE_WEB_DRIVER_NAME='WebKitWebDriver'

if [ -z "${PROJECT_PATH}" ]; then
  PROJECT_PATH="${MOCK_DIR}/04-desktop-dev"
fi

if [ ! -d "${PROJECT_PATH}" ]; then
  echo "❌ E2E project not found: ${PROJECT_PATH}"
  echo "Set RALPH_E2E_PROJECT (or RALPH_MOCK_DIR for defaults) to a valid project directory."
  exit 1
fi

if [ ! -f "${PROJECT_PATH}/.ralph/db/ralph.db" ]; then
  echo "❌ Invalid E2E project: missing .ralph/db/ralph.db at ${PROJECT_PATH}"
  echo "Initialize the project fixture before running e2e tests."
  exit 1
fi

DRIVER_PATH=""
if [ -n "${DRIVER_BINARY}" ]; then
  if [ ! -x "${DRIVER_BINARY}" ]; then
    echo "❌ TAURI_DRIVER_BINARY is set but not executable: ${DRIVER_BINARY}"
    exit 1
  fi
  DRIVER_PATH="${DRIVER_BINARY}"
  echo "✓ Using TAURI_DRIVER_BINARY: ${DRIVER_PATH}"
else
  DRIVER_PATH=$(command -v tauri-driver || true)
  if [ -n "${DRIVER_PATH}" ]; then
    echo "✓ tauri-driver found in PATH: ${DRIVER_PATH}"
  else
    HOME_DRIVER="${HOME}/.cargo/bin/tauri-driver"
    if [ -x "${HOME_DRIVER}" ]; then
      DRIVER_PATH="${HOME_DRIVER}"
      echo "✓ tauri-driver found at ${DRIVER_PATH}"
    fi
  fi
fi

if ! command -v ${NATIVE_WEB_DRIVER_NAME} >/dev/null 2>&1; then
  echo "❌ ${NATIVE_WEB_DRIVER_NAME} not found in PATH."
  echo "Install the native WebKit WebDriver package for your distro:"
  if command -v dnf >/dev/null 2>&1; then
    echo "  Fedora/RHEL: sudo dnf install webkitgtk6.0"
  elif command -v apt-get >/dev/null 2>&1; then
    echo "  Debian/Ubuntu: sudo apt-get install webkit2gtk-driver"
  elif command -v pacman >/dev/null 2>&1; then
    echo "  Arch Linux: pacman -S webkit2gtk"
  fi
  echo "or provide a system-specific native WebKit driver before running e2e."
  echo "Docs: https://v2.tauri.app/develop/tests/webdriver/"
  exit 1
fi

if [ -z "${DRIVER_PATH}" ]; then
  echo "❌ tauri-driver not found."
  echo "Install with: cargo install tauri-driver --locked"
  echo "Or set TAURI_DRIVER_BINARY to an executable tauri-driver binary."
  exit 1
fi

echo "✓ e2e preflight passed"
