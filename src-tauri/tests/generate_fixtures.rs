//! Fixture generator tests
//!
//! These tests GENERATE the fixtures in fixtures/ directory.
//! Run with: cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures -- --nocapture
//!
//! Fixtures use .undetect-ralph/ (not .ralph/) so they're not detected as Ralph projects.
//! The mock workflow (just dev-mock) renames .undetect-ralph/ to .ralph/ when copying.

use std::fs;
use std::path::PathBuf;

/// Helper to initialize a project (creates .undetect-ralph/ for fixtures)
fn initialize_project_for_fixture(
    path: PathBuf,
    project_title: String,
    use_undetect: bool,
) -> Result<(), String> {
    use yaml_db::{
        DisciplinesFile, FeaturesFile, MetadataFile, ProjectMetadata, TasksFile,
    };

    if !path.exists() {
        return Err(format!("Directory not found: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("Not a directory: {}", path.display()));
    }

    // Create .ralph/ or .undetect-ralph/ directory
    let ralph_dir = if use_undetect {
        path.join(".undetect-ralph")
    } else {
        path.join(".ralph")
    };

    if ralph_dir.exists() {
        return Err(format!(
            "{} already exists at {}",
            if use_undetect {
                ".undetect-ralph/"
            } else {
                ".ralph/"
            },
            path.display()
        ));
    }

    fs::create_dir(&ralph_dir)
        .map_err(|e| format!("Failed to create ralph directory: {}", e))?;

    // Create db/ directory
    let db_path = ralph_dir.join("db");
    fs::create_dir(&db_path).map_err(|e| format!("Failed to create db/ directory: {}", e))?;

    let now = chrono::Utc::now().format("%Y-%m-%d").to_string();

    // Create empty tasks.yaml (AI agents will add tasks)
    let tasks_file = TasksFile::new(db_path.join("tasks.yaml"));
    tasks_file.save()?;

    // Create empty features.yaml (AI agents will add features)
    let features_file = FeaturesFile::new(db_path.join("features.yaml"));
    features_file.save()?;

    // Create disciplines.yaml with defaults
    let mut disciplines = DisciplinesFile::new(db_path.join("disciplines.yaml"));
    disciplines.initialize_defaults();
    disciplines.save()?;

    // Create metadata.yaml (no counters - no tasks yet)
    let mut metadata = MetadataFile::new(db_path.join("metadata.yaml"));
    metadata.project = ProjectMetadata {
        title: project_title.clone(),
        description: Some("Add project description here".to_string()),
        created: Some(now),
    };
    // No need to rebuild counters - empty task list
    metadata.save()?;

    // Create CLAUDE.RALPH.md template
    let claude_path = ralph_dir.join("CLAUDE.RALPH.md");
    let claude_template = format!(
        r#"# {} - Ralph Context

## Project Overview

Add context about this project that Claude should know when working on it.

## Architecture

Describe the architecture, tech stack, and key components.

## Coding Standards

- List any coding conventions
- Style guides
- Best practices

## Important Notes

- Any gotchas or things to watch out for
- Known issues or limitations
- Dependencies or external services
"#,
        project_title
    );

    fs::write(&claude_path, claude_template)
        .map_err(|e| format!("Failed to create CLAUDE.RALPH.md: {}", e))?;

    Ok(())
}

/// Generate empty-project fixture (before initialization)
#[test]
fn generate_fixture_empty_project() {
    println!("\n=== Generating fixture: empty-project ===");

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("fixtures");

    // Ensure fixtures/ directory exists
    fs::create_dir_all(&fixtures_dir).unwrap();

    let fixture_path = fixtures_dir.join("empty-project");

    // Clean and recreate
    if fixture_path.exists() {
        fs::remove_dir_all(&fixture_path).unwrap();
    }
    fs::create_dir_all(&fixture_path).unwrap();

    // Just a README - no .undetect-ralph/
    let readme = r#"# Empty Project

**Purpose**: Test project initialization from scratch

This fixture is intentionally empty (no `.undetect-ralph/` directory).
It's used to test the `initialize_ralph_project` command.

## Usage

```bash
# Generate fixtures first
cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures -- --nocapture

# This fixture is the BEFORE state - it cannot be used directly with dev-mock
# Use initialized-project fixture instead
```

## What Gets Created

When `initialize_ralph_project` is called on this directory:
- `.undetect-ralph/db/tasks.yaml` (empty - AI agents will add tasks)
- `.undetect-ralph/db/features.yaml` (empty - AI agents will add features)
- `.undetect-ralph/db/disciplines.yaml` (10 defaults for reference)
- `.undetect-ralph/db/metadata.yaml` (project info, no counters yet)
- `.undetect-ralph/CLAUDE.RALPH.md` (template)

See `initialized-project` fixture for the AFTER state.
"#;

    fs::write(fixture_path.join("README.md"), readme).unwrap();

    println!("✓ Created empty-project fixture at: {}", fixture_path.display());
}

/// Generate initialized-project fixture (after initialization)
#[test]
fn generate_fixture_initialized_project() {
    println!("\n=== Generating fixture: initialized-project ===");

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("fixtures");

    // Ensure fixtures/ directory exists
    fs::create_dir_all(&fixtures_dir).unwrap();

    let fixture_path = fixtures_dir.join("initialized-project");

    // Clean and recreate
    if fixture_path.exists() {
        fs::remove_dir_all(&fixture_path).unwrap();
    }
    fs::create_dir_all(&fixture_path).unwrap();

    // Add README
    let readme = r#"# Initialized Project

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

- `.undetect-ralph/db/tasks.yaml` - Empty (AI agents will add tasks)
- `.undetect-ralph/db/features.yaml` - Empty (AI agents will add features)
- `.undetect-ralph/db/disciplines.yaml` - 10 default disciplines (reference)
- `.undetect-ralph/db/metadata.yaml` - Project metadata (no counters yet)
- `.undetect-ralph/CLAUDE.RALPH.md` - Template for context

## Expected Behavior

- Loop starts with no tasks (clean slate)
- AI agents will create tasks and features as needed
- Disciplines provide defaults for common categories
- Ready for AI-driven development workflow
"#;

    fs::write(fixture_path.join("README.md"), readme).unwrap();

    // Initialize with .undetect-ralph/
    initialize_project_for_fixture(
        fixture_path.clone(),
        "Initialized Project".to_string(),
        true, // use .undetect-ralph
    )
    .unwrap();

    println!("✓ Created initialized-project fixture at: {}", fixture_path.display());
    println!("\n=== Contents ===");

    let db_path = fixture_path.join(".undetect-ralph/db");
    println!("\n--- tasks.yaml ---");
    println!("{}", fs::read_to_string(db_path.join("tasks.yaml")).unwrap());

    println!("\n--- features.yaml ---");
    println!("{}", fs::read_to_string(db_path.join("features.yaml")).unwrap());

    println!("\n--- metadata.yaml ---");
    println!("{}", fs::read_to_string(db_path.join("metadata.yaml")).unwrap());
}

/// Generate all fixtures
/// Note: Calls test functions directly, so run with --test-threads=1 to avoid conflicts
#[test]
fn generate_all_fixtures() {
    println!("\n========================================");
    println!("GENERATING ALL FIXTURES");
    println!("========================================");

    // Ensure fixtures directory exists before generating any fixtures
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("fixtures");
    fs::create_dir_all(&fixtures_dir).unwrap();

    generate_fixture_empty_project();
    generate_fixture_initialized_project();

    println!("\n========================================");
    println!("✅ ALL FIXTURES GENERATED");
    println!("========================================");
    println!("\nNext steps:");
    println!("  1. Review generated fixtures in fixtures/");
    println!("  2. Run: just reset-mock");
    println!("  3. Test with: just dev-mock initialized-project");
    println!("  4. Commit fixtures to git");
}
