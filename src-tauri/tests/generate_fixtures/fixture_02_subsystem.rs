use super::helpers::{initialize_project_for_fixture, open_fixture_db};
use sqlite_db::SubsystemInput;
use std::fs;
use std::path::PathBuf;

pub(crate) fn generate_fixture_02_desktop_feature() {
    println!("\n=== Generating fixture: 02-desktop-feature ===");

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("fixtures");

    fs::create_dir_all(&fixtures_dir).unwrap();
    let fixture_path = fixtures_dir.join("02-desktop-feature");

    if fixture_path.exists() {
        fs::remove_dir_all(&fixture_path).unwrap();
    }
    fs::create_dir_all(&fixture_path).unwrap();

    let readme = "# Desktop Subsystem

**Purpose**: Desktop stack project with a subsystem defined, but no tasks yet

This fixture shows a project initialized with Desktop stack that has a subsystem
defined (e.g., \"authentication\"), but no tasks have been created yet.

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
";

    fs::write(fixture_path.join("README.md"), readme).unwrap();

    // Initialize
    initialize_project_for_fixture(fixture_path.clone(), "Desktop Subsystem".to_owned(), true)
        .unwrap();

    // Add a subsystem
    let db = open_fixture_db(&fixture_path);
    db.create_subsystem(SubsystemInput {
        name: "authentication".to_owned(),
        display_name: "Authentication".to_owned(),
        acronym: "AUTH".to_owned(),
        ..Default::default()
    })
    .unwrap();

    println!(
        "✓ Created 02-desktop-feature fixture at: {}",
        fixture_path.display()
    );
}
