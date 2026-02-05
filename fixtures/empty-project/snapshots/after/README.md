# Empty Project Fixture

**Purpose**: Test project initialization from scratch

This fixture contains only a README - no `.ralph/` directory. It's used to test the `initialize_ralph_project` command.

## Expected Behavior

When `initialize_ralph_project` is called on this directory:

1. Creates `.ralph/db/` directory structure
2. Creates `tasks.yaml` with one starter task
3. Creates `features.yaml` with "setup" feature
4. Creates `disciplines.yaml` with 10 default disciplines
5. Creates `metadata.yaml` with project info and counters
6. Creates `CLAUDE.RALPH.md` template

## Usage

```bash
# Via Ralph CLI (not implemented yet - would need UI)
ralph --init /path/to/project

# Via test
cargo test --manifest-path src-tauri/Cargo.toml test_initialize_project
```

## Verification

After initialization:
- `.ralph/db/tasks.yaml` should have 1 task with ID=1
- `.ralph/db/features.yaml` should have "setup" feature
- `.ralph/db/disciplines.yaml` should have 10 default disciplines
- `.ralph/db/metadata.yaml` should have project title and counters
- `.ralph/CLAUDE.RALPH.md` should exist
