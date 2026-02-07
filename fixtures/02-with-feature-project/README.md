# With Feature Project

**Purpose**: Project with a feature defined, but no tasks yet

This fixture shows a project that has been initialized and has a feature
defined (e.g., "authentication"), but no tasks have been created yet.

## Usage

```bash
# Generate fixtures
cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures -- --nocapture

# Reset mock and use
just reset-mock
just dev-mock 02-with-feature-project
```

## Contents

- `.undetect-ralph/db/ralph.db` - SQLite database (1 feature, no tasks)
- `.undetect-ralph/CLAUDE.RALPH.md` - Template

## Progression

Shows state after AI agent has created a feature but before any tasks.
Next stage: 03-with-tasks-project
