# With Tasks Project

**Purpose**: Project with features and tasks (ready for loop)

This fixture shows a complete project ready for Ralph Loop to execute.
It has features defined and tasks created.

## Usage

```bash
# Generate fixtures
cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures -- --nocapture

# Reset mock and use
just reset-mock
just dev-mock 03-with-tasks-project
```

## Contents

- `.undetect-ralph/db/ralph.db` - SQLite database (2 features, 3 tasks)

## Tasks

1. **authentication/backend** - Implement login API (high priority)
2. **authentication/frontend** - Build login form (depends on #1)
3. **user-profile/frontend** - Create profile page

## Use Cases

- Test Ralph Loop execution
- Monkey testing with real task data
- Verify task dependency handling
- Test multi-feature projects
