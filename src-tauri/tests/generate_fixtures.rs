//! Fixture generator tests
//!
//! These tests GENERATE fixtures under `fixtures/`.
//! Run with:
//! `cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures -- --nocapture`
//!
//! Fixtures use `.undetect-ralph/` so they are not auto-detected as live Ralph projects.
//! The mock workflow (`just dev-mock`) renames `.undetect-ralph/` to `.ralph/`.

mod generate_fixtures {
    pub(crate) mod fixture_01_blank;
    pub(crate) mod fixture_02_subsystem;
    pub(crate) mod fixture_03_tasks;
    pub(crate) mod fixture_04_dev;
    pub(crate) mod fixture_05_templates;
    pub(crate) mod helpers;
}
mod test_support;

use std::fs;
use std::path::PathBuf;

#[test]
fn generate_fixture_00_empty_project() {
    println!("\n=== Generating fixture: 00-empty-project ===");

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("fixtures");
    fs::create_dir_all(&fixtures_dir).unwrap();

    let fixture_path = fixtures_dir.join("00-empty-project");
    if fixture_path.exists() {
        fs::remove_dir_all(&fixture_path).unwrap();
    }
    fs::create_dir_all(&fixture_path).unwrap();

    let readme = "# Empty Project

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
";
    fs::write(fixture_path.join("README.md"), readme).unwrap();

    println!(
        "✓ Created 00-empty-project fixture at: {}",
        fixture_path.display()
    );
}

#[test]
fn generate_fixture_01_desktop_blank() {
    generate_fixtures::fixture_01_blank::generate_fixture_01_desktop_blank();
}

#[test]
fn generate_fixture_02_desktop_feature() {
    generate_fixtures::fixture_02_subsystem::generate_fixture_02_desktop_feature();
}

#[test]
fn generate_fixture_03_desktop_tasks() {
    generate_fixtures::fixture_03_tasks::generate_fixture_03_desktop_tasks();
}

#[test]
fn generate_fixture_04_desktop_dev() {
    generate_fixtures::fixture_04_dev::generate_fixture_04_desktop_dev();
}

#[test]
fn generate_fixture_05_desktop_templates() {
    generate_fixtures::fixture_05_templates::generate_fixture_05_desktop_templates();
}

/// Generate all fixtures.
/// Note: Calls fixture test fns directly, so run with --test-threads=1.
/// Run explicitly with:
/// `cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures generate_all_fixtures -- --ignored --nocapture --test-threads=1`
#[test]
#[ignore = "Calls test functions directly, run with --test-threads=1 to avoid conflicts"]
fn generate_all_fixtures() {
    println!("\n========================================");
    println!("GENERATING ALL FIXTURES");
    println!("========================================");

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("fixtures");
    fs::create_dir_all(&fixtures_dir).unwrap();

    generate_fixture_00_empty_project();
    generate_fixture_01_desktop_blank();
    generate_fixture_02_desktop_feature();
    generate_fixture_03_desktop_tasks();
    generate_fixture_04_desktop_dev();
    generate_fixture_05_desktop_templates();

    println!("\n========================================");
    println!("ALL 6 FIXTURES GENERATED");
    println!("========================================");
    println!("\nFixture progression:");
    println!("  00-empty-project     → Just README, no .undetect-ralph/");
    println!("  01-desktop-blank     → Desktop stack, empty tasks/subsystems, SQLite DB + images");
    println!("  02-desktop-feature   → Desktop stack, 1 subsystem, no tasks");
    println!("  03-desktop-tasks     → Desktop stack, 2 subsystems, 3 tasks");
    println!("  04-desktop-dev       → Desktop stack, 5 subsystems, 20 tasks (comprehensive)");
    println!(
        "  05-desktop-templates → Desktop stack, routine task templates + pulled runtime tasks"
    );
    println!("\nNext steps:");
    println!("  1. Review generated fixtures in fixtures/");
    println!("  2. Run: just reset-mock");
    println!("  3. Test with: just dev-mock 05-desktop-templates");
    println!("  4. Commit fixtures to git");
}
