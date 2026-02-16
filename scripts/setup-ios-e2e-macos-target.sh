#!/usr/bin/env bash
set -euo pipefail

TARGET_HOST="${RALPH_IOS_E2E_TARGET_HOST:-127.0.0.1}"
TARGET_USER="${RALPH_IOS_E2E_TARGET_USERNAME:-$(id -un)}"
TARGET_SSH_PORT="${RALPH_IOS_E2E_TARGET_SSH_PORT:-22}"
TARGET_BASE_DIR="${RALPH_IOS_E2E_TARGET_BASE_DIR:-/tmp/ralph-ios-e2e-target}"
TARGET_RALPHD_PORT="${RALPH_IOS_E2E_TARGET_RALPHD_PORT:-9944}"
TARGET_FIXTURE="${RALPH_IOS_E2E_TARGET_FIXTURE:-04-desktop-dev}"
MOCK_DIR="${RALPH_MOCK_DIR:-/tmp/ralph4days-mock}"

LOCAL_RALPHD_BIN="target/release/ralphd"
LOCAL_FIXTURE_DIR="${MOCK_DIR}/${TARGET_FIXTURE}"
TARGET_BIN_DIR="${TARGET_BASE_DIR}/bin"
TARGET_PROJECTS_DIR="${TARGET_BASE_DIR}/projects"
TARGET_PROJECT_DIR="${TARGET_PROJECTS_DIR}/${TARGET_FIXTURE}"
TARGET_LOG_PATH="${TARGET_BASE_DIR}/ralphd.log"
TARGET_PID_PATH="${TARGET_BASE_DIR}/ralphd.pid"

if ! command -v cargo >/dev/null 2>&1; then
  echo "❌ Missing required command: cargo"
  exit 1
fi

if ! command -v tar >/dev/null 2>&1; then
  echo "❌ Missing required command: tar"
  exit 1
fi

if ! command -v bash >/dev/null 2>&1; then
  echo "❌ Missing required command: bash"
  exit 1
fi

is_loopback_target=0
if [ "${TARGET_HOST}" = "127.0.0.1" ] || [ "${TARGET_HOST}" = "localhost" ] || [ "${TARGET_HOST}" = "::1" ]; then
  is_loopback_target=1
fi

if [ "${is_loopback_target}" -ne 1 ]; then
  if ! command -v ssh >/dev/null 2>&1; then
    echo "❌ Missing required command: ssh"
    exit 1
  fi
  if ! command -v scp >/dev/null 2>&1; then
    echo "❌ Missing required command: scp"
    exit 1
  fi
fi

echo "==> Preparing local fixture mocks"
bash scripts/reset-mock.sh

if [ ! -d "${LOCAL_FIXTURE_DIR}" ] || [ ! -f "${LOCAL_FIXTURE_DIR}/.ralph/db/ralph.db" ]; then
  echo "❌ Fixture project is missing or invalid: ${LOCAL_FIXTURE_DIR}"
  exit 1
fi

echo "==> Building ralphd release binary"
cargo build -p ralphd --release

if [ ! -x "${LOCAL_RALPHD_BIN}" ]; then
  echo "❌ Expected ralphd binary not found: ${LOCAL_RALPHD_BIN}"
  exit 1
fi

start_local_target() {
  mkdir -p "${TARGET_BIN_DIR}" "${TARGET_PROJECTS_DIR}"
  cp "${LOCAL_RALPHD_BIN}" "${TARGET_BIN_DIR}/ralphd"

  rm -rf "${TARGET_PROJECT_DIR}"
  cp -R "${LOCAL_FIXTURE_DIR}" "${TARGET_PROJECT_DIR}"

  if [ -f "${TARGET_PID_PATH}" ]; then
    prior_pid="$(cat "${TARGET_PID_PATH}")"
    if [ -n "${prior_pid}" ] && kill -0 "${prior_pid}" >/dev/null 2>&1; then
      kill "${prior_pid}"
      wait "${prior_pid}" 2>/dev/null || true
    fi
    rm -f "${TARGET_PID_PATH}"
  fi

  nohup "${TARGET_BIN_DIR}/ralphd" --bind "127.0.0.1:${TARGET_RALPHD_PORT}" >"${TARGET_LOG_PATH}" 2>&1 &
  echo "$!" >"${TARGET_PID_PATH}"
}

start_remote_target() {
  ssh -p "${TARGET_SSH_PORT}" "${TARGET_USER}@${TARGET_HOST}" \
    "mkdir -p '${TARGET_BIN_DIR}' '${TARGET_PROJECTS_DIR}'"

  scp -P "${TARGET_SSH_PORT}" "${LOCAL_RALPHD_BIN}" "${TARGET_USER}@${TARGET_HOST}:${TARGET_BIN_DIR}/ralphd"

  tar -C "${MOCK_DIR}" -cf - "${TARGET_FIXTURE}" | ssh -p "${TARGET_SSH_PORT}" "${TARGET_USER}@${TARGET_HOST}" \
    "rm -rf '${TARGET_PROJECT_DIR}' && mkdir -p '${TARGET_PROJECTS_DIR}' && tar -C '${TARGET_PROJECTS_DIR}' -xf -"

  ssh -p "${TARGET_SSH_PORT}" "${TARGET_USER}@${TARGET_HOST}" "\
set -euo pipefail
if [ -f '${TARGET_PID_PATH}' ]; then
  prior_pid=\"\$(cat '${TARGET_PID_PATH}')\"
  if [ -n \"\${prior_pid}\" ] && kill -0 \"\${prior_pid}\" >/dev/null 2>&1; then
    kill \"\${prior_pid}\"
    wait \"\${prior_pid}\" 2>/dev/null || true
  fi
  rm -f '${TARGET_PID_PATH}'
fi
nohup '${TARGET_BIN_DIR}/ralphd' --bind '127.0.0.1:${TARGET_RALPHD_PORT}' >'${TARGET_LOG_PATH}' 2>&1 </dev/null &
echo \$! > '${TARGET_PID_PATH}'
"
}

verify_local_target() {
  pid="$(cat "${TARGET_PID_PATH}")"
  if [ -z "${pid}" ] || ! kill -0 "${pid}" >/dev/null 2>&1; then
    echo "❌ Local ralphd is not running. Log: ${TARGET_LOG_PATH}"
    cat "${TARGET_LOG_PATH}" || true
    exit 1
  fi

  if [ ! -f "${TARGET_PROJECT_DIR}/.ralph/db/ralph.db" ]; then
    echo "❌ Local target fixture is missing database: ${TARGET_PROJECT_DIR}/.ralph/db/ralph.db"
    exit 1
  fi
}

verify_remote_target() {
  ssh -p "${TARGET_SSH_PORT}" "${TARGET_USER}@${TARGET_HOST}" "\
set -euo pipefail
pid=\"\$(cat '${TARGET_PID_PATH}')\"
if [ -z \"\${pid}\" ] || ! kill -0 \"\${pid}\" >/dev/null 2>&1; then
  echo '❌ Remote ralphd is not running. Log follows:'
  cat '${TARGET_LOG_PATH}' || true
  exit 1
fi
if [ ! -f '${TARGET_PROJECT_DIR}/.ralph/db/ralph.db' ]; then
  echo '❌ Remote target fixture is missing database: ${TARGET_PROJECT_DIR}/.ralph/db/ralph.db'
  exit 1
fi
"
}

echo "==> Installing fixture and starting ralphd on macOS target (${TARGET_USER}@${TARGET_HOST})"
if [ "${is_loopback_target}" -eq 1 ]; then
  start_local_target
  verify_local_target
else
  start_remote_target
  verify_remote_target
fi

echo "✓ macOS target is provisioned"
echo "  host: ${TARGET_HOST}"
echo "  user: ${TARGET_USER}"
echo "  ssh port: ${TARGET_SSH_PORT}"
echo "  ralphd port: ${TARGET_RALPHD_PORT}"
echo "  fixture path: ${TARGET_PROJECT_DIR}"
