# Initialized Project

**Purpose**: Freshly initialized Ralph project with starter task

This fixture shows the state immediately after running `initialize_ralph_project`.
It has `.undetect-ralph/` structure with one starter task.

## Usage

```bash
# Generate fixtures
cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures -- --nocapture

# Reset mock from fixtures
just reset-mock

# Use this fixture
just dev-mock initialized-project
# OR
ralph --project mock/initialized-project
```

## Contents

- `.undetect-ralph/db/tasks.yaml` - 1 starter task (ID=1, "Replace this with your first task")
- `.undetect-ralph/db/features.yaml` - "setup" feature
- `.undetect-ralph/db/disciplines.yaml` - 10 default disciplines
- `.undetect-ralph/db/metadata.yaml` - Project metadata + counters
- `.undetect-ralph/CLAUDE.RALPH.md` - Template for context

## Expected Behavior

- Loop should start with 1 pending task
- User can replace the starter task with their actual tasks
- Ready for monkey testing and manual exploration
