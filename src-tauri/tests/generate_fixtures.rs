//! Fixture generator tests
//!
//! These tests GENERATE the fixtures in fixtures/ directory.
//! Run with: cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures -- --nocapture
//!
//! Fixtures use .undetect-ralph/ (not .ralph/) so they're not detected as Ralph projects.
//! The mock workflow (just dev-mock) renames .undetect-ralph/ to .ralph/ when copying.

use sqlite_db::{FeatureInput, FixedClock, SqliteDb};
use std::fs;
use std::path::{Path, PathBuf};

/// Helper to initialize a project (creates .undetect-ralph/ for fixtures)
fn initialize_project_for_fixture(
    path: PathBuf,
    project_title: String,
    use_undetect: bool,
) -> Result<(), String> {
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

    fs::create_dir(&ralph_dir).map_err(|e| format!("Failed to create ralph directory: {e}"))?;

    let db_dir = ralph_dir.join("db");
    fs::create_dir(&db_dir).map_err(|e| format!("Failed to create db/ directory: {e}"))?;

    // Create images directory
    let images_dir = ralph_dir.join("images").join("disciplines");
    let _ = fs::create_dir_all(&images_dir);

    // Create and initialize the SQLite database
    let db_path = db_dir.join("ralph.db");
    let db = SqliteDb::open_with_clock(&db_path, fixed_clock())?;
    for d in predefined_disciplines::get_disciplines_for_stack(2) {
        let skills_json = serde_json::to_string(&d.skills).unwrap_or_else(|_| "[]".to_owned());

        let image_path =
            if let Some(bytes) = predefined_disciplines::get_discipline_image(2, &d.name) {
                let rel = format!("images/disciplines/{}.png", d.name);
                let abs = ralph_dir.join(&rel);
                let _ = fs::write(&abs, bytes);
                Some(rel)
            } else {
                None
            };

        let crops_json = d.crops.as_ref().and_then(|c| serde_json::to_string(c).ok());

        db.create_discipline(sqlite_db::DisciplineInput {
            name: d.name,
            display_name: d.display_name,
            acronym: d.acronym,
            icon: d.icon,
            color: d.color,
            system_prompt: Some(d.system_prompt),
            skills: skills_json,
            conventions: Some(d.conventions),
            mcp_servers: "[]".to_owned(),
            image_path,
            crops: crops_json,
            description: None,
            image_prompt: None,
        })
        .map_err(|e| format!("Failed to seed discipline: {e}"))?;
    }
    db.initialize_metadata(
        project_title.clone(),
        Some("Add project description here".to_owned()),
    )?;

    let claude_path = ralph_dir.join("CLAUDE.RALPH.md");
    let claude_template = format!(
        "# {project_title} - Ralph Context

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
"
    );

    fs::write(&claude_path, claude_template)
        .map_err(|e| format!("Failed to create CLAUDE.RALPH.md: {e}"))?;

    Ok(())
}

fn fixed_clock() -> Box<dyn sqlite_db::Clock> {
    Box::new(FixedClock(
        chrono::NaiveDate::from_ymd_opt(2026, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc(),
    ))
}

fn open_fixture_db(fixture_path: &Path) -> SqliteDb {
    let db_path = fixture_path.join(".undetect-ralph/db/ralph.db");
    SqliteDb::open_with_clock(&db_path, fixed_clock()).unwrap()
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

    let readme = "# Empty Project

**Purpose**: Test project initialization from scratch

This fixture is intentionally empty (no `.undetect-ralph/` directory).
It's used to test the `initialize_ralph_project` command.

## Usage

```bash
# Generate fixtures first
cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures -- --nocapture

# This fixture is the BEFORE state - it cannot be used directly with dev-mock
# Use 01-desktop-blank fixture instead
```

## What Gets Created

When `initialize_ralph_project` is called on this directory:
- `.undetect-ralph/db/ralph.db` (SQLite database with schema, defaults, metadata)
- `.undetect-ralph/CLAUDE.RALPH.md` (template)

See `01-desktop-blank` fixture for the AFTER state.
";

    fs::write(fixture_path.join("README.md"), readme).unwrap();

    println!(
        "✓ Created 00-empty-project fixture at: {}",
        fixture_path.display()
    );
}

/// Generate 01-desktop-blank fixture (Desktop stack, no features/tasks)
#[test]
fn generate_fixture_01_desktop_blank() {
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

- Loop starts with no tasks (clean slate)
- AI agents will create tasks and features as needed
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

/// Generate 02-desktop-feature fixture (Desktop stack, has feature, no tasks yet)
#[test]
fn generate_fixture_02_desktop_feature() {
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

    let readme = "# Desktop Feature

**Purpose**: Desktop stack project with a feature defined, but no tasks yet

This fixture shows a project initialized with Desktop stack that has a feature
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

- `.undetect-ralph/db/ralph.db` - SQLite database (1 feature, no tasks, 8 Desktop disciplines)
- `.undetect-ralph/images/disciplines/` - Discipline portrait images
- `.undetect-ralph/CLAUDE.RALPH.md` - Template

## Progression

Shows state after AI agent has created a feature but before any tasks.
Next stage: 03-desktop-tasks
";

    fs::write(fixture_path.join("README.md"), readme).unwrap();

    // Initialize
    initialize_project_for_fixture(fixture_path.clone(), "Desktop Feature".to_owned(), true)
        .unwrap();

    // Add a feature
    let db = open_fixture_db(&fixture_path);
    db.create_feature(FeatureInput {
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

/// Generate 03-desktop-tasks fixture (Desktop stack, has features + tasks)
#[test]
fn generate_fixture_03_desktop_tasks() {
    use sqlite_db::TaskInput;

    println!("\n=== Generating fixture: 03-desktop-tasks ===");

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("fixtures");

    fs::create_dir_all(&fixtures_dir).unwrap();
    let fixture_path = fixtures_dir.join("03-desktop-tasks");

    if fixture_path.exists() {
        fs::remove_dir_all(&fixture_path).unwrap();
    }
    fs::create_dir_all(&fixture_path).unwrap();

    let readme = "# Desktop Tasks

**Purpose**: Desktop stack project with features and tasks (ready for loop)

This fixture shows a complete project ready for Ralph Loop to execute.
It has features defined and tasks created, all using Desktop stack disciplines.

## Usage

```bash
# Generate fixtures
cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures -- --nocapture

# Reset mock and use
just reset-mock
just dev-mock 03-desktop-tasks
```

## Contents

- `.undetect-ralph/db/ralph.db` - SQLite database (2 features, 3 tasks, 8 Desktop disciplines)
- `.undetect-ralph/images/disciplines/` - Discipline portrait images

## Tasks

1. **authentication/backend** - Implement login API (high priority)
2. **authentication/frontend** - Build login form (depends on #1)
3. **user-profile/frontend** - Create profile page

## Use Cases

- Test Ralph Loop execution
- Monkey testing with real task data
- Verify task dependency handling
- Test multi-feature projects
";

    fs::write(fixture_path.join("README.md"), readme).unwrap();

    // Initialize
    initialize_project_for_fixture(fixture_path.clone(), "Desktop Tasks".to_owned(), true).unwrap();

    let db = open_fixture_db(&fixture_path);

    // Add features
    db.create_feature(FeatureInput {
        name: "authentication".to_owned(),
        display_name: "Authentication".to_owned(),
        acronym: "AUTH".to_owned(),
        ..Default::default()
    })
    .unwrap();
    db.create_feature(FeatureInput {
        name: "user-profile".to_owned(),
        display_name: "User Profile".to_owned(),
        acronym: "USPR".to_owned(),
        ..Default::default()
    })
    .unwrap();

    // Add tasks
    db.create_task(TaskInput {
        feature: "authentication".to_owned(),
        discipline: "backend".to_owned(),
        title: "Implement login API".to_owned(),
        description: Some("Create REST API endpoints for user authentication".to_owned()),
        priority: Some(sqlite_db::Priority::High),
        tags: vec!["api".to_owned(), "security".to_owned()],
        depends_on: vec![],
        acceptance_criteria: Some(vec![
            "POST /login endpoint works".to_owned(),
            "Returns JWT token".to_owned(),
        ]),
        context_files: vec!["src/auth/mod.rs".to_owned(), "src/routes/auth.rs".to_owned()],
        output_artifacts: vec![
            "src/routes/auth.rs".to_owned(),
            "tests/auth_test.rs".to_owned(),
        ],
        hints: Some("Use bcrypt for password hashing, not SHA256. Check existing middleware pattern in src/middleware/".to_owned()),
        estimated_turns: Some(3),
        provenance: Some(sqlite_db::TaskProvenance::Agent),
    })
    .unwrap();

    db.add_comment(
        1,
        sqlite_db::CommentAuthor::Agent,
        Some(1),
        "First attempt failed: forgot to add JWT_SECRET to .env".to_owned(),
    )
    .unwrap();

    db.create_task(TaskInput {
        feature: "authentication".to_owned(),
        discipline: "frontend".to_owned(),
        title: "Build login form".to_owned(),
        description: Some("Create UI for user login".to_owned()),
        priority: Some(sqlite_db::Priority::Medium),
        tags: vec!["ui".to_owned()],
        depends_on: vec![1],
        acceptance_criteria: Some(vec!["Form validates input".to_owned()]),
        context_files: vec!["src/components/LoginForm.tsx".to_owned()],
        output_artifacts: vec![],
        hints: None,
        estimated_turns: Some(2),
        provenance: Some(sqlite_db::TaskProvenance::Human),
    })
    .unwrap();

    db.create_task(TaskInput {
        feature: "user-profile".to_owned(),
        discipline: "frontend".to_owned(),
        title: "Create profile page".to_owned(),
        description: Some("User profile display and editing".to_owned()),
        priority: Some(sqlite_db::Priority::Low),
        tags: vec!["ui".to_owned()],
        depends_on: vec![],
        acceptance_criteria: Some(vec!["Shows user info".to_owned()]),
        context_files: vec![],
        output_artifacts: vec![],
        hints: None,
        estimated_turns: None,
        provenance: Some(sqlite_db::TaskProvenance::System),
    })
    .unwrap();

    println!(
        "✓ Created 03-desktop-tasks fixture at: {}",
        fixture_path.display()
    );
}

/// Generate 04-desktop-dev fixture (comprehensive mid-progress project for UI development)
#[test]
fn generate_fixture_04_desktop_dev() {
    use sqlite_db::TaskInput;

    println!("\n=== Generating fixture: 04-desktop-dev ===");

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("fixtures");

    fs::create_dir_all(&fixtures_dir).unwrap();
    let fixture_path = fixtures_dir.join("04-desktop-dev");

    if fixture_path.exists() {
        fs::remove_dir_all(&fixture_path).unwrap();
    }
    fs::create_dir_all(&fixture_path).unwrap();

    let readme = "# Desktop Dev — Bookmarks Manager

**Purpose**: Comprehensive mid-progress fixture exercising every frontend rendering path.

20 tasks across 5 features and 7 disciplines. Desktop stack (stack 2) with discipline images.
Covers all status/priority combos, dependency chains up to 3 deep, blocked_by reasons,
0–4 acceptance criteria, and varied timestamps.

## Usage

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures -- --nocapture --test-threads=1
just reset-mock
just dev-mock 04-desktop-dev
```

## What this exercises

- **TaskDetailSidebar**: all 5 status badges, all 4 priority badges, blocked_by alert,
  depends_on badges, acceptance criteria list, tags, created/updated/completed timestamps
- **PlaylistView**: blocked+skipped in \"Issues\", done section, in_progress NOW PLAYING, pending
- **FeaturesPage**: 5 features with varied completion %
- **DisciplinesPage**: 8 Desktop stack disciplines with images
- **Filters**: 14 distinct tags, every status/priority combo, text search on titles+descriptions
- **TaskIdDisplay**: multiple feature+discipline acronym combos
";

    fs::write(fixture_path.join("README.md"), readme).unwrap();

    // Initialize project structure + default disciplines
    initialize_project_for_fixture(fixture_path.clone(), "Bookmarks Manager".to_owned(), true)
        .unwrap();

    let db = open_fixture_db(&fixture_path);

    // --- Features ---
    db.create_feature(FeatureInput {
        name: "bookmark-crud".to_owned(),
        display_name: "Bookmark CRUD".to_owned(),
        acronym: "BKMK".to_owned(),
        description: Some("Core bookmark create, read, update, delete operations".to_owned()),
        ..Default::default()
    })
    .unwrap();
    db.create_feature(FeatureInput {
        name: "collections".to_owned(),
        display_name: "Collections".to_owned(),
        acronym: "COLL".to_owned(),
        description: Some("Organize bookmarks into named collections".to_owned()),
        ..Default::default()
    })
    .unwrap();
    db.create_feature(FeatureInput {
        name: "search".to_owned(),
        display_name: "Search".to_owned(),
        acronym: "SRCH".to_owned(),
        description: Some("Full-text search and filtering across bookmarks".to_owned()),
        ..Default::default()
    })
    .unwrap();
    db.create_feature(FeatureInput {
        name: "import-export".to_owned(),
        display_name: "Import Export".to_owned(),
        acronym: "IMEX".to_owned(),
        description: Some("Import from HTML, export to JSON".to_owned()),
        ..Default::default()
    })
    .unwrap();
    db.create_feature(FeatureInput {
        name: "settings".to_owned(),
        display_name: "Settings".to_owned(),
        acronym: "STNG".to_owned(),
        description: Some("User preferences and theme configuration".to_owned()),
        ..Default::default()
    })
    .unwrap();

    // --- Tasks ---
    // All tasks created as pending via API, then we use execute_raw() to set varied statuses.

    // Task 1: bookmark-crud / design / done / low
    let _id1 = db
        .create_task(TaskInput {
            feature: "bookmark-crud".to_owned(),
            discipline: "frontend".to_owned(),
            title: "Bookmark card layout".to_owned(),
            description: Some(
                "Design the bookmark card component with favicon, title, URL, and action buttons"
                    .to_owned(),
            ),
            priority: Some(sqlite_db::Priority::Low),
            tags: vec!["ui".to_owned(), "design".to_owned()],
            depends_on: vec![],
            acceptance_criteria: Some(vec![
                "Card displays favicon, title, and truncated URL".to_owned(),
                "Action buttons visible on hover".to_owned(),
            ]),
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
        })
        .unwrap();

    // Task 2: bookmark-crud / frontend / done / high
    let _id2 = db
        .create_task(TaskInput {
            feature: "bookmark-crud".to_owned(),
            discipline: "frontend".to_owned(),
            title: "Create bookmark form".to_owned(),
            description: Some(
                "Implement the form to add new bookmarks with URL validation and auto-title fetch"
                    .to_owned(),
            ),
            priority: Some(sqlite_db::Priority::High),
            tags: vec!["ui".to_owned(), "forms".to_owned()],
            depends_on: vec![],
            acceptance_criteria: Some(vec![
                "Form accepts URL input with validation".to_owned(),
                "Auto-fetches page title from URL".to_owned(),
                "Shows loading state during fetch".to_owned(),
            ]),
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
        })
        .unwrap();

    // Task 3: bookmark-crud / backend / done / high (depends on 2)
    let _id3 = db
        .create_task(TaskInput {
            feature: "bookmark-crud".to_owned(),
            discipline: "backend".to_owned(),
            title: "Bookmark localStorage storage".to_owned(),
            description: Some(
                "Implement localStorage-based persistence layer for bookmarks with CRUD operations"
                    .to_owned(),
            ),
            priority: Some(sqlite_db::Priority::High),
            tags: vec!["storage".to_owned()],
            depends_on: vec![2],
            acceptance_criteria: Some(vec![
                "Bookmarks persist across page reloads".to_owned(),
                "CRUD operations update localStorage atomically".to_owned(),
                "Handles storage quota errors gracefully".to_owned(),
            ]),
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
        })
        .unwrap();

    // Task 4: bookmark-crud / testing / in_progress / medium (depends on 3)
    let _id4 = db
        .create_task(TaskInput {
            feature: "bookmark-crud".to_owned(),
            discipline: "quality".to_owned(),
            title: "Unit tests for bookmark CRUD".to_owned(),
            description: Some(
                "Write comprehensive unit tests for create, read, update, and delete operations"
                    .to_owned(),
            ),
            priority: Some(sqlite_db::Priority::Medium),
            tags: vec!["testing".to_owned()],
            depends_on: vec![3],
            acceptance_criteria: Some(vec![
                "Tests cover all CRUD operations".to_owned(),
                "Edge cases for empty and malformed URLs tested".to_owned(),
            ]),
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
        })
        .unwrap();

    // Tasks 5-20: remaining tasks (all pending by default)
    let _id5 = db
        .create_task(TaskInput {
            feature: "bookmark-crud".to_owned(),
            discipline: "frontend".to_owned(),
            title: "Edit bookmark modal".to_owned(),
            description: Some(
                "Modal dialog for editing existing bookmark title, URL, and notes".to_owned(),
            ),
            priority: Some(sqlite_db::Priority::Medium),
            tags: vec!["ui".to_owned(), "forms".to_owned()],
            depends_on: vec![2],
            acceptance_criteria: Some(vec![
                "Modal pre-fills current bookmark data".to_owned(),
                "Validates URL format on save".to_owned(),
            ]),
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
        })
        .unwrap();

    let _id6 = db
        .create_task(TaskInput {
            feature: "bookmark-crud".to_owned(),
            discipline: "frontend".to_owned(),
            title: "Bulk delete bookmarks".to_owned(),
            description: Some(
                "Multi-select bookmarks and delete them in batch with confirmation dialog"
                    .to_owned(),
            ),
            priority: Some(sqlite_db::Priority::Medium),
            tags: vec!["ui".to_owned()],
            depends_on: vec![3],
            acceptance_criteria: Some(vec![
                "Checkbox selection for multiple bookmarks".to_owned(),
                "Confirmation dialog before bulk delete".to_owned(),
            ]),
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
        })
        .unwrap();

    let _id7 = db
        .create_task(TaskInput {
            feature: "bookmark-crud".to_owned(),
            discipline: "security".to_owned(),
            title: "URL input sanitization".to_owned(),
            description: Some(
                "Sanitize and validate all URL inputs to prevent XSS and injection attacks"
                    .to_owned(),
            ),
            priority: Some(sqlite_db::Priority::High),
            tags: vec!["security".to_owned(), "validation".to_owned()],
            depends_on: vec![],
            acceptance_criteria: Some(vec![
                "Rejects javascript: and data: URLs".to_owned(),
                "Escapes HTML entities in bookmark titles".to_owned(),
                "Validates URL format before storage".to_owned(),
            ]),
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
        })
        .unwrap();

    // Task 8: collections / backend / done / high
    let _id8 = db
        .create_task(TaskInput {
            feature: "collections".to_owned(),
            discipline: "backend".to_owned(),
            title: "Collection data model".to_owned(),
            description: Some(
                "Define collection schema with name, color, icon, and bookmark references"
                    .to_owned(),
            ),
            priority: Some(sqlite_db::Priority::High),
            tags: vec!["storage".to_owned(), "data-model".to_owned()],
            depends_on: vec![],
            acceptance_criteria: Some(vec![
                "Collection stores name, color, icon, and ordered bookmark IDs".to_owned(),
                "Supports many-to-many relationship with bookmarks".to_owned(),
            ]),
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
        })
        .unwrap();

    // Task 9: collections / frontend / in_progress / high (depends on 8)
    let _id9 = db.create_task(TaskInput {
        feature: "collections".to_owned(),
        discipline: "frontend".to_owned(),
        title: "Collection sidebar".to_owned(),
        description: Some("Sidebar component showing all collections with bookmark counts and quick navigation".to_owned()),
        priority: Some(sqlite_db::Priority::High),
        tags: vec!["ui".to_owned(), "navigation".to_owned()],
        depends_on: vec![8],
        acceptance_criteria: Some(vec!["Sidebar lists all collections with bookmark counts".to_owned(), "Click collection filters bookmark list".to_owned(), "Collapse/expand sidebar on mobile".to_owned()]),
        context_files: vec![], output_artifacts: vec![], hints: None, estimated_turns: None, provenance: None,
    }).unwrap();

    // Task 10: collections / frontend / pending / medium (depends on 9)
    let _id10 = db
        .create_task(TaskInput {
            feature: "collections".to_owned(),
            discipline: "frontend".to_owned(),
            title: "Drag-and-drop sorting".to_owned(),
            description: Some(
                "Allow reordering bookmarks within a collection via drag-and-drop".to_owned(),
            ),
            priority: Some(sqlite_db::Priority::Medium),
            tags: vec!["ui".to_owned(), "interaction".to_owned()],
            depends_on: vec![9],
            acceptance_criteria: Some(vec![
                "Drag handle on each bookmark card".to_owned(),
                "Visual feedback during drag operation".to_owned(),
            ]),
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
        })
        .unwrap();

    // Task 11: collections / design / pending / none
    let _id11 = db
        .create_task(TaskInput {
            feature: "collections".to_owned(),
            discipline: "frontend".to_owned(),
            title: "Collection icons and colors".to_owned(),
            description: Some(
                "Design the icon picker and color palette for collection customization".to_owned(),
            ),
            priority: None,
            tags: vec!["design".to_owned()],
            depends_on: vec![],
            acceptance_criteria: Some(vec![]),
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
        })
        .unwrap();

    // Task 12: collections / frontend / pending / low (depends on 9, 8)
    let _id12 = db
        .create_task(TaskInput {
            feature: "collections".to_owned(),
            discipline: "frontend".to_owned(),
            title: "Nested collections".to_owned(),
            description: Some(
                "Support hierarchical collection nesting with tree view navigation".to_owned(),
            ),
            priority: Some(sqlite_db::Priority::Low),
            tags: vec!["ui".to_owned(), "navigation".to_owned()],
            depends_on: vec![9, 8],
            acceptance_criteria: Some(vec![
                "Collections can contain sub-collections".to_owned(),
                "Tree view shows hierarchy with expand/collapse".to_owned(),
            ]),
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
        })
        .unwrap();

    // Task 13: search / backend / pending / critical (depends on 3)
    let _id13 = db.create_task(TaskInput {
        feature: "search".to_owned(),
        discipline: "backend".to_owned(),
        title: "Full-text search index".to_owned(),
        description: Some("Build an inverted index for full-text search across bookmark titles, URLs, and notes".to_owned()),
        priority: Some(sqlite_db::Priority::Critical),
        tags: vec!["search".to_owned(), "performance".to_owned()],
        depends_on: vec![3],
        acceptance_criteria: Some(vec!["Index updates on bookmark create/update/delete".to_owned(), "Search returns results in under 50ms for 10k bookmarks".to_owned(), "Supports partial word matching".to_owned(), "Ranks results by relevance".to_owned()]),
        context_files: vec![], output_artifacts: vec![], hints: None, estimated_turns: None, provenance: None,
    }).unwrap();

    // Task 14: search / frontend / blocked / high (depends on 13)
    let _id14 = db
        .create_task(TaskInput {
            feature: "search".to_owned(),
            discipline: "frontend".to_owned(),
            title: "Search bar with autocomplete".to_owned(),
            description: Some(
                "Search input with debounced autocomplete dropdown showing matching bookmarks"
                    .to_owned(),
            ),
            priority: Some(sqlite_db::Priority::High),
            tags: vec!["ui".to_owned(), "search".to_owned()],
            depends_on: vec![13],
            acceptance_criteria: Some(vec![
                "Debounced input with 300ms delay".to_owned(),
                "Dropdown shows top 5 matching bookmarks".to_owned(),
                "Keyboard navigation through results".to_owned(),
            ]),
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
        })
        .unwrap();

    // Task 15: search / testing / pending / none (depends on 13)
    let _id15 = db
        .create_task(TaskInput {
            feature: "search".to_owned(),
            discipline: "quality".to_owned(),
            title: "Search ranking tests".to_owned(),
            description: Some(
                "Test search result ranking and relevance scoring with various query patterns"
                    .to_owned(),
            ),
            priority: None,
            tags: vec!["testing".to_owned(), "search".to_owned()],
            depends_on: vec![13],
            acceptance_criteria: Some(vec![
                "Exact title matches rank highest".to_owned(),
                "Partial matches rank by relevance score".to_owned(),
            ]),
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
        })
        .unwrap();

    // Task 16: import-export / backend / pending / high
    let _id16 = db
        .create_task(TaskInput {
            feature: "import-export".to_owned(),
            discipline: "backend".to_owned(),
            title: "HTML bookmark parser".to_owned(),
            description: Some(
                "Parse Netscape bookmark HTML format exported by Chrome, Firefox, and Safari"
                    .to_owned(),
            ),
            priority: Some(sqlite_db::Priority::High),
            tags: vec!["parser".to_owned(), "import".to_owned()],
            depends_on: vec![],
            acceptance_criteria: Some(vec![
                "Parses Chrome bookmark export format".to_owned(),
                "Parses Firefox bookmark export format".to_owned(),
                "Preserves folder structure as collections".to_owned(),
            ]),
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
        })
        .unwrap();

    // Task 17: import-export / frontend / blocked / medium (depends on 16)
    let _id17 = db
        .create_task(TaskInput {
            feature: "import-export".to_owned(),
            discipline: "frontend".to_owned(),
            title: "Import bookmarks UI".to_owned(),
            description: Some(
                "File upload dialog for importing bookmarks with preview and conflict resolution"
                    .to_owned(),
            ),
            priority: Some(sqlite_db::Priority::Medium),
            tags: vec!["ui".to_owned(), "import".to_owned()],
            depends_on: vec![16],
            acceptance_criteria: Some(vec![
                "File picker accepts .html files".to_owned(),
                "Preview shows bookmarks to import before confirming".to_owned(),
            ]),
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
        })
        .unwrap();

    // Task 18: import-export / frontend / pending / low (depends on 3)
    let _id18 = db
        .create_task(TaskInput {
            feature: "import-export".to_owned(),
            discipline: "frontend".to_owned(),
            title: "Export to JSON".to_owned(),
            description: Some(
                "Export all bookmarks and collections to a JSON file for backup".to_owned(),
            ),
            priority: Some(sqlite_db::Priority::Low),
            tags: vec!["export".to_owned()],
            depends_on: vec![3],
            acceptance_criteria: Some(vec![
                "Exports all bookmarks with metadata".to_owned(),
                "Includes collection membership info".to_owned(),
            ]),
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
        })
        .unwrap();

    // Task 19: settings / docs / skipped / low
    let _id19 = db
        .create_task(TaskInput {
            feature: "settings".to_owned(),
            discipline: "documentation".to_owned(),
            title: "Write settings documentation".to_owned(),
            description: Some(
                "Document all available settings and their default values".to_owned(),
            ),
            priority: Some(sqlite_db::Priority::Low),
            tags: vec!["docs".to_owned()],
            depends_on: vec![],
            acceptance_criteria: Some(vec![]),
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
        })
        .unwrap();

    // Task 20: settings / database / pending / none
    let _id20 = db
        .create_task(TaskInput {
            feature: "settings".to_owned(),
            discipline: "data".to_owned(),
            title: "Theme preference storage".to_owned(),
            description: Some(
                "Store user theme preference (light/dark/system) in local database".to_owned(),
            ),
            priority: None,
            tags: vec!["storage".to_owned()],
            depends_on: vec![],
            acceptance_criteria: Some(vec!["Persists theme preference across sessions".to_owned()]),
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
        })
        .unwrap();

    // Set varied statuses via raw SQL (fixture-only, not public API)
    // Tasks 1,2,3,8 = done; Tasks 4,9 = in_progress; Tasks 14,17 = blocked; Task 19 = skipped
    db.execute_raw("UPDATE tasks SET status = 'done', completed = '2025-01-14' WHERE id = 1")
        .unwrap();
    db.execute_raw("UPDATE tasks SET status = 'done', completed = '2025-01-16' WHERE id = 2")
        .unwrap();
    db.execute_raw("UPDATE tasks SET status = 'done', completed = '2025-01-18' WHERE id = 3")
        .unwrap();
    db.execute_raw("UPDATE tasks SET status = 'in_progress', updated = '2025-01-20' WHERE id = 4")
        .unwrap();
    db.execute_raw("UPDATE tasks SET status = 'done', completed = '2025-01-15' WHERE id = 8")
        .unwrap();
    db.execute_raw("UPDATE tasks SET status = 'in_progress', updated = '2025-01-21' WHERE id = 9")
        .unwrap();
    db.execute_raw("UPDATE tasks SET status = 'blocked', blocked_by = 'Waiting for search index (#13)' WHERE id = 14").unwrap();
    db.execute_raw(
        "UPDATE tasks SET status = 'blocked', blocked_by = 'Needs HTML parser (#16)' WHERE id = 17",
    )
    .unwrap();
    db.execute_raw("UPDATE tasks SET status = 'skipped' WHERE id = 19")
        .unwrap();

    println!(
        "\n✓ Created 04-desktop-dev fixture at: {}",
        fixture_path.display()
    );
    println!("  5 features, 20 tasks (4 done, 2 in_progress, 11 pending, 2 blocked, 1 skipped)");
}

/// Generate all fixtures
/// Note: Calls test functions directly, so run with --test-threads=1 to avoid conflicts
/// Ignored by default to avoid race conditions with individual fixture tests.
/// Run explicitly with: cargo test --test generate_fixtures generate_all_fixtures -- --ignored --nocapture
#[test]
#[ignore = "Calls test functions directly, run with --test-threads=1 to avoid conflicts"]
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
    generate_fixture_01_desktop_blank();
    generate_fixture_02_desktop_feature();
    generate_fixture_03_desktop_tasks();
    generate_fixture_04_desktop_dev();

    println!("\n========================================");
    println!("ALL 5 FIXTURES GENERATED");
    println!("========================================");
    println!("\nFixture progression:");
    println!("  00-empty-project     → Just README, no .undetect-ralph/");
    println!("  01-desktop-blank     → Desktop stack, empty tasks/features, SQLite DB + images");
    println!("  02-desktop-feature   → Desktop stack, 1 feature, no tasks");
    println!("  03-desktop-tasks     → Desktop stack, 2 features, 3 tasks");
    println!("  04-desktop-dev       → Desktop stack, 5 features, 20 tasks (comprehensive)");
    println!("\nNext steps:");
    println!("  1. Review generated fixtures in fixtures/");
    println!("  2. Run: just reset-mock");
    println!("  3. Test with: just dev-mock 04-desktop-dev");
    println!("  4. Commit fixtures to git");
}
