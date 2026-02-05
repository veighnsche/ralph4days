#!/usr/bin/env bash
# Reset script for elaborate-prd fixture
# This fixture is read-only, so reset just cleans generated files

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Resetting elaborate-prd fixture..."

# Remove generated files
rm -f "$SCRIPT_DIR/.ralph/progress.txt"
rm -f "$SCRIPT_DIR/.ralph/learnings.txt"
rm -f "$SCRIPT_DIR/CLAUDE.md"
rm -f "$SCRIPT_DIR/CLAUDE.md.ralph-backup"

echo "✓ elaborate-prd fixture reset complete"
