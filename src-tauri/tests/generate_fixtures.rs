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

/// Generate 00-empty-project fixture (before initialization)
#[test]
fn generate_fixture_00_empty_project() {
    println!("\n=== Generating fixture: 00-empty-project ===");

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("fixtures");

    // Ensure fixtures/ directory exists
    fs::create_dir_all(&fixtures_dir).unwrap();

    let fixture_path = fixtures_dir.join("00-empty-project");

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

    println!("✓ Created 00-empty-project fixture at: {}", fixture_path.display());
}

/// Generate 01-initialized-project fixture (after initialization)
#[test]
fn generate_fixture_01_initialized_project() {
    println!("\n=== Generating fixture: 01-initialized-project ===");

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("fixtures");

    // Ensure fixtures/ directory exists
    fs::create_dir_all(&fixtures_dir).unwrap();

    let fixture_path = fixtures_dir.join("01-initialized-project");

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

    println!("✓ Created 01-initialized-project fixture at: {}", fixture_path.display());
}

/// Generate 02-with-feature-project fixture (has feature, no tasks yet)
#[test]
fn generate_fixture_02_with_feature() {
    use yaml_db::FeaturesFile;

    println!("\n=== Generating fixture: 02-with-feature-project ===");

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("fixtures");

    fs::create_dir_all(&fixtures_dir).unwrap();
    let fixture_path = fixtures_dir.join("02-with-feature-project");

    if fixture_path.exists() {
        fs::remove_dir_all(&fixture_path).unwrap();
    }
    fs::create_dir_all(&fixture_path).unwrap();

    // Add README
    let readme = r#"# With Feature Project

**Purpose**: Project with a feature defined, but no tasks yet

This fixture shows a project that has been initialized and has a feature
defined (e.g., "authentication"), but no tasks have been created yet.

## Usage

```bash
# Generate fixtures
cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures generate_all_fixtures -- --nocapture --test-threads=1

# Reset mock and use
just reset-mock
just dev-mock 02-with-feature-project
```

## Contents

- `.undetect-ralph/db/tasks.yaml` - Empty
- `.undetect-ralph/db/features.yaml` - 1 feature defined
- `.undetect-ralph/db/disciplines.yaml` - 10 defaults
- `.undetect-ralph/db/metadata.yaml` - Project metadata

## Progression

Shows state after AI agent has created a feature but before any tasks.
Next stage: 03-with-tasks-project
"#;

    fs::write(fixture_path.join("README.md"), readme).unwrap();

    // Initialize
    initialize_project_for_fixture(
        fixture_path.clone(),
        "Feature Project".to_string(),
        true,
    )
    .unwrap();

    // Add a feature
    let db_path = fixture_path.join(".undetect-ralph/db");
    let mut features_file = FeaturesFile::new(db_path.join("features.yaml"));
    features_file.load().unwrap();
    features_file.ensure_feature_exists("authentication").unwrap();
    features_file.save().unwrap();

    println!("✓ Created 02-with-feature-project fixture at: {}", fixture_path.display());
}

/// Generate 03-with-tasks-project fixture (has feature + tasks)
#[test]
fn generate_fixture_03_with_tasks() {
    use yaml_db::{FeaturesFile, Priority, Task, TaskStatus, TasksFile};

    println!("\n=== Generating fixture: 03-with-tasks-project ===");

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("fixtures");

    fs::create_dir_all(&fixtures_dir).unwrap();
    let fixture_path = fixtures_dir.join("03-with-tasks-project");

    if fixture_path.exists() {
        fs::remove_dir_all(&fixture_path).unwrap();
    }
    fs::create_dir_all(&fixture_path).unwrap();

    // Add README
    let readme = r#"# With Tasks Project

**Purpose**: Project with features and tasks (ready for loop)

This fixture shows a complete project ready for Ralph Loop to execute.
It has features defined and tasks created.

## Usage

```bash
# Generate fixtures
cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures generate_all_fixtures -- --nocapture --test-threads=1

# Reset mock and use
just reset-mock
just dev-mock 03-with-tasks-project
```

## Contents

- `.undetect-ralph/db/tasks.yaml` - 3 tasks across 2 features
- `.undetect-ralph/db/features.yaml` - 2 features
- `.undetect-ralph/db/disciplines.yaml` - 10 defaults
- `.undetect-ralph/db/metadata.yaml` - Project metadata + counters

## Tasks

1. **authentication/backend** - Implement login API (high priority)
2. **authentication/frontend** - Build login form (depends on #1)
3. **user-profile/frontend** - Create profile page

## Use Cases

- Test Ralph Loop execution
- Monkey testing with real task data
- Verify task dependency handling
- Test multi-feature projects
"#;

    fs::write(fixture_path.join("README.md"), readme).unwrap();

    // Initialize
    initialize_project_for_fixture(
        fixture_path.clone(),
        "Tasks Project".to_string(),
        true,
    )
    .unwrap();

    let db_path = fixture_path.join(".undetect-ralph/db");

    // Add features
    let mut features_file = FeaturesFile::new(db_path.join("features.yaml"));
    features_file.load().unwrap();
    features_file.ensure_feature_exists("authentication").unwrap();
    features_file.ensure_feature_exists("user-profile").unwrap();
    features_file.save().unwrap();

    // Add tasks
    let mut tasks_file = TasksFile::new(db_path.join("tasks.yaml"));
    tasks_file.load().unwrap();

    let now = chrono::Utc::now().format("%Y-%m-%d").to_string();

    tasks_file.add_task(Task {
        id: 1,
        feature: "authentication".to_string(),
        discipline: "backend".to_string(),
        title: "Implement login API".to_string(),
        description: Some("Create REST API endpoints for user authentication".to_string()),
        status: TaskStatus::Pending,
        priority: Some(Priority::High),
        tags: vec!["api".to_string(), "security".to_string()],
        depends_on: vec![],
        blocked_by: None,
        created: Some(now.clone()),
        updated: None,
        completed: None,
        acceptance_criteria: vec![
            "POST /login endpoint works".to_string(),
            "Returns JWT token".to_string(),
        ],
    });

    tasks_file.add_task(Task {
        id: 2,
        feature: "authentication".to_string(),
        discipline: "frontend".to_string(),
        title: "Build login form".to_string(),
        description: Some("Create UI for user login".to_string()),
        status: TaskStatus::Pending,
        priority: Some(Priority::Medium),
        tags: vec!["ui".to_string()],
        depends_on: vec![1],
        blocked_by: None,
        created: Some(now.clone()),
        updated: None,
        completed: None,
        acceptance_criteria: vec!["Form validates input".to_string()],
    });

    tasks_file.add_task(Task {
        id: 3,
        feature: "user-profile".to_string(),
        discipline: "frontend".to_string(),
        title: "Create profile page".to_string(),
        description: Some("User profile display and editing".to_string()),
        status: TaskStatus::Pending,
        priority: Some(Priority::Low),
        tags: vec!["ui".to_string()],
        depends_on: vec![],
        blocked_by: None,
        created: Some(now),
        updated: None,
        completed: None,
        acceptance_criteria: vec!["Shows user info".to_string()],
    });

    tasks_file.save().unwrap();

    // Update metadata with counters
    use yaml_db::MetadataFile;
    let mut metadata = MetadataFile::new(db_path.join("metadata.yaml"));
    metadata.load().unwrap();
    metadata.rebuild_counters(tasks_file.get_all());
    metadata.save().unwrap();

    println!("✓ Created 03-with-tasks-project fixture at: {}", fixture_path.display());
}

/// Print fixture contents
fn print_fixture_contents(fixture_path: &PathBuf) {
    let db_path = fixture_path.join(".undetect-ralph/db");

    println!("\n=== Contents ===");
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

    generate_fixture_00_empty_project();
    generate_fixture_01_initialized_project();
    generate_fixture_02_with_feature();
    generate_fixture_03_with_tasks();

    println!("\n========================================");
    println!("✅ ALL 4 FIXTURES GENERATED");
    println!("========================================");
    println!("\nFixture progression:");
    println!("  00-empty-project         → Just README, no .undetect-ralph/");
    println!("  01-initialized-project   → Empty tasks/features");
    println!("  02-with-feature-project  → Has 1 feature, no tasks");
    println!("  03-with-tasks-project    → Has 2 features, 3 tasks");
    println!("\nNext steps:");
    println!("  1. Review generated fixtures in fixtures/");
    println!("  2. Run: just reset-mock");
    println!("  3. Test with: just dev-mock 03-with-tasks-project");
    println!("  4. Commit fixtures to git");
}
