#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
FIXTURES_DIR="$PROJECT_ROOT/fixtures"

fail() {
  echo "❌ Fixture verification failed: $1" >&2
  exit 1
}

require_dir() {
  local path="$1"
  [ -d "$path" ] || fail "Missing directory: $path"
}

require_file() {
  local path="$1"
  [ -f "$path" ] || fail "Missing file: $path"
}

require_sql_result_eq() {
  local db="$1"
  local sql="$2"
  local expected="$3"
  local actual
  actual="$(sqlite3 "$db" "$sql")"
  [ "$actual" = "$expected" ] || fail "DB check failed for $db. SQL: $sql | expected: $expected | actual: $actual"
}

echo "🔍 Verifying fixture structure..."
require_dir "$FIXTURES_DIR"
require_dir "$FIXTURES_DIR/00-empty-project"
require_dir "$FIXTURES_DIR/01-desktop-blank"
require_dir "$FIXTURES_DIR/02-desktop-feature"
require_dir "$FIXTURES_DIR/03-desktop-tasks"
require_dir "$FIXTURES_DIR/04-desktop-dev"

if [ -d "$FIXTURES_DIR/00-empty-project/.undetect-ralph" ]; then
  fail "00-empty-project must not contain .undetect-ralph/"
fi

for fixture in 01-desktop-blank 02-desktop-feature 03-desktop-tasks 04-desktop-dev; do
  db="$FIXTURES_DIR/$fixture/.undetect-ralph/db/ralph.db"
  require_file "$db"

  echo "🧪 Verifying DB: $fixture"
  require_sql_result_eq "$db" "PRAGMA integrity_check;" "ok"
  require_sql_result_eq "$db" "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='agent_sessions';" "1"
  require_sql_result_eq "$db" "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='agent_session_events';" "1"
  require_sql_result_eq "$db" "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='agent_session_transcript';" "1"
  require_sql_result_eq "$db" "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='task_signal_comments';" "1"
done

echo "✅ Fixture verification passed"
