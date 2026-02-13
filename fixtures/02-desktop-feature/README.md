# Desktop Subsystem

**Purpose**: Desktop stack project with a subsystem defined, but no tasks yet

This fixture shows a project initialized with Desktop stack that has a subsystem
defined (e.g., "authentication"), but no tasks have been created yet.

## Usage

```bash
# Generate fixtures
cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures -- --nocapture

# Reset mock and use
just reset-mock
just dev-mock 02-desktop-feature
```

## Contents

- `.undetect-ralph/db/ralph.db` - SQLite database (1 subsystem, no tasks, 8 Desktop disciplines)
- `.undetect-ralph/images/disciplines/` - Discipline portrait images
- `.undetect-ralph/CLAUDE.RALPH.md` - Template

## Progression

Shows state after AI agent has created a subsystem but before any tasks.
Next stage: 03-desktop-tasks
