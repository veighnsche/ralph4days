#!/bin/bash
# Manual test script for 3-tier task ID system

set -e

FIXTURE_PATH="$(pwd)/fixtures/3-tier-tasks"

echo "Testing 3-tier task ID system..."
echo "Fixture path: $FIXTURE_PATH"

# Verify fixture exists
if [ ! -f "$FIXTURE_PATH/.ralph/prd.yaml" ]; then
  echo "ERROR: Fixture prd.yaml not found"
  exit 1
fi

echo "✓ Fixture exists"

# Check original counters
echo ""
echo "Original counters in prd.yaml:"
grep -A 10 "_counters:" "$FIXTURE_PATH/.ralph/prd.yaml" | head -6

echo ""
echo "Original task IDs:"
grep "^  - id:" "$FIXTURE_PATH/.ralph/prd.yaml"

echo ""
echo "✓ All checks passed!"
echo ""
echo "To manually test task creation:"
echo "1. Run: ralph --project $FIXTURE_PATH"
echo "2. Click 'New Task' button in PRD header"
echo "3. Fill form: feature='onboarding', discipline='frontend', title='Add welcome screen'"
echo "4. Verify preview shows 'onboarding/frontend/1'"
echo "5. Click Create"
echo "6. Verify task appears in playlist with ID 'onboarding/frontend/1'"
echo "7. Check prd.yaml counters updated"
