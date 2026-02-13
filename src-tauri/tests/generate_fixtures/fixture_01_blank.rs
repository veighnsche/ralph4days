use super::helpers::initialize_project_for_fixture;
use std::fs;
use std::path::PathBuf;

pub(crate) fn generate_fixture_01_desktop_blank() {
    println!("\n=== Generating fixture: 01-desktop-blank ===");

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("fixtures");

    // Ensure fixtures/ directory exists
    fs::create_dir_all(&fixtures_dir).unwrap();

    let fixture_path = fixtures_dir.join("01-desktop-blank");

    // Clean and recreate
    if fixture_path.exists() {
        fs::remove_dir_all(&fixture_path).unwrap();
    }
    fs::create_dir_all(&fixture_path).unwrap();

    let readme = "# Desktop Blank

**Purpose**: Freshly initialized Ralph project with Desktop stack (empty, ready for AI agents)

This fixture shows the state immediately after running `initialize_ralph_project` with stack 2 (Desktop).
It has `.undetect-ralph/` structure with empty tasks/subsystems (AI agents will populate).

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

- `.undetect-ralph/db/ralph.db` - SQLite database (empty tasks/subsystems, 8 Desktop disciplines)
- `.undetect-ralph/images/disciplines/` - Discipline portrait images
- `.undetect-ralph/CLAUDE.RALPH.md` - Template for context

## Expected Behavior

- Execution sequence starts with no tasks (clean slate)
- AI agents will create tasks and subsystems as needed
- Disciplines provide Desktop stack defaults (Frontend, Backend, Data, etc.)
- Ready for AI-driven development workflow
";

    fs::write(fixture_path.join("README.md"), readme).unwrap();

    // Initialize with .undetect-ralph/
    initialize_project_for_fixture(
        fixture_path.clone(),
        "Desktop Blank".to_owned(),
        true, // use .undetect-ralph
    )
    .unwrap();

    println!(
        "✓ Created 01-desktop-blank fixture at: {}",
        fixture_path.display()
    );
}
