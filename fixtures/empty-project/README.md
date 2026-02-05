# Empty Project

**Purpose**: Test project initialization from scratch

This fixture is intentionally empty (no `.undetect-ralph/` directory).
It's used to test the `initialize_ralph_project` command.

## Usage

```bash
# Generate fixtures first
cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures -- --nocapture

# This fixture is the BEFORE state - it cannot be used directly with dev-mock
# Use initialized-project fixture instead
```

## What Gets Created

When `initialize_ralph_project` is called on this directory:
- `.undetect-ralph/db/tasks.yaml` (empty - AI agents will add tasks)
- `.undetect-ralph/db/features.yaml` (empty - AI agents will add features)
- `.undetect-ralph/db/disciplines.yaml` (10 defaults for reference)
- `.undetect-ralph/db/metadata.yaml` (project info, no counters yet)
- `.undetect-ralph/CLAUDE.RALPH.md` (template)

See `initialized-project` fixture for the AFTER state.
