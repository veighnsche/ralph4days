# Empty Project

**Purpose**: Test project initialization from scratch

This fixture is intentionally empty (no `.undetect-ralph/` directory).
It's used to test the `initialize_ralph_project` command.

## Usage

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures -- --nocapture
```

## What Gets Created

When `initialize_ralph_project` is called on this directory:
- `.undetect-ralph/db/ralph.db` (SQLite database with schema, defaults, metadata)
- `.undetect-ralph/CLAUDE.RALPH.md` (template)

See `01-desktop-blank` fixture for the after state.
