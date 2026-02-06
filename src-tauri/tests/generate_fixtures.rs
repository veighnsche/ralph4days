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
    use yaml_db::{DisciplinesFile, FeaturesFile, MetadataFile, ProjectMetadata, TasksFile};

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

    fs::create_dir(&ralph_dir).map_err(|e| format!("Failed to create ralph directory: {}", e))?;

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

    println!(
        "✓ Created 00-empty-project fixture at: {}",
        fixture_path.display()
    );
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

    println!(
        "✓ Created 01-initialized-project fixture at: {}",
        fixture_path.display()
    );
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
    initialize_project_for_fixture(fixture_path.clone(), "Feature Project".to_string(), true)
        .unwrap();

    // Add a feature
    let db_path = fixture_path.join(".undetect-ralph/db");
    let mut features_file = FeaturesFile::new(db_path.join("features.yaml"));
    features_file.load().unwrap();
    features_file
        .ensure_feature_exists("authentication", "AUTH")
        .unwrap();
    features_file.save().unwrap();

    println!(
        "✓ Created 02-with-feature-project fixture at: {}",
        fixture_path.display()
    );
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
    initialize_project_for_fixture(fixture_path.clone(), "Tasks Project".to_string(), true)
        .unwrap();

    let db_path = fixture_path.join(".undetect-ralph/db");

    // Add features
    let mut features_file = FeaturesFile::new(db_path.join("features.yaml"));
    features_file.load().unwrap();
    features_file
        .ensure_feature_exists("authentication", "AUTH")
        .unwrap();
    features_file
        .ensure_feature_exists("user-profile", "USPR")
        .unwrap();
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

    println!(
        "✓ Created 03-with-tasks-project fixture at: {}",
        fixture_path.display()
    );
}

/// Generate 04-dev-project fixture (comprehensive mid-progress project for UI development)
#[test]
fn generate_fixture_04_dev_project() {
    use yaml_db::{Feature, FeaturesFile, MetadataFile, Priority, Task, TaskStatus, TasksFile};

    println!("\n=== Generating fixture: 04-dev-project ===");

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("fixtures");

    fs::create_dir_all(&fixtures_dir).unwrap();
    let fixture_path = fixtures_dir.join("04-dev-project");

    if fixture_path.exists() {
        fs::remove_dir_all(&fixture_path).unwrap();
    }
    fs::create_dir_all(&fixture_path).unwrap();

    let readme = r#"# Dev Project — Bookmarks Manager

**Purpose**: Comprehensive mid-progress fixture exercising every frontend rendering path.

20 tasks across 5 features and 7 disciplines. Covers all status/priority combos,
dependency chains up to 3 deep, blocked_by reasons, 0–4 acceptance criteria,
and varied timestamps.

## Usage

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures generate_all_fixtures -- --nocapture --test-threads=1
just reset-mock
just dev-mock 04-dev-project
```

## What this exercises

- **TaskDetailSidebar**: all 5 status badges, all 4 priority badges, blocked_by alert,
  depends_on badges, acceptance criteria list, tags, created/updated/completed timestamps
- **PlaylistView**: blocked+skipped in "Issues", done section, in_progress NOW PLAYING, pending
- **FeaturesPage**: 5 features with varied completion %
- **DisciplinesPage**: 7 disciplines with tasks, 3 with 0 tasks
- **Filters**: 14 distinct tags, every status/priority combo, text search on titles+descriptions
- **TaskIdDisplay**: multiple feature+discipline acronym combos
"#;

    fs::write(fixture_path.join("README.md"), readme).unwrap();

    // Initialize project structure + default disciplines
    initialize_project_for_fixture(fixture_path.clone(), "Bookmarks Manager".to_string(), true)
        .unwrap();

    let db_path = fixture_path.join(".undetect-ralph/db");

    // --- Features ---
    let mut features_file = FeaturesFile::new(db_path.join("features.yaml"));
    features_file.load().unwrap();

    features_file.add(Feature {
        name: "bookmark-crud".to_string(),
        display_name: "Bookmark CRUD".to_string(),
        acronym: "BKMK".to_string(),
        description: Some("Core bookmark create, read, update, delete operations".to_string()),
        created: Some("2025-01-10".to_string()),
    });
    features_file.add(Feature {
        name: "collections".to_string(),
        display_name: "Collections".to_string(),
        acronym: "COLL".to_string(),
        description: Some("Organize bookmarks into named collections".to_string()),
        created: Some("2025-01-10".to_string()),
    });
    features_file.add(Feature {
        name: "search".to_string(),
        display_name: "Search".to_string(),
        acronym: "SRCH".to_string(),
        description: Some("Full-text search and filtering across bookmarks".to_string()),
        created: Some("2025-01-11".to_string()),
    });
    features_file.add(Feature {
        name: "import-export".to_string(),
        display_name: "Import Export".to_string(),
        acronym: "IMEX".to_string(),
        description: Some("Import from HTML, export to JSON".to_string()),
        created: Some("2025-01-11".to_string()),
    });
    features_file.add(Feature {
        name: "settings".to_string(),
        display_name: "Settings".to_string(),
        acronym: "STNG".to_string(),
        description: Some("User preferences and theme configuration".to_string()),
        created: Some("2025-01-12".to_string()),
    });

    features_file.save().unwrap();

    // --- Tasks ---
    let mut tasks_file = TasksFile::new(db_path.join("tasks.yaml"));
    tasks_file.load().unwrap();

    // Task 1: bookmark-crud / design / done / low
    tasks_file.add_task(Task {
        id: 1,
        feature: "bookmark-crud".to_string(),
        discipline: "design".to_string(),
        title: "Bookmark card layout".to_string(),
        description: Some(
            "Design the bookmark card component with favicon, title, URL, and action buttons"
                .to_string(),
        ),
        status: TaskStatus::Done,
        priority: Some(Priority::Low),
        tags: vec!["ui".to_string(), "design".to_string()],
        depends_on: vec![],
        blocked_by: None,
        created: Some("2025-01-10".to_string()),
        updated: None,
        completed: Some("2025-01-14".to_string()),
        acceptance_criteria: vec![
            "Card displays favicon, title, and truncated URL".to_string(),
            "Action buttons visible on hover".to_string(),
        ],
    });

    // Task 2: bookmark-crud / frontend / done / high
    tasks_file.add_task(Task {
        id: 2,
        feature: "bookmark-crud".to_string(),
        discipline: "frontend".to_string(),
        title: "Create bookmark form".to_string(),
        description: Some(
            "Implement the form to add new bookmarks with URL validation and auto-title fetch"
                .to_string(),
        ),
        status: TaskStatus::Done,
        priority: Some(Priority::High),
        tags: vec!["ui".to_string(), "forms".to_string()],
        depends_on: vec![],
        blocked_by: None,
        created: Some("2025-01-10".to_string()),
        updated: None,
        completed: Some("2025-01-16".to_string()),
        acceptance_criteria: vec![
            "Form accepts URL input with validation".to_string(),
            "Auto-fetches page title from URL".to_string(),
            "Shows loading state during fetch".to_string(),
        ],
    });

    // Task 3: bookmark-crud / backend / done / high
    tasks_file.add_task(Task {
        id: 3,
        feature: "bookmark-crud".to_string(),
        discipline: "backend".to_string(),
        title: "Bookmark localStorage storage".to_string(),
        description: Some(
            "Implement localStorage-based persistence layer for bookmarks with CRUD operations"
                .to_string(),
        ),
        status: TaskStatus::Done,
        priority: Some(Priority::High),
        tags: vec!["storage".to_string()],
        depends_on: vec![2],
        blocked_by: None,
        created: Some("2025-01-11".to_string()),
        updated: None,
        completed: Some("2025-01-18".to_string()),
        acceptance_criteria: vec![
            "Bookmarks persist across page reloads".to_string(),
            "CRUD operations update localStorage atomically".to_string(),
            "Handles storage quota errors gracefully".to_string(),
        ],
    });

    // Task 4: bookmark-crud / testing / in_progress / medium
    tasks_file.add_task(Task {
        id: 4,
        feature: "bookmark-crud".to_string(),
        discipline: "testing".to_string(),
        title: "Unit tests for bookmark CRUD".to_string(),
        description: Some(
            "Write comprehensive unit tests for create, read, update, and delete operations"
                .to_string(),
        ),
        status: TaskStatus::InProgress,
        priority: Some(Priority::Medium),
        tags: vec!["testing".to_string()],
        depends_on: vec![3],
        blocked_by: None,
        created: Some("2025-01-15".to_string()),
        updated: Some("2025-01-20".to_string()),
        completed: None,
        acceptance_criteria: vec![
            "Tests cover all CRUD operations".to_string(),
            "Edge cases for empty and malformed URLs tested".to_string(),
        ],
    });

    // Task 5: bookmark-crud / frontend / pending / medium
    tasks_file.add_task(Task {
        id: 5,
        feature: "bookmark-crud".to_string(),
        discipline: "frontend".to_string(),
        title: "Edit bookmark modal".to_string(),
        description: Some(
            "Modal dialog for editing existing bookmark title, URL, and notes".to_string(),
        ),
        status: TaskStatus::Pending,
        priority: Some(Priority::Medium),
        tags: vec!["ui".to_string(), "forms".to_string()],
        depends_on: vec![2],
        blocked_by: None,
        created: Some("2025-01-12".to_string()),
        updated: None,
        completed: None,
        acceptance_criteria: vec![
            "Modal pre-fills current bookmark data".to_string(),
            "Validates URL format on save".to_string(),
        ],
    });

    // Task 6: bookmark-crud / frontend / pending / medium
    tasks_file.add_task(Task {
        id: 6,
        feature: "bookmark-crud".to_string(),
        discipline: "frontend".to_string(),
        title: "Bulk delete bookmarks".to_string(),
        description: Some(
            "Multi-select bookmarks and delete them in batch with confirmation dialog".to_string(),
        ),
        status: TaskStatus::Pending,
        priority: Some(Priority::Medium),
        tags: vec!["ui".to_string()],
        depends_on: vec![3],
        blocked_by: None,
        created: Some("2025-01-12".to_string()),
        updated: None,
        completed: None,
        acceptance_criteria: vec![
            "Checkbox selection for multiple bookmarks".to_string(),
            "Confirmation dialog before bulk delete".to_string(),
        ],
    });

    // Task 7: bookmark-crud / security / pending / high
    tasks_file.add_task(Task {
        id: 7,
        feature: "bookmark-crud".to_string(),
        discipline: "security".to_string(),
        title: "URL input sanitization".to_string(),
        description: Some(
            "Sanitize and validate all URL inputs to prevent XSS and injection attacks".to_string(),
        ),
        status: TaskStatus::Pending,
        priority: Some(Priority::High),
        tags: vec!["security".to_string(), "validation".to_string()],
        depends_on: vec![],
        blocked_by: None,
        created: Some("2025-01-13".to_string()),
        updated: None,
        completed: None,
        acceptance_criteria: vec![
            "Rejects javascript: and data: URLs".to_string(),
            "Escapes HTML entities in bookmark titles".to_string(),
            "Validates URL format before storage".to_string(),
        ],
    });

    // Task 8: collections / backend / done / high
    tasks_file.add_task(Task {
        id: 8,
        feature: "collections".to_string(),
        discipline: "backend".to_string(),
        title: "Collection data model".to_string(),
        description: Some(
            "Define collection schema with name, color, icon, and bookmark references".to_string(),
        ),
        status: TaskStatus::Done,
        priority: Some(Priority::High),
        tags: vec!["storage".to_string(), "data-model".to_string()],
        depends_on: vec![],
        blocked_by: None,
        created: Some("2025-01-10".to_string()),
        updated: None,
        completed: Some("2025-01-15".to_string()),
        acceptance_criteria: vec![
            "Collection stores name, color, icon, and ordered bookmark IDs".to_string(),
            "Supports many-to-many relationship with bookmarks".to_string(),
        ],
    });

    // Task 9: collections / frontend / in_progress / high
    tasks_file.add_task(Task {
        id: 9,
        feature: "collections".to_string(),
        discipline: "frontend".to_string(),
        title: "Collection sidebar".to_string(),
        description: Some(
            "Sidebar component showing all collections with bookmark counts and quick navigation"
                .to_string(),
        ),
        status: TaskStatus::InProgress,
        priority: Some(Priority::High),
        tags: vec!["ui".to_string(), "navigation".to_string()],
        depends_on: vec![8],
        blocked_by: None,
        created: Some("2025-01-14".to_string()),
        updated: Some("2025-01-21".to_string()),
        completed: None,
        acceptance_criteria: vec![
            "Sidebar lists all collections with bookmark counts".to_string(),
            "Click collection filters bookmark list".to_string(),
            "Collapse/expand sidebar on mobile".to_string(),
        ],
    });

    // Task 10: collections / frontend / pending / medium
    tasks_file.add_task(Task {
        id: 10,
        feature: "collections".to_string(),
        discipline: "frontend".to_string(),
        title: "Drag-and-drop sorting".to_string(),
        description: Some(
            "Allow reordering bookmarks within a collection via drag-and-drop".to_string(),
        ),
        status: TaskStatus::Pending,
        priority: Some(Priority::Medium),
        tags: vec!["ui".to_string(), "interaction".to_string()],
        depends_on: vec![9],
        blocked_by: None,
        created: Some("2025-01-14".to_string()),
        updated: None,
        completed: None,
        acceptance_criteria: vec![
            "Drag handle on each bookmark card".to_string(),
            "Visual feedback during drag operation".to_string(),
        ],
    });

    // Task 11: collections / design / pending / none
    tasks_file.add_task(Task {
        id: 11,
        feature: "collections".to_string(),
        discipline: "design".to_string(),
        title: "Collection icons and colors".to_string(),
        description: Some(
            "Design the icon picker and color palette for collection customization".to_string(),
        ),
        status: TaskStatus::Pending,
        priority: None,
        tags: vec!["design".to_string()],
        depends_on: vec![],
        blocked_by: None,
        created: Some("2025-01-15".to_string()),
        updated: None,
        completed: None,
        acceptance_criteria: vec![],
    });

    // Task 12: collections / frontend / pending / low
    tasks_file.add_task(Task {
        id: 12,
        feature: "collections".to_string(),
        discipline: "frontend".to_string(),
        title: "Nested collections".to_string(),
        description: Some(
            "Support hierarchical collection nesting with tree view navigation".to_string(),
        ),
        status: TaskStatus::Pending,
        priority: Some(Priority::Low),
        tags: vec!["ui".to_string(), "navigation".to_string()],
        depends_on: vec![9, 8],
        blocked_by: None,
        created: Some("2025-01-15".to_string()),
        updated: None,
        completed: None,
        acceptance_criteria: vec![
            "Collections can contain sub-collections".to_string(),
            "Tree view shows hierarchy with expand/collapse".to_string(),
        ],
    });

    // Task 13: search / backend / pending / critical
    tasks_file.add_task(Task {
        id: 13,
        feature: "search".to_string(),
        discipline: "backend".to_string(),
        title: "Full-text search index".to_string(),
        description: Some(
            "Build an inverted index for full-text search across bookmark titles, URLs, and notes"
                .to_string(),
        ),
        status: TaskStatus::Pending,
        priority: Some(Priority::Critical),
        tags: vec!["search".to_string(), "performance".to_string()],
        depends_on: vec![3],
        blocked_by: None,
        created: Some("2025-01-11".to_string()),
        updated: None,
        completed: None,
        acceptance_criteria: vec![
            "Index updates on bookmark create/update/delete".to_string(),
            "Search returns results in under 50ms for 10k bookmarks".to_string(),
            "Supports partial word matching".to_string(),
            "Ranks results by relevance".to_string(),
        ],
    });

    // Task 14: search / frontend / blocked / high
    tasks_file.add_task(Task {
        id: 14,
        feature: "search".to_string(),
        discipline: "frontend".to_string(),
        title: "Search bar with autocomplete".to_string(),
        description: Some(
            "Search input with debounced autocomplete dropdown showing matching bookmarks"
                .to_string(),
        ),
        status: TaskStatus::Blocked,
        priority: Some(Priority::High),
        tags: vec!["ui".to_string(), "search".to_string()],
        depends_on: vec![13],
        blocked_by: Some("Waiting for search index (#13)".to_string()),
        created: Some("2025-01-12".to_string()),
        updated: None,
        completed: None,
        acceptance_criteria: vec![
            "Debounced input with 300ms delay".to_string(),
            "Dropdown shows top 5 matching bookmarks".to_string(),
            "Keyboard navigation through results".to_string(),
        ],
    });

    // Task 15: search / testing / pending / none
    tasks_file.add_task(Task {
        id: 15,
        feature: "search".to_string(),
        discipline: "testing".to_string(),
        title: "Search ranking tests".to_string(),
        description: Some(
            "Test search result ranking and relevance scoring with various query patterns"
                .to_string(),
        ),
        status: TaskStatus::Pending,
        priority: None,
        tags: vec!["testing".to_string(), "search".to_string()],
        depends_on: vec![13],
        blocked_by: None,
        created: Some("2025-01-13".to_string()),
        updated: None,
        completed: None,
        acceptance_criteria: vec![
            "Exact title matches rank highest".to_string(),
            "Partial matches rank by relevance score".to_string(),
        ],
    });

    // Task 16: import-export / backend / pending / high
    tasks_file.add_task(Task {
        id: 16,
        feature: "import-export".to_string(),
        discipline: "backend".to_string(),
        title: "HTML bookmark parser".to_string(),
        description: Some(
            "Parse Netscape bookmark HTML format exported by Chrome, Firefox, and Safari"
                .to_string(),
        ),
        status: TaskStatus::Pending,
        priority: Some(Priority::High),
        tags: vec!["parser".to_string(), "import".to_string()],
        depends_on: vec![],
        blocked_by: None,
        created: Some("2025-01-12".to_string()),
        updated: None,
        completed: None,
        acceptance_criteria: vec![
            "Parses Chrome bookmark export format".to_string(),
            "Parses Firefox bookmark export format".to_string(),
            "Preserves folder structure as collections".to_string(),
        ],
    });

    // Task 17: import-export / frontend / blocked / medium
    tasks_file.add_task(Task {
        id: 17,
        feature: "import-export".to_string(),
        discipline: "frontend".to_string(),
        title: "Import bookmarks UI".to_string(),
        description: Some(
            "File upload dialog for importing bookmarks with preview and conflict resolution"
                .to_string(),
        ),
        status: TaskStatus::Blocked,
        priority: Some(Priority::Medium),
        tags: vec!["ui".to_string(), "import".to_string()],
        depends_on: vec![16],
        blocked_by: Some("Needs HTML parser (#16)".to_string()),
        created: Some("2025-01-13".to_string()),
        updated: None,
        completed: None,
        acceptance_criteria: vec![
            "File picker accepts .html files".to_string(),
            "Preview shows bookmarks to import before confirming".to_string(),
        ],
    });

    // Task 18: import-export / frontend / pending / low
    tasks_file.add_task(Task {
        id: 18,
        feature: "import-export".to_string(),
        discipline: "frontend".to_string(),
        title: "Export to JSON".to_string(),
        description: Some(
            "Export all bookmarks and collections to a JSON file for backup".to_string(),
        ),
        status: TaskStatus::Pending,
        priority: Some(Priority::Low),
        tags: vec!["export".to_string()],
        depends_on: vec![3],
        blocked_by: None,
        created: Some("2025-01-13".to_string()),
        updated: None,
        completed: None,
        acceptance_criteria: vec![
            "Exports all bookmarks with metadata".to_string(),
            "Includes collection membership info".to_string(),
        ],
    });

    // Task 19: settings / docs / skipped / low
    tasks_file.add_task(Task {
        id: 19,
        feature: "settings".to_string(),
        discipline: "docs".to_string(),
        title: "Write settings documentation".to_string(),
        description: Some("Document all available settings and their default values".to_string()),
        status: TaskStatus::Skipped,
        priority: Some(Priority::Low),
        tags: vec!["docs".to_string()],
        depends_on: vec![],
        blocked_by: None,
        created: Some("2025-01-14".to_string()),
        updated: None,
        completed: None,
        acceptance_criteria: vec![],
    });

    // Task 20: settings / database / pending / none
    tasks_file.add_task(Task {
        id: 20,
        feature: "settings".to_string(),
        discipline: "database".to_string(),
        title: "Theme preference storage".to_string(),
        description: Some(
            "Store user theme preference (light/dark/system) in local database".to_string(),
        ),
        status: TaskStatus::Pending,
        priority: None,
        tags: vec!["storage".to_string()],
        depends_on: vec![],
        blocked_by: None,
        created: Some("2025-01-14".to_string()),
        updated: None,
        completed: None,
        acceptance_criteria: vec!["Persists theme preference across sessions".to_string()],
    });

    tasks_file.save().unwrap();

    // --- Rebuild metadata counters ---
    let mut metadata = MetadataFile::new(db_path.join("metadata.yaml"));
    metadata.load().unwrap();
    metadata.rebuild_counters(tasks_file.get_all());
    metadata.save().unwrap();

    // Print summary
    print_fixture_contents(&fixture_path);

    println!(
        "\n✓ Created 04-dev-project fixture at: {}",
        fixture_path.display()
    );
    println!("  5 features, 20 tasks (4 done, 2 in_progress, 11 pending, 2 blocked, 1 skipped)");
}

/// Print fixture contents
fn print_fixture_contents(fixture_path: &PathBuf) {
    let db_path = fixture_path.join(".undetect-ralph/db");

    println!("\n=== Contents ===");
    println!("\n--- tasks.yaml ---");
    println!(
        "{}",
        fs::read_to_string(db_path.join("tasks.yaml")).unwrap()
    );

    println!("\n--- features.yaml ---");
    println!(
        "{}",
        fs::read_to_string(db_path.join("features.yaml")).unwrap()
    );

    println!("\n--- metadata.yaml ---");
    println!(
        "{}",
        fs::read_to_string(db_path.join("metadata.yaml")).unwrap()
    );
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
    generate_fixture_04_dev_project();

    println!("\n========================================");
    println!("✅ ALL 5 FIXTURES GENERATED");
    println!("========================================");
    println!("\nFixture progression:");
    println!("  00-empty-project         → Just README, no .undetect-ralph/");
    println!("  01-initialized-project   → Empty tasks/features");
    println!("  02-with-feature-project  → Has 1 feature, no tasks");
    println!("  03-with-tasks-project    → Has 2 features, 3 tasks");
    println!("  04-dev-project           → 5 features, 20 tasks (comprehensive dev fixture)");
    println!("\nNext steps:");
    println!("  1. Review generated fixtures in fixtures/");
    println!("  2. Run: just reset-mock");
    println!("  3. Test with: just dev-mock 04-dev-project");
    println!("  4. Commit fixtures to git");
}
