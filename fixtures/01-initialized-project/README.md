# Initialized Project

**Purpose**: Freshly initialized Ralph project (empty, ready for AI agents)

This fixture shows the state immediately after running `initialize_ralph_project`.
It has `.undetect-ralph/` structure with empty tasks/features (AI agents will populate).

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

- `.undetect-ralph/db/ralph.db` - SQLite database (empty tasks/features, 10 default disciplines)
- `.undetect-ralph/CLAUDE.RALPH.md` - Template for context

## Expected Behavior

- Loop starts with no tasks (clean slate)
- AI agents will create tasks and features as needed
- Disciplines provide defaults for common categories
- Ready for AI-driven development workflow
