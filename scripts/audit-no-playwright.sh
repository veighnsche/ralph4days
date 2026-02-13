#!/usr/bin/env bash
set -euo pipefail

SCRIPT_PATH="$0"

if ! command -v rg >/dev/null 2>&1; then
  echo "rg is required for this audit: install ripgrep" >&2
  exit 1
fi

RAW_MATCHES="$(rg -n --hidden -i "playwright" --glob '!node_modules/**' package.json justfile wdio.conf.js e2e-tauri src-tauri src scripts || true)"

FILTERED_MATCHES="$(printf '%s\n' "${RAW_MATCHES}" | rg -v -i "(package.json:.*\"audit:no-playwright\")|(^justfile:.*audit-no-playwright)|(^justfile:.*bun run audit:no-playwright)|(^scripts/audit-no-playwright\.sh:)" || true)"

if [ -n "${FILTERED_MATCHES}" ]; then
  echo "${FILTERED_MATCHES}" >&2
  echo "FAIL: Playwright references detected in active test/runtime surface." >&2
  echo "Search scope was: package.json, justfile, wdio.conf.js, e2e-tauri, src-tauri, src, scripts" >&2
  echo "Remove these references before running e2e. Script: ${SCRIPT_PATH}" >&2
  exit 1
fi

exit 0
