//! Fixture generator tests
//!
//! These tests GENERATE the fixtures in fixtures/ directory.
//! Run with: cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures -- --nocapture
//!
//! Fixtures use .undetect-ralph/ (not .ralph/) so they're not detected as Ralph projects.
//! The mock workflow (just dev-mock) renames .undetect-ralph/ to .ralph/ when copying.

use sqlite_db::{
    AddFeatureCommentInput, FeatureInput, FixedClock, SqliteDb, TaskProvenance, TaskStatus,
};
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
    let db = SqliteDb::open(&db_path, Some(fixed_clock()))?;
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
    SqliteDb::open(&db_path, Some(fixed_clock())).unwrap()
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
        status: None,
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
        None,
        Some(1),
        None,
        "First attempt failed: forgot to add JWT_SECRET to .env".to_owned(),
    )
    .unwrap();

    db.create_task(TaskInput {
        feature: "authentication".to_owned(),
        discipline: "frontend".to_owned(),
        title: "Build login form".to_owned(),
        description: Some("Create UI for user login".to_owned()),
        status: None,
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
        status: None,
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
    })
    .unwrap();
    db.create_feature(FeatureInput {
        name: "collections".to_owned(),
        display_name: "Collections".to_owned(),
        acronym: "COLL".to_owned(),
        description: Some("Organize bookmarks into named collections".to_owned()),
    })
    .unwrap();
    db.create_feature(FeatureInput {
        name: "search".to_owned(),
        display_name: "Search".to_owned(),
        acronym: "SRCH".to_owned(),
        description: Some("Full-text search and filtering across bookmarks".to_owned()),
    })
    .unwrap();
    db.create_feature(FeatureInput {
        name: "import-export".to_owned(),
        display_name: "Import Export".to_owned(),
        acronym: "IMEX".to_owned(),
        description: Some("Import from HTML, export to JSON".to_owned()),
    })
    .unwrap();
    db.create_feature(FeatureInput {
        name: "settings".to_owned(),
        display_name: "Settings".to_owned(),
        acronym: "STNG".to_owned(),
        description: Some("User preferences and theme configuration".to_owned()),
    })
    .unwrap();

    // --- Feature Comments ---
    db.add_feature_comment(AddFeatureCommentInput {
        feature_name: "bookmark-crud".to_owned(),
        category: "design-decision".to_owned(),
        discipline: None,
        agent_task_id: None,
        body: "Use optimistic updates for bookmark creation to avoid UI lag".to_owned(),
        summary: Some("Use optimistic updates for creates".to_owned()),
        reason: Some("Network latency makes synchronous saves feel sluggish".to_owned()),
        source_iteration: None,
    })
    .unwrap();
    db.add_feature_comment(AddFeatureCommentInput {
        feature_name: "bookmark-crud".to_owned(),
        category: "gotcha".to_owned(),
        discipline: Some("frontend".to_owned()),
        agent_task_id: Some(2),
        body: "Favicon URLs often 404 — always provide a fallback icon".to_owned(),
        summary: None,
        reason: None,
        source_iteration: Some(1),
    })
    .unwrap();
    db.add_feature_comment(AddFeatureCommentInput {
        feature_name: "bookmark-crud".to_owned(),
        category: "convention".to_owned(),
        discipline: Some("backend".to_owned()),
        agent_task_id: Some(3),
        body: "All bookmark IDs are ULIDs, not auto-increment integers. This keeps them sortable by creation time without a separate timestamp index.".to_owned(),
        summary: Some("ULIDs for bookmark IDs".to_owned()),
        reason: Some("Sortable by creation time without extra index".to_owned()),
        source_iteration: Some(1),
    })
    .unwrap();
    db.add_feature_comment(AddFeatureCommentInput {
        feature_name: "bookmark-crud".to_owned(),
        category: "architecture".to_owned(),
        discipline: Some("backend".to_owned()),
        agent_task_id: None,
        body: "Bookmark storage uses a write-ahead log pattern: mutations go to a WAL table first, then get compacted into the main bookmarks table on read. This avoids locking during writes.".to_owned(),
        summary: Some("WAL pattern for bookmark writes".to_owned()),
        reason: Some("Avoids write locks on the main table".to_owned()),
        source_iteration: None,
    })
    .unwrap();
    db.add_feature_comment(AddFeatureCommentInput {
        feature_name: "bookmark-crud".to_owned(),
        category: "gotcha".to_owned(),
        discipline: Some("quality".to_owned()),
        agent_task_id: Some(4),
        body: "URL normalization strips trailing slashes and lowercases the hostname, but preserves path case. Two URLs that look different may be the same bookmark after normalization.".to_owned(),
        summary: None,
        reason: None,
        source_iteration: Some(2),
    })
    .unwrap();
    db.add_feature_comment(AddFeatureCommentInput {
        feature_name: "bookmark-crud".to_owned(),
        category: "boundary".to_owned(),
        discipline: Some("security".to_owned()),
        agent_task_id: None,
        body: "Never store raw user-provided HTML in bookmark notes. All note content goes through DOMPurify before persistence.".to_owned(),
        summary: Some("Sanitize notes with DOMPurify".to_owned()),
        reason: Some("Prevents stored XSS via bookmark notes".to_owned()),
        source_iteration: None,
    })
    .unwrap();
    db.add_feature_comment(AddFeatureCommentInput {
        feature_name: "bookmark-crud".to_owned(),
        category: "dependency".to_owned(),
        discipline: Some("frontend".to_owned()),
        agent_task_id: None,
        body: "The bookmark card component depends on the favicon proxy service. If the proxy is down, cards should render with a generic globe icon instead of breaking.".to_owned(),
        summary: Some("Favicon proxy fallback to globe icon".to_owned()),
        reason: None,
        source_iteration: None,
    })
    .unwrap();
    db.add_feature_comment(AddFeatureCommentInput {
        feature_name: "bookmark-crud".to_owned(),
        category: "design-decision".to_owned(),
        discipline: Some("frontend".to_owned()),
        agent_task_id: Some(5),
        body: "Edit modal uses a sheet sliding in from the right, not a centered dialog. This keeps the bookmark list visible for context while editing.".to_owned(),
        summary: Some("Sheet for edit, not dialog".to_owned()),
        reason: Some("User can see the list while editing".to_owned()),
        source_iteration: Some(3),
    })
    .unwrap();
    db.add_feature_comment(AddFeatureCommentInput {
        feature_name: "bookmark-crud".to_owned(),
        category: "gotcha".to_owned(),
        discipline: Some("data".to_owned()),
        agent_task_id: None,
        body: "Bulk delete must cascade to collection membership. Deleting a bookmark that belongs to 3 collections needs to clean up all 3 junction rows.".to_owned(),
        summary: None,
        reason: Some("Orphaned junction rows cause ghost counts in collection sidebar".to_owned()),
        source_iteration: None,
    })
    .unwrap();
    db.add_feature_comment(AddFeatureCommentInput {
        feature_name: "bookmark-crud".to_owned(),
        category: "convention".to_owned(),
        discipline: Some("platform".to_owned()),
        agent_task_id: None,
        body: "All CRUD endpoints follow the pattern: POST /bookmarks, GET /bookmarks/:id, PATCH /bookmarks/:id, DELETE /bookmarks/:id. No PUT — partial updates only.".to_owned(),
        summary: Some("PATCH for updates, no PUT".to_owned()),
        reason: Some("Partial updates reduce payload size and merge conflicts".to_owned()),
        source_iteration: None,
    })
    .unwrap();
    db.add_feature_comment(AddFeatureCommentInput {
        feature_name: "bookmark-crud".to_owned(),
        category: "architecture".to_owned(),
        discipline: Some("frontend".to_owned()),
        agent_task_id: Some(1),
        body: "Bookmark list uses virtual scrolling via tanstack-virtual. Only visible cards are in the DOM. This is critical — users with 5k+ bookmarks were hitting 2s render times without it.".to_owned(),
        summary: Some("Virtual scrolling for large lists".to_owned()),
        reason: Some("5k+ bookmarks caused 2s render without virtualization".to_owned()),
        source_iteration: Some(1),
    })
    .unwrap();
    db.add_feature_comment(AddFeatureCommentInput {
        feature_name: "bookmark-crud".to_owned(),
        category: "boundary".to_owned(),
        discipline: Some("backend".to_owned()),
        agent_task_id: None,
        body: "Maximum bookmark title length is 500 chars, URL is 2048 chars, notes is 10000 chars. Enforce at both API and DB constraint level.".to_owned(),
        summary: Some("Field length limits: title 500, URL 2048, notes 10k".to_owned()),
        reason: None,
        source_iteration: None,
    })
    .unwrap();
    db.add_feature_comment(AddFeatureCommentInput {
        feature_name: "bookmark-crud".to_owned(),
        category: "design-decision".to_owned(),
        discipline: None,
        agent_task_id: None,
        body: "Soft delete with 30-day trash retention. Deleted bookmarks move to a trash view and auto-purge after 30 days. Users can restore from trash.".to_owned(),
        summary: Some("Soft delete with 30-day trash".to_owned()),
        reason: Some("Prevents accidental permanent data loss".to_owned()),
        source_iteration: None,
    })
    .unwrap();
    db.add_feature_comment(AddFeatureCommentInput {
        feature_name: "collections".to_owned(),
        category: "architecture".to_owned(),

        discipline: None,
        agent_task_id: None,
        body: "Collections are flat, not nested — no recursive trees".to_owned(),
        summary: Some("Flat collections only, no nesting".to_owned()),
        reason: Some("Keeps the data model simple and avoids infinite depth bugs".to_owned()),
        source_iteration: None,
    })
    .unwrap();
    db.add_feature_comment(AddFeatureCommentInput {
        feature_name: "collections".to_owned(),
        category: "convention".to_owned(),

        discipline: None,
        agent_task_id: None,
        body: "Collection names are unique per user, case-insensitive".to_owned(),
        summary: None,
        reason: None,
        source_iteration: None,
    })
    .unwrap();
    db.add_feature_comment(AddFeatureCommentInput {
        feature_name: "search".to_owned(),
        category: "design-decision".to_owned(),

        discipline: None,
        agent_task_id: None,
        body: "Use SQLite FTS5 for full-text search instead of client-side filtering".to_owned(),
        summary: Some("Use FTS5 for search".to_owned()),
        reason: Some("Scales better with large bookmark collections".to_owned()),
        source_iteration: None,
    })
    .unwrap();
    db.add_feature_comment(AddFeatureCommentInput {
        feature_name: "search".to_owned(),
        category: "gotcha".to_owned(),

        discipline: Some("backend".to_owned()),
        agent_task_id: Some(8),
        body: "FTS5 tokenizer splits on hyphens — URLs with dashes need special handling"
            .to_owned(),
        summary: None,
        reason: None,
        source_iteration: Some(2),
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
            status: None,
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
            status: None,
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
            status: None,
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
            status: None,
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
            status: None,
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
            status: None,
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
            status: None,
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
            status: None,
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
        status: None,
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
            status: None,
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
            status: None,
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
            status: None,
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
        status: None,
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
            status: None,
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
            status: None,
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
            status: None,
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
            status: None,
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
            status: None,
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
            status: None,
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
            status: None,
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

    // Task 21: MCP Verb Reference (ALL 8 SIGNALS IN ONE TASK)
    let _id21 = db
        .create_task(TaskInput {
            feature: "bookmark-crud".to_owned(),
            discipline: "frontend".to_owned(),
            title: "🔬 MCP Signal Reference — All 8 Verbs".to_owned(),
            description: Some(
                "**DEV ONLY:** This task demonstrates all 8 MCP exhaust pipe verbs in a single timeline. Each signal showcases the full schema for that verb type. Use this as a visual reference for signal rendering.".to_owned(),
            ),
            status: None,
            priority: Some(sqlite_db::Priority::Low),
            tags: vec!["dev".to_owned(), "reference".to_owned(), "mcp-signals".to_owned()],
            depends_on: vec![],
            acceptance_criteria: Some(vec![
                "All 8 MCP verbs represented in comment timeline".to_owned(),
                "Each signal uses full schema for that verb".to_owned(),
            ]),
            context_files: vec![],
            output_artifacts: vec![],
            hints: Some("This is a fixture-only reference task. Delete before production.".to_owned()),
            estimated_turns: None,
            provenance: None,
        })
        .unwrap();

    // Set provenance on all 21 tasks
    for id in [1, 3, 4, 5, 6, 8, 9, 10, 12, 13, 14, 15, 16, 17, 18, 20] {
        db.set_task_provenance(id, TaskProvenance::Agent).unwrap();
    }
    for id in [2, 7, 11, 21] {
        db.set_task_provenance(id, TaskProvenance::Human).unwrap();
    }
    db.set_task_provenance(19, TaskProvenance::System).unwrap();

    // Mark task 21 as in_progress so signals are visible
    db.set_task_status_with_date(21, TaskStatus::InProgress, "2025-01-22")
        .unwrap();

    db.add_comment(
        1,
        Some("frontend".to_owned()),
        None,
        None,
        "Card layout finalized, using 3-column grid on desktop.".to_owned(),
    )
    .unwrap();
    db.add_comment(
        2,
        Some("frontend".to_owned()),
        None,
        None,
        "Auto-title fetch uses og:title with URL fallback.".to_owned(),
    )
    .unwrap();
    db.add_comment(
        3,
        Some("backend".to_owned()),
        Some(3),
        None,
        "localStorage wrapper handles quota errors with LRU eviction.".to_owned(),
    )
    .unwrap();
    db.add_comment(
        4,
        Some("quality".to_owned()),
        Some(4),
        Some("high".to_owned()),
        "Found edge case: empty URL string passes validation. Adding test.".to_owned(),
    )
    .unwrap();
    db.add_comment(
        4,
        None,
        None,
        None,
        "Also test unicode URLs please.".to_owned(),
    )
    .unwrap();
    db.add_comment(
        7,
        Some("security".to_owned()),
        None,
        Some("high".to_owned()),
        "Added CSP header and input sanitization for all URL fields.".to_owned(),
    )
    .unwrap();
    db.add_comment(
        8,
        Some("backend".to_owned()),
        Some(8),
        None,
        "Schema uses JSON column for bookmark refs, supports ordering.".to_owned(),
    )
    .unwrap();
    db.add_comment(
        9,
        Some("frontend".to_owned()),
        Some(9),
        Some("medium".to_owned()),
        "Sidebar uses virtual scroll for collections > 50.".to_owned(),
    )
    .unwrap();
    db.add_comment(
        9,
        Some("frontend".to_owned()),
        Some(9),
        None,
        "Collapse state persisted in localStorage.".to_owned(),
    )
    .unwrap();
    db.add_comment(
        13,
        Some("backend".to_owned()),
        None,
        None,
        "Evaluating lunr.js vs custom inverted index. lunr.js is 8kb gzipped.".to_owned(),
    )
    .unwrap();
    db.add_comment(
        14,
        None,
        None,
        Some("low".to_owned()),
        "Blocked until search index is ready. Low priority for now.".to_owned(),
    )
    .unwrap();
    db.add_comment(
        16,
        Some("backend".to_owned()),
        None,
        None,
        "Chrome and Firefox use same Netscape format. Safari differs slightly.".to_owned(),
    )
    .unwrap();
    db.add_comment(
        16,
        Some("quality".to_owned()),
        None,
        None,
        "Need sample export files from each browser for test fixtures.".to_owned(),
    )
    .unwrap();
    db.add_comment(
        17,
        None,
        None,
        None,
        "Blocked on HTML parser. Will design the UI in parallel once unblocked.".to_owned(),
    )
    .unwrap();
    db.add_comment(
        20,
        Some("data".to_owned()),
        None,
        None,
        "Using IndexedDB for theme + future settings. localStorage too limited.".to_owned(),
    )
    .unwrap();

    // Set varied statuses with historical dates (fixture-only)
    // Tasks 1,2,3,8 = done; Tasks 4,9 = in_progress; Tasks 14,17 = blocked; Task 19 = skipped
    db.set_task_status_with_date(1, TaskStatus::Done, "2025-01-14")
        .unwrap();
    db.set_task_status_with_date(2, TaskStatus::Done, "2025-01-16")
        .unwrap();
    db.set_task_status_with_date(3, TaskStatus::Done, "2025-01-18")
        .unwrap();
    db.set_task_status_with_date(4, TaskStatus::InProgress, "2025-01-20")
        .unwrap();
    db.set_task_status_with_date(8, TaskStatus::Done, "2025-01-15")
        .unwrap();
    db.set_task_status_with_date(9, TaskStatus::InProgress, "2025-01-21")
        .unwrap();
    db.set_task_status(14, TaskStatus::Blocked).unwrap();
    db.set_task_status(17, TaskStatus::Blocked).unwrap();
    db.set_task_status(19, TaskStatus::Skipped).unwrap();

    // --- Task Signals (as structured comments) ---
    // TODO: Reimplement signal examples with new canonical schema (discipline_id, verb, text fields)
    // Commented out during Phase 2 schema migration
    /*
    // Task 4 (in_progress, unit tests): 2 sessions with mixed verbs
    // Session 1: flag + learned + partial
    db.execute_raw(
        "INSERT INTO task_comments (task_id, session_id, author, signal_verb, signal_payload, body, created) VALUES
        (4, 'sess-4a-001', 'quality', 'flag', '{\"what\":\"Empty URL string passes bookmark validation\",\"severity\":\"warning\",\"category\":\"bug\"}', '🚩 **Flag (warning):** Empty URL string passes bookmark validation\n\n**Category:** bug', '2025-01-20 10:05:00'),
        (4, 'sess-4a-001', 'quality', 'learned', '{\"text\":\"localStorage has a 5MB quota per origin in Chromium\",\"kind\":\"discovery\",\"scope\":\"feature\",\"rationale\":\"Needed to calculate max bookmarks before quota hit\"}', '💡 **Learned (discovery):** localStorage has a 5MB quota per origin in Chromium\n\n**Rationale:** Needed to calculate max bookmarks before quota hit\n\n**Scope:** feature', '2025-01-20 10:12:00'),
        (4, 'sess-4a-001', 'quality', 'partial', '{\"summary\":\"Wrote 12 of 18 planned test cases for CRUD operations\",\"remaining\":\"Need delete edge cases and bulk operations tests\"}', '⊙ **Partial:** Wrote 12 of 18 planned test cases for CRUD operations\n\n**Remaining:** Need delete edge cases and bulk operations tests', '2025-01-20 10:30:00')"
    ).unwrap();

    // Session 2: ask (blocking, unanswered) + flag + learned + stuck
    db.execute_raw(
        "INSERT INTO task_comments (task_id, session_id, author, signal_verb, signal_payload, body, created) VALUES
        (4, 'sess-4b-002', 'quality', 'ask', '{\"question\":\"Should empty URL strings be treated as validation errors or silently skipped?\",\"options\":[\"Reject with error\",\"Skip silently\",\"Auto-fill with placeholder URL\"],\"preferred\":\"Reject with error\",\"blocking\":true}', '❓ **Ask (blocking):** Should empty URL strings be treated as validation errors or silently skipped?\n\n**Preferred:** Reject with error\n\n**Options:**\n- Reject with error\n- Skip silently\n- Auto-fill with placeholder URL', '2025-01-20 14:00:00'),
        (4, 'sess-4b-002', 'quality', 'flag', '{\"what\":\"Unicode URLs cause double-encoding in localStorage keys\",\"severity\":\"blocking\",\"category\":\"bug\"}', '🚩 **Flag (blocking):** Unicode URLs cause double-encoding in localStorage keys\n\n**Category:** bug', '2025-01-20 14:15:00'),
        (4, 'sess-4b-002', 'quality', 'learned', '{\"text\":\"Vitest mocks of localStorage need explicit reset between tests or state leaks across suites\",\"kind\":\"convention\",\"scope\":\"task\"}', '💡 **Learned (convention):** Vitest mocks of localStorage need explicit reset between tests or state leaks across suites\n\n**Scope:** task', '2025-01-20 14:20:00'),
        (4, 'sess-4b-002', 'quality', 'stuck', '{\"reason\":\"Cannot proceed with delete tests until the empty URL validation question is answered — test assertions depend on the expected behavior\"}', '⚠ **Stuck:** Cannot proceed with delete tests until the empty URL validation question is answered — test assertions depend on the expected behavior', '2025-01-20 14:25:00')"
    ).unwrap();

    // Task 9 (in_progress, collection sidebar): 1 session with suggest + partial
    db.execute_raw(
        "INSERT INTO task_comments (task_id, session_id, author, signal_verb, signal_payload, body, created) VALUES
        (9, 'sess-9a-001', 'frontend', 'suggest', '{\"what\":\"Add keyboard shortcut (Cmd+B) to toggle sidebar\",\"kind\":\"new_task\",\"why\":\"Power users expect sidebar toggle shortcuts in desktop apps\"}', '💭 **Suggest (new_task):** Add keyboard shortcut (Cmd+B) to toggle sidebar\n\n**Why:** Power users expect sidebar toggle shortcuts in desktop apps', '2025-01-21 09:10:00'),
        (9, 'sess-9a-001', 'frontend', 'learned', '{\"text\":\"Virtual scroll in sidebar needs a fixed height container — percentage heights do not work with ResizeObserver\",\"kind\":\"gotcha\",\"scope\":\"task\"}', '💡 **Learned (gotcha):** Virtual scroll in sidebar needs a fixed height container — percentage heights do not work with ResizeObserver\n\n**Scope:** task', '2025-01-21 09:20:00'),
        (9, 'sess-9a-001', 'frontend', 'partial', '{\"summary\":\"Sidebar renders collection list with counts. Collapse/expand works.\",\"remaining\":\"Mobile responsive behavior and virtual scroll for 50+ collections\"}', '⊙ **Partial:** Sidebar renders collection list with counts. Collapse/expand works.\n\n**Remaining:** Mobile responsive behavior and virtual scroll for 50+ collections', '2025-01-21 09:45:00')"
    ).unwrap();

    // Task 3 (done, bookmark storage): 2 sessions showing progression to done
    // Session 1: blocked + partial
    db.execute_raw(
        "INSERT INTO task_comments (task_id, session_id, author, signal_verb, signal_payload, body, created) VALUES
        (3, 'sess-3a-001', 'backend', 'blocked', '{\"on\":\"localStorage API not available in test environment\",\"kind\":\"environment\",\"detail\":\"Vitest runs in Node where localStorage is not defined. Need to configure jsdom or add a mock.\"}', '🚫 **Blocked (environment):** localStorage API not available in test environment\n\n**Detail:** Vitest runs in Node where localStorage is not defined. Need to configure jsdom or add a mock.', '2025-01-17 11:00:00'),
        (3, 'sess-3a-001', 'backend', 'partial', '{\"summary\":\"CRUD functions written but untestable without localStorage mock\",\"remaining\":\"Configure jsdom environment for tests, then verify all operations\"}', '⊙ **Partial:** CRUD functions written but untestable without localStorage mock\n\n**Remaining:** Configure jsdom environment for tests, then verify all operations', '2025-01-17 11:30:00')"
    ).unwrap();

    // Session 2: flag (info) + learned + done
    db.execute_raw(
        "INSERT INTO task_comments (task_id, session_id, author, signal_verb, signal_payload, body, created) VALUES
        (3, 'sess-3b-002', 'backend', 'flag', '{\"what\":\"localStorage.setItem can throw QuotaExceededError\",\"severity\":\"info\",\"category\":\"edge-case\"}', '🚩 **Flag (info):** localStorage.setItem can throw QuotaExceededError\n\n**Category:** edge-case', '2025-01-18 08:15:00'),
        (3, 'sess-3b-002', 'backend', 'learned', '{\"text\":\"Wrapping localStorage calls in try/catch with LRU eviction fallback handles quota gracefully\",\"kind\":\"pattern\",\"scope\":\"feature\",\"rationale\":\"Users with thousands of bookmarks will eventually hit the 5MB limit\"}', '💡 **Learned (pattern):** Wrapping localStorage calls in try/catch with LRU eviction fallback handles quota gracefully\n\n**Rationale:** Users with thousands of bookmarks will eventually hit the 5MB limit\n\n**Scope:** feature', '2025-01-18 08:30:00'),
        (3, 'sess-3b-002', 'backend', 'done', '{\"summary\":\"All CRUD operations implemented with localStorage persistence. Quota errors handled via LRU eviction. All 3 acceptance criteria pass.\"}', '✓ **Done:** All CRUD operations implemented with localStorage persistence. Quota errors handled via LRU eviction. All 3 acceptance criteria pass.', '2025-01-18 09:00:00')"
    ).unwrap();

    // Task 14 (blocked, search bar): 1 session showing blocked
    db.execute_raw(
        "INSERT INTO task_comments (task_id, session_id, author, signal_verb, signal_payload, body, created) VALUES
        (14, 'sess-14a-001', 'frontend', 'blocked', '{\"on\":\"Full-text search index (task #13) not implemented yet\",\"kind\":\"dependency\",\"detail\":\"Cannot build autocomplete without a search backend to query against\"}', '🚫 **Blocked (dependency):** Full-text search index (task #13) not implemented yet\n\n**Detail:** Cannot build autocomplete without a search backend to query against', '2025-01-22 10:00:00')"
    ).unwrap();

    // Task 1 (done, bookmark card layout): 1 session, clean done
    db.execute_raw(
        "INSERT INTO task_comments (task_id, session_id, author, signal_verb, signal_payload, body, created) VALUES
        (1, 'sess-1a-001', 'frontend', 'learned', '{\"text\":\"Favicon URLs frequently 404 — must use fallback globe icon\",\"kind\":\"gotcha\",\"scope\":\"feature\"}', '💡 **Learned (gotcha):** Favicon URLs frequently 404 — must use fallback globe icon\n\n**Scope:** feature', '2025-01-14 08:20:00'),
        (1, 'sess-1a-001', 'frontend', 'done', '{\"summary\":\"Bookmark card component with favicon, title, truncated URL, and hover action buttons. Uses 3-column grid on desktop.\"}', '✓ **Done:** Bookmark card component with favicon, title, truncated URL, and hover action buttons. Uses 3-column grid on desktop.', '2025-01-14 09:00:00')"
    ).unwrap();

    // Task 8 (done, collection data model): 1 session with ask (answered) + done
    db.execute_raw(
        "INSERT INTO task_comments (task_id, session_id, author, signal_verb, signal_payload, signal_answered, body, created) VALUES
        (8, 'sess-8a-001', 'backend', 'ask', '{\"question\":\"Should collections support ordering of their bookmarks, or just store them as an unordered set?\",\"options\":[\"Ordered (array of IDs)\",\"Unordered (set of IDs)\"],\"preferred\":\"Ordered (array of IDs)\",\"blocking\":true}', 'Ordered (array of IDs)', '❓ **Ask (blocking):** Should collections support ordering of their bookmarks, or just store them as an unordered set?\n\n**Preferred:** Ordered (array of IDs)\n\n**Options:**\n- Ordered (array of IDs)\n- Unordered (set of IDs)\n\n**Answer:** Ordered (array of IDs)', '2025-01-15 10:00:00')"
    ).unwrap();
    db.execute_raw(
        "INSERT INTO task_comments (task_id, session_id, author, signal_verb, signal_payload, body, created) VALUES
        (8, 'sess-8a-001', 'backend', 'done', '{\"summary\":\"Collection schema defined with name, color, icon, and ordered bookmark ID array. Junction table supports many-to-many with position column.\"}', '✓ **Done:** Collection schema defined with name, color, icon, and ordered bookmark ID array. Junction table supports many-to-many with position column.', '2025-01-15 11:00:00')"
    ).unwrap();

    // Task 7 (pending, URL sanitization): 1 session with suggest that created a draft task
    db.execute_raw(
        "INSERT INTO task_comments (task_id, session_id, author, signal_verb, signal_payload, body, created) VALUES
        (7, 'sess-7a-001', 'security', 'suggest', '{\"what\":\"Add CSP meta tag to index.html to prevent inline script injection\",\"kind\":\"improvement\",\"why\":\"Defense-in-depth — even if XSS bypasses input sanitization, CSP blocks execution\"}', '💭 **Suggest (improvement):** Add CSP meta tag to index.html to prevent inline script injection\n\n**Why:** Defense-in-depth — even if XSS bypasses input sanitization, CSP blocks execution', '2025-01-19 15:00:00'),
        (7, 'sess-7a-001', 'security', 'flag', '{\"what\":\"data: URLs can embed arbitrary content including scripts\",\"severity\":\"blocking\",\"category\":\"security\"}', '🚩 **Flag (blocking):** data: URLs can embed arbitrary content including scripts\n\n**Category:** security', '2025-01-19 15:10:00')"
    ).unwrap();

    // Task 21 (in_progress, MCP Signal Reference): ALL 8 VERBS + VARIANTS (15 total signals)
    db.execute_raw(
        "INSERT INTO task_comments (task_id, session_id, author, signal_verb, signal_payload, body, created) VALUES
        (21, 'sess-21-ref', 'frontend', 'done', '{\"summary\":\"__Agent writes DONE summary when task is 100% complete. Example: Implemented CSV/JSON export with quota handling, all 3 acceptance criteria pass, tests green, PR merged.\"}', '__Agent writes DONE summary when task is 100% complete. Example: Implemented CSV/JSON export with quota handling, all 3 acceptance criteria pass, tests green, PR merged.', '2025-01-22 14:00:00'),
        (21, 'sess-21-ref', 'frontend', 'partial', '{\"summary\":\"__Agent describes what WAS completed in this session. Example: Implemented CSV export format, added filename sanitization, wrote 12 of 18 unit tests.\",\"remaining\":\"__Agent describes what STILL needs to be done. Example: JSON export format, bulk export UI, remaining 6 edge-case tests.\"}', '__PARTIAL signals show incremental progress. Agent fills TWO fields: summary (what got done) + remaining (what is left).\n\n__summary: Implemented CSV export format, added filename sanitization, wrote 12 of 18 unit tests.\n\n__remaining: JSON export format, bulk export UI, remaining 6 edge-case tests.', '2025-01-22 14:05:00'),
        (21, 'sess-21-ref', 'frontend', 'stuck', '{\"reason\":\"__Agent explains WHY they are stuck — technical blocker, missing info, ambiguous spec, conflicting requirements. Example: Cannot determine correct MIME type for .bookmark files — RFC 7231 spec is ambiguous and Chrome/Firefox behave differently.\"}', '__STUCK signals mean agent hit a wall and cannot proceed without help.\n\n__reason: Cannot determine correct MIME type for .bookmark files — RFC 7231 spec is ambiguous and Chrome/Firefox behave differently.', '2025-01-22 14:10:00'),
        (21, 'sess-21-ref', 'frontend', 'ask', '{\"question\":\"__Agent poses a BLOCKING question that halts progress. Example: Should export filename include timestamp (bookmarks-2025-01-22.json) or use static name (bookmarks.json)?\",\"options\":[\"__Option 1: Add timestamp\",\"__Option 2: Static filename\",\"__Option 3: Let user choose via dialog\"],\"preferred\":\"__Option 1: Add timestamp\",\"blocking\":true}', '__ASK (BLOCKING variant) — Question that halts progress until answered.\n\n__question: Should export filename include timestamp (bookmarks-2025-01-22.json) or use static name (bookmarks.json)?\n\n__preferred: Option 1 (Add timestamp)\n\n__options:\n- Add timestamp (bookmarks-2025-01-22.json)\n- Static name (bookmarks.json)\n- Let user choose via dialog\n\n__blocking: true', '2025-01-22 14:15:00'),
        (21, 'sess-21-ref', 'frontend', 'ask', '{\"question\":\"__Agent poses a NON-BLOCKING question for clarification/preference. Example: Should we use .csv or .tsv extension for tab-separated exports?\",\"options\":[\"__Use .csv (more common)\",\"__Use .tsv (more accurate)\"],\"preferred\":\"__Use .tsv (more accurate)\",\"blocking\":false}', '__ASK (NON-BLOCKING variant) — Clarification question that doesn''t halt work.\n\n__question: Should we use .csv or .tsv extension for tab-separated exports?\n\n__preferred: Use .tsv (more accurate)\n\n__options:\n- Use .csv (more common)\n- Use .tsv (more accurate)\n\n__blocking: false (can proceed with default)', '2025-01-22 14:17:00'),
        (21, 'sess-21-ref', 'frontend', 'flag', '{\"what\":\"__BLOCKING severity — Critical bug that prevents core functionality. Example: Export button downloads empty 0-byte file when localStorage.getItem() returns null.\",\"severity\":\"blocking\",\"category\":\"bug\"}', '__FLAG (BLOCKING severity) — Critical issue that breaks core functionality.\n\n__severity: blocking\n__category: bug\n\n__what: Export button downloads empty 0-byte file when localStorage.getItem() returns null.', '2025-01-22 14:20:00'),
        (21, 'sess-21-ref', 'frontend', 'flag', '{\"what\":\"__WARNING severity — Non-critical issue worth fixing but not urgent. Example: Export with 10k+ bookmarks takes 3+ seconds and freezes UI thread.\",\"severity\":\"warning\",\"category\":\"performance\"}', '__FLAG (WARNING severity) — Issue worth addressing but not critical.\n\n__severity: warning\n__category: performance\n\n__what: Export with 10k+ bookmarks takes 3+ seconds and freezes UI thread.', '2025-01-22 14:22:00'),
        (21, 'sess-21-ref', 'frontend', 'flag', '{\"what\":\"__INFO severity — FYI observation or minor edge case. Example: Export filename gets truncated to 255 chars on older Windows filesystems.\",\"severity\":\"info\",\"category\":\"edge-case\"}', '__FLAG (INFO severity) — Minor observation or edge case for awareness.\n\n__severity: info\n__category: edge-case\n\n__what: Export filename gets truncated to 255 chars on older Windows filesystems.', '2025-01-22 14:24:00'),
        (21, 'sess-21-ref', 'frontend', 'learned', '{\"text\":\"__GOTCHA kind — Non-obvious behavior that can trip you up. Example: Browser download APIs create Blob URLs that must be manually revoked via URL.revokeObjectURL() or they leak memory until page reload.\",\"kind\":\"gotcha\",\"scope\":\"feature\",\"rationale\":\"__Without cleanup, every export leaks ~50KB. User with 100 exports = 5MB leaked RAM.\"}', '__LEARNED (GOTCHA kind) — Non-obvious behavior worth documenting.\n\n__kind: gotcha\n__scope: feature\n\n__text: Browser download APIs create Blob URLs that must be manually revoked via URL.revokeObjectURL() or they leak memory until page reload.\n\n__rationale: Without cleanup, every export leaks ~50KB. User with 100 exports = 5MB leaked RAM.', '2025-01-22 14:26:00'),
        (21, 'sess-21-ref', 'frontend', 'learned', '{\"text\":\"__PATTERN kind — Reusable approach or technique discovered. Example: Wrap all localStorage operations in try/catch with LRU eviction fallback — handles quota errors gracefully.\",\"kind\":\"pattern\",\"scope\":\"task\"}', '__LEARNED (PATTERN kind) — Reusable technique or approach.\n\n__kind: pattern\n__scope: task\n\n__text: Wrap all localStorage operations in try/catch with LRU eviction fallback — handles quota errors gracefully.', '2025-01-22 14:28:00'),
        (21, 'sess-21-ref', 'frontend', 'learned', '{\"text\":\"__CONVENTION kind — Team standard or coding rule established. Example: All export filenames follow pattern: appname-entity-timestamp.ext (e.g., ralph-bookmarks-2025-01-22.json).\",\"kind\":\"convention\",\"scope\":\"feature\"}', '__LEARNED (CONVENTION kind) — Team standard or naming rule.\n\n__kind: convention\n__scope: feature\n\n__text: All export filenames follow pattern: appname-entity-timestamp.ext (e.g., ralph-bookmarks-2025-01-22.json).', '2025-01-22 14:30:00'),
        (21, 'sess-21-ref', 'frontend', 'suggest', '{\"what\":\"__IMPROVEMENT kind — Enhancement to existing functionality. Example: Add Copy to Clipboard button next to Download — users want to paste bookmark JSON into Slack without saving a file.\",\"kind\":\"improvement\",\"why\":\"__40% of exports in analytics are followed by manual file-open-copy-paste. Direct clipboard = better UX.\"}', '__SUGGEST (IMPROVEMENT kind) — Enhancement to existing feature.\n\n__kind: improvement\n\n__what: Add Copy to Clipboard button next to Download — users want to paste bookmark JSON into Slack without saving a file.\n\n__why: 40% of exports in analytics are followed by manual file-open-copy-paste. Direct clipboard = better UX.', '2025-01-22 14:32:00'),
        (21, 'sess-21-ref', 'frontend', 'suggest', '{\"what\":\"__NEW_TASK kind — Proposal for entirely new feature or task. Example: Add scheduled auto-export that backs up bookmarks to user''s chosen cloud storage every 24 hours.\",\"kind\":\"new_task\",\"why\":\"__Users in support threads frequently ask about backup/sync. Auto-export prevents data loss.\"}', '__SUGGEST (NEW_TASK kind) — Proposal for new feature.\n\n__kind: new_task\n\n__what: Add scheduled auto-export that backs up bookmarks to user''s chosen cloud storage every 24 hours.\n\n__why: Users in support threads frequently ask about backup/sync. Auto-export prevents data loss.', '2025-01-22 14:34:00'),
        (21, 'sess-21-ref', 'frontend', 'blocked', '{\"on\":\"__DEPENDENCY kind — Blocked by missing code/API from another task. Example: Need backend API endpoint POST /export/stream for server-side export generation.\",\"kind\":\"dependency\",\"detail\":\"__Client-side export works for <1000 bookmarks but crashes tab with larger datasets. Need streaming response.\"}', '__BLOCKED (DEPENDENCY kind) — Waiting on another task/team.\n\n__kind: dependency\n\n__on: Need backend API endpoint POST /export/stream for server-side export generation.\n\n__detail: Client-side export works for <1000 bookmarks but crashes tab with larger datasets. Need streaming response.', '2025-01-22 14:36:00'),
        (21, 'sess-21-ref', 'frontend', 'blocked', '{\"on\":\"__ENVIRONMENT kind — Blocked by dev environment or tooling issue. Example: Bun v1.2+ required for File System Access API — CI still running Bun v1.0.15.\",\"kind\":\"environment\"}', '__BLOCKED (ENVIRONMENT kind) — Dev environment or tooling blocker.\n\n__kind: environment\n\n__on: Bun v1.2+ required for File System Access API — CI still running Bun v1.0.15.', '2025-01-22 14:38:00')"
    ).unwrap();
    */

    println!(
        "\n✓ Created 04-desktop-dev fixture at: {}",
        fixture_path.display()
    );
    println!("  5 features, 21 tasks (4 done, 3 in_progress, 11 pending, 2 blocked, 1 skipped)");
    // println!("  37 signals across 10 sessions on 8 tasks (all 8 verbs represented)");
    println!("  → Task #21: MCP Signal Reference with ALL 8 VERBS + VARIANTS (15 signals)");
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
