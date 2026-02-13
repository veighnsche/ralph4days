# Desktop Tasks

**Purpose**: Desktop stack project with subsystems, tasks, and a couple routine templates.

This fixture shows a complete project ready for Ralph task execution to run.
It has subsystems defined and tasks created, all using Desktop stack disciplines.

## Usage

```bash
# Generate fixtures
cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures -- --nocapture

# Reset mock and use
just reset-mock
just dev-mock 03-desktop-tasks
```

## Contents

- `.undetect-ralph/db/ralph.db` - SQLite database (2 subsystems, 3 tasks, 2 templates, 8 Desktop disciplines)
- `.undetect-ralph/images/disciplines/` - Discipline portrait images

## Tasks

1. **authentication/backend** - Implement login API (high priority)
2. **authentication/frontend** - Build login form (depends on #1)
3. **user-profile/frontend** - Create profile page

## Use Cases

- Test task execution sequencing
- Monkey testing with real task data
- Verify task dependency handling
- Test multi-subsystem projects
