#!/bin/bash
# Reset the single-task fixture to its initial clean state
# Usage: ./reset.sh

set -e

FIXTURE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RALPH_DIR="$FIXTURE_DIR/.ralph"

echo "=== Resetting single-task fixture ==="
echo "Location: $FIXTURE_DIR"
echo ""

# Reset prd.yaml to initial state
echo "Resetting prd.yaml..."
cat > "$RALPH_DIR/prd.yaml" <<'EOF'
schema_version: "1.0"
project:
  title: "Single Task Test Project"
  description: "Minimal test fixture for Ralph Loop"
  created: "2026-02-05"

tasks:
  - id: "task-001"
    title: "Write a hello world script"
    description: "Create a simple script that prints 'Hello, World!' to stdout"
    status: "pending"
    priority: "medium"
    tags: ["testing", "simple"]
    created: "2026-02-05"
EOF

# Remove auto-generated files
echo "Removing generated files..."
rm -f "$RALPH_DIR/progress.txt"
rm -f "$RALPH_DIR/learnings.txt"

# Remove injected CLAUDE.md (and backup)
if [ -f "$FIXTURE_DIR/CLAUDE.md" ]; then
    echo "Removing injected CLAUDE.md..."
    rm -f "$FIXTURE_DIR/CLAUDE.md"
fi

if [ -f "$FIXTURE_DIR/CLAUDE.md.ralph-backup" ]; then
    echo "Removing CLAUDE.md.ralph-backup..."
    rm -f "$FIXTURE_DIR/CLAUDE.md.ralph-backup"
fi

# Remove any generated project files
echo "Removing generated project files..."
find "$FIXTURE_DIR" -type f ! -path "*/.ralph/*" ! -name "README.md" ! -name "reset.sh" -delete 2>/dev/null || true

# Show final state
echo ""
echo "=== Reset complete ==="
echo ""
echo "Current fixture structure:"
tree -a -L 2 "$FIXTURE_DIR" 2>/dev/null || find "$FIXTURE_DIR" -type f | sed "s|$FIXTURE_DIR|.|g"
echo ""
echo "✓ Fixture is ready for testing"
