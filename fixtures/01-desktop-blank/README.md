# Desktop Blank

**Purpose**: Freshly initialized Ralph project with Desktop stack (empty, ready for AI agents)

This fixture shows the state immediately after running `initialize_ralph_project` with stack 2 (Desktop).
It has `.undetect-ralph/` structure with empty tasks/features (AI agents will populate).

## Usage

```bash
# Generate fixtures
cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures -- --nocapture

# Reset mock from fixtures
just reset-mock

# Use this fixture
just dev-mock 01-desktop-blank
# OR
ralph --project mock/01-desktop-blank
```

## Contents

- `.undetect-ralph/db/ralph.db` - SQLite database (empty tasks/features, 8 Desktop disciplines)
- `.undetect-ralph/images/disciplines/` - Discipline portrait images
- `.undetect-ralph/CLAUDE.RALPH.md` - Template for context

## Expected Behavior

- Execution sequence starts with no tasks (clean slate)
- AI agents will create tasks and features as needed
- Disciplines provide Desktop stack defaults (Frontend, Backend, Data, etc.)
- Ready for AI-driven development workflow
