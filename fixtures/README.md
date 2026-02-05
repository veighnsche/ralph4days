# Test Fixtures

Development fixtures for testing Ralph Loop.

## Available Fixtures

### Single Task Fixture

**Location**: `fixtures/single-task/`
**Purpose**: Minimal smoke test - one simple task
**Expected**: Complete in 1 iteration

**Task**: Write a hello world script

## Fixture Structure

All fixtures follow the Ralph project standard (see [SPEC-030](../.specs/030_RALPH_PROJECT_STANDARDS.md)):

```
fixture-name/
├── .ralph/
│   ├── prd.yaml         # Task list in structured YAML format (REQUIRED)
│   └── CLAUDE.RALPH.md  # Ralph-specific context for Claude (OPTIONAL)
├── README.md            # Human-readable fixture documentation
└── reset.sh             # Script to reset fixture to initial state
```

### PRD Format

Fixtures use `prd.yaml` (not `prd.md`). See [SPEC-035](../.specs/035_PRD_FORMAT.md) for full specification.

Example:
```yaml
schema_version: "1.0"
project:
  title: "Single Task Test Project"
  description: "Minimal test fixture for Ralph Loop"

tasks:
  - id: "task-001"
    title: "Write a hello world script"
    status: "pending"      # pending | in_progress | done | blocked | skipped
    priority: "medium"     # low | medium | high | critical
    tags: ["testing", "simple"]
```

### Runtime Files (auto-generated during execution)

```
.ralph/
├── progress.txt        # Iteration log (appended after each iteration)
└── learnings.txt       # Patterns learned (updated by Opus reviews)
```

Also at project root:
```
CLAUDE.md               # Injected from CLAUDE.RALPH.md during loop
CLAUDE.md.ralph-backup  # Backup of original CLAUDE.md (if existed)
```

## Usage

### Running Ralph with a Fixture

```bash
# Option 1: Launch with CLI arg
ralph --project fixtures/single-task

# Option 2: Launch and select via ProjectPicker
ralph
# Then browse to fixtures/single-task
```

### Resetting Fixtures

After running tests, reset fixtures to clean state:

```bash
# Reset all fixtures
just reset-fixtures

# Reset specific fixture
just reset-single-task

# Or run reset script directly
./fixtures/single-task/reset.sh
```

**What reset does**:
- Restores `prd.yaml` to initial state (all tasks → pending)
- Removes generated files (`progress.txt`, `learnings.txt`)
- Removes injected `CLAUDE.md` and backups
- Removes any files created by task execution
- Preserves fixture structure and original files

### Cleaning Without Resetting

To remove generated files without resetting task status:

```bash
just clean-fixtures
```

## Fixture Development

### Creating a New Fixture

1. Create directory: `fixtures/my-fixture/`
2. Add `.ralph/prd.yaml` with task definitions
3. (Optional) Add `.ralph/CLAUDE.RALPH.md` with context
4. Add `README.md` describing the fixture
5. Create `reset.sh` script (copy from `single-task/reset.sh`)
6. Test the fixture with Ralph

### Reset Script Template

```bash
#!/bin/bash
set -e

FIXTURE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RALPH_DIR="$FIXTURE_DIR/.ralph"

echo "Resetting fixture..."

# Reset prd.yaml
cat > "$RALPH_DIR/prd.yaml" <<'EOF'
# Your initial prd.yaml content here
EOF

# Remove generated files
rm -f "$RALPH_DIR/progress.txt"
rm -f "$RALPH_DIR/learnings.txt"
rm -f "$FIXTURE_DIR/CLAUDE.md"
rm -f "$FIXTURE_DIR/CLAUDE.md.ralph-backup"

# Remove fixture-specific generated files
rm -f "$FIXTURE_DIR/output.txt"

echo "✓ Fixture reset complete"
```

## Verification

After Ralph completes a fixture:

1. **Check task status**: All tasks should be `done` or `skipped` in `prd.yaml`
2. **Check outputs**: Expected files should exist
3. **Check logs**: `progress.txt` should have iteration history
4. **Reset and repeat**: Verify fixture can be reset and re-run

## Common Issues

### "No .ralph folder found"

The fixture directory doesn't have `.ralph/` subdirectory. Create it with:
```bash
mkdir -p fixtures/my-fixture/.ralph
```

### "prd.yaml not found"

Ralph requires `.ralph/prd.yaml`. The old `prd.md` format is deprecated. See SPEC-035 for migration.

### Fixture won't reset

Make sure `reset.sh` is executable:
```bash
chmod +x fixtures/my-fixture/reset.sh
```

## Future Fixtures (TODO)

Planned test fixtures:

- `multi-task` - Multiple sequential tasks with dependencies
- `parallel-tasks` - Tasks that can run concurrently
- `blocked-task` - Task that becomes blocked and must be unblocked
- `opus-review` - Fixture that triggers Opus review after N iterations
- `max-iterations` - Fixture that hits max iteration limit
- `stagnation` - Fixture that causes stagnation detection
- `complex-feature` - Real-world feature implementation scenario

## References

- [SPEC-030: Ralph Project Standards](../.specs/030_RALPH_PROJECT_STANDARDS.md)
- [SPEC-035: PRD Format Standard](../.specs/035_PRD_FORMAT.md)
- [Ralph Loop CLAUDE.md](../CLAUDE.md)
