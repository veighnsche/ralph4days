#!/usr/bin/env bash
set -euo pipefail

# Global reset script for Ralph test data
# Copies fixtures to mock/ and makes them detectable as Ralph projects

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
FIXTURES_DIR="$PROJECT_ROOT/fixtures"
MOCK_DIR="$PROJECT_ROOT/mock"

echo "🧹 Cleaning mock directory..."
rm -rf "$MOCK_DIR"
mkdir -p "$MOCK_DIR"

echo "📋 Copying fixtures to mock..."
cp -r "$FIXTURES_DIR"/* "$MOCK_DIR/"

echo "🔧 Renaming .undetect-ralph to .ralph..."
find "$MOCK_DIR" -type d -name ".undetect-ralph" | while read -r dir; do
    parent="$(dirname "$dir")"
    mv "$dir" "$parent/.ralph"
    echo "  ✓ ${parent#$MOCK_DIR/}"
done

echo "✅ Mock directory ready at: $MOCK_DIR"
echo ""
echo "Projects available:"
find "$MOCK_DIR" -type d -name ".ralph" -exec dirname {} \; | while read -r project; do
    echo "  - ${project#$MOCK_DIR/}"
done
