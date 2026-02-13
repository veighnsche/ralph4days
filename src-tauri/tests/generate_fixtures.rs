//! Fixture generator tests
//!
//! These tests GENERATE the fixtures in fixtures/ directory.
//! Run with: cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures -- --nocapture
//!
//! Fixtures use .undetect-ralph/ (not .ralph/) so they're not detected as Ralph projects.
//! The mock workflow (just dev-mock) renames .undetect-ralph/ to .ralph/ when copying.

use sqlite_db::{
    AddFeatureCommentInput, AskSignalInput, BlockedSignalInput, DoneSignalInput, FeatureInput,
    FixedClock, FlagSignalInput, LearnedSignalInput, PartialSignalInput, SqliteDb,
    StuckSignalInput, SuggestSignalInput, TaskProvenance, TaskStatus,
};
use std::fs;
use std::path::{Path, PathBuf};

fn stack_02_launch_defaults(
    discipline_name: &str,
) -> (Option<String>, Option<String>, Option<String>, Option<bool>) {
    match discipline_name {
        "frontend" | "backend" | "integration" | "platform" => (
            Some("codex".to_owned()),
            Some("gpt-5.3-codex".to_owned()),
            None,
            Some(true),
        ),
        "data" => (
            Some("claude".to_owned()),
            Some("sonnet".to_owned()),
            None,
            Some(true),
        ),
        "documentation" => (None, None, None, None),
        "quality" | "security" => (
            Some("claude".to_owned()),
            Some("opus".to_owned()),
            Some("high".to_owned()),
            Some(true),
        ),
        _ => panic!("Unexpected stack-02 discipline: {discipline_name}"),
    }
}

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
        let (agent, model, effort, thinking) = stack_02_launch_defaults(&d.name);

        db.create_discipline(sqlite_db::DisciplineInput {
            name: d.name,
            display_name: d.display_name,
            acronym: d.acronym,
            icon: d.icon,
            color: d.color,
            system_prompt: Some(d.system_prompt),
            agent,
            model,
            effort,
            thinking,
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

- Execution sequence starts with no tasks (clean slate)
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

**Purpose**: Desktop stack project with features and tasks (ready for execution sequence)

This fixture shows a complete project ready for Ralph task execution to run.
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

- Test task execution sequencing
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
            agent: None,
            model: None,
            effort: None,
            thinking: None,
    })
    .unwrap();

    db.add_signal(
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
        agent: None,
        model: None,
        effort: None,
        thinking: None,
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
        agent: None,
        model: None,
        effort: None,
        thinking: None,
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
            agent: Some("claude".to_owned()),
            model: Some("sonnet".to_owned()),
            effort: None,
            thinking: Some(true),
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
            agent: Some("claude".to_owned()),
            model: Some("opus".to_owned()),
            effort: Some("high".to_owned()),
            thinking: Some(true),
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
            agent: None,
            model: None,
            effort: None,
            thinking: None,
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
            agent: None,
            model: None,
            effort: None,
            thinking: None,
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
            agent: None,
            model: None,
            effort: None,
            thinking: None,
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
            agent: None,
            model: None,
            effort: None,
            thinking: None,
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
            agent: None,
            model: None,
            effort: None,
            thinking: None,
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
            agent: None,
            model: None,
            effort: None,
            thinking: None,
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
            agent: None,
            model: None,
            effort: None,
            thinking: None,
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
            agent: None,
            model: None,
            effort: None,
            thinking: None,
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
            agent: None,
            model: None,
            effort: None,
            thinking: None,
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
            agent: None,
            model: None,
            effort: None,
            thinking: None,
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
            agent: None,
            model: None,
            effort: None,
            thinking: None,
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
            agent: None,
            model: None,
            effort: None,
            thinking: None,
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
            agent: None,
            model: None,
            effort: None,
            thinking: None,
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
            agent: None,
            model: None,
            effort: None,
            thinking: None,
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
            agent: None,
            model: None,
            effort: None,
            thinking: None,
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
            agent: None,
            model: None,
            effort: None,
            thinking: None,
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
            agent: None,
            model: None,
            effort: None,
            thinking: None,
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
            agent: None,
            model: None,
            effort: None,
            thinking: None,
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
            agent: None,
            model: None,
            effort: None,
            thinking: None,
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

    db.add_signal(
        1,
        Some("frontend".to_owned()),
        None,
        None,
        "Card layout finalized, using 3-column grid on desktop.".to_owned(),
    )
    .unwrap();
    db.add_signal(
        2,
        Some("frontend".to_owned()),
        None,
        None,
        "Auto-title fetch uses og:title with URL fallback.".to_owned(),
    )
    .unwrap();
    db.add_signal(
        3,
        Some("backend".to_owned()),
        Some(3),
        None,
        "localStorage wrapper handles quota errors with LRU eviction.".to_owned(),
    )
    .unwrap();
    db.add_signal(
        4,
        Some("quality".to_owned()),
        Some(4),
        Some("high".to_owned()),
        "Found edge case: empty URL string passes validation. Adding test.".to_owned(),
    )
    .unwrap();
    db.add_signal(
        4,
        None,
        None,
        None,
        "Also test unicode URLs please.".to_owned(),
    )
    .unwrap();
    db.add_signal(
        7,
        Some("security".to_owned()),
        None,
        Some("high".to_owned()),
        "Added CSP header and input sanitization for all URL fields.".to_owned(),
    )
    .unwrap();
    db.add_signal(
        8,
        Some("backend".to_owned()),
        Some(8),
        None,
        "Schema uses JSON column for bookmark refs, supports ordering.".to_owned(),
    )
    .unwrap();
    db.add_signal(
        9,
        Some("frontend".to_owned()),
        Some(9),
        Some("medium".to_owned()),
        "Sidebar uses virtual scroll for collections > 50.".to_owned(),
    )
    .unwrap();
    db.add_signal(
        9,
        Some("frontend".to_owned()),
        Some(9),
        None,
        "Collapse state persisted in localStorage.".to_owned(),
    )
    .unwrap();
    db.add_signal(
        13,
        Some("backend".to_owned()),
        None,
        None,
        "Evaluating lunr.js vs custom inverted index. lunr.js is 8kb gzipped.".to_owned(),
    )
    .unwrap();
    db.add_signal(
        14,
        None,
        None,
        Some("low".to_owned()),
        "Blocked until search index is ready. Low priority for now.".to_owned(),
    )
    .unwrap();
    db.add_signal(
        16,
        Some("backend".to_owned()),
        None,
        None,
        "Chrome and Firefox use same Netscape format. Safari differs slightly.".to_owned(),
    )
    .unwrap();
    db.add_signal(
        16,
        Some("quality".to_owned()),
        None,
        None,
        "Need sample export files from each browser for test fixtures.".to_owned(),
    )
    .unwrap();
    db.add_signal(
        17,
        None,
        None,
        None,
        "Blocked on HTML parser. Will design the UI in parallel once unblocked.".to_owned(),
    )
    .unwrap();
    db.add_signal(
        20,
        Some("data".to_owned()),
        None,
        None,
        "Using IndexedDB for theme + future settings. localStorage too limited.".to_owned(),
    )
    .unwrap();

    // Set varied statuses with historical dates (fixture-only)
    // Tasks 1,2,3,8 = completed; Tasks 4,9 = in_progress; Tasks 14,17 = blocked; Task 19 = skipped
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

    // --- Task Comments with Signal Verbs ---
    // Task 21: MCP Signal Reference - ALL 8 VERBS + VARIANTS (15 examples)
    // Signals from DIFFERENT disciplines to demonstrate cross-discipline communication

    let sess = "sess-21-ref";

    // 1. DONE verb - Frontend completes their work
    db.insert_done_signal(
        Some("frontend"),
        DoneSignalInput {
            task_id: 21,
            session_id: sess.to_owned(),
            summary: "Implemented CSV/JSON export with quota handling, all 3 acceptance criteria pass, tests green, PR merged.".to_owned(),
        },
    )
    .unwrap();

    // 2. PARTIAL verb - Backend reports partial progress
    db.insert_partial_signal(
        Some("backend"),
        PartialSignalInput {
            task_id: 21,
            session_id: sess.to_owned(),
            summary: "Implemented CSV export format, added filename sanitization, wrote 12 of 18 unit tests.".to_owned(),
            remaining: "JSON export format, bulk export UI, remaining 6 edge-case tests.".to_owned(),
        },
    )
    .unwrap();

    // 3. STUCK verb - Quality team is stuck
    db.insert_stuck_signal(
        Some("quality"),
        StuckSignalInput {
            task_id: 21,
            session_id: sess.to_owned(),
            reason: "Cannot determine correct MIME type for .bookmark files — RFC 7231 spec is ambiguous and Chrome/Firefox behave differently.".to_owned(),
        },
    )
    .unwrap();

    // 4. ASK verb (BLOCKING variant) - Frontend asks for decision
    db.insert_ask_signal(
        Some("frontend"),
        AskSignalInput {
            task_id: 21,
            session_id: sess.to_owned(),
            question: "Should export filename include timestamp (bookmarks-2025-01-22.json) or use static name (bookmarks.json)?".to_owned(),
            blocking: true,
            options: Some(vec![
                "Add timestamp".to_owned(),
                "Static filename".to_owned(),
                "Let user choose via dialog".to_owned(),
            ]),
            preferred: Some("Add timestamp".to_owned()),
        },
    )
    .unwrap();

    // 5. ASK verb (NON-BLOCKING variant) - Backend asks for input
    db.insert_ask_signal(
        Some("backend"),
        AskSignalInput {
            task_id: 21,
            session_id: sess.to_owned(),
            question: "Should we use .csv or .tsv extension for tab-separated exports?".to_owned(),
            blocking: false,
            options: Some(vec![
                "Use .csv (more common)".to_owned(),
                "Use .tsv (more accurate)".to_owned(),
            ]),
            preferred: Some("Use .tsv (more accurate)".to_owned()),
        },
    )
    .unwrap();

    // 6-8. FLAG verb (3 severity variants) - Different disciplines flag issues
    db.insert_flag_signal(
        Some("frontend"),
        FlagSignalInput {
            task_id: 21,
            session_id: sess.to_owned(),
            what: "Export button downloads empty 0-byte file when localStorage.getItem() returns null.".to_owned(),
            severity: "blocking".to_owned(),
            category: "bug".to_owned(),
        },
    )
    .unwrap();

    db.insert_flag_signal(
        Some("backend"),
        FlagSignalInput {
            task_id: 21,
            session_id: sess.to_owned(),
            what: "Export with 10k+ bookmarks takes 3+ seconds and freezes UI thread.".to_owned(),
            severity: "warning".to_owned(),
            category: "performance".to_owned(),
        },
    )
    .unwrap();

    db.insert_flag_signal(
        Some("quality"),
        FlagSignalInput {
            task_id: 21,
            session_id: sess.to_owned(),
            what: "Export filename gets truncated to 255 chars on older Windows filesystems."
                .to_owned(),
            severity: "info".to_owned(),
            category: "ambiguity".to_owned(),
        },
    )
    .unwrap();

    // 9-11. LEARNED verb (3 kind variants) - Different disciplines share learnings
    db.insert_learned_signal(
        Some("frontend"),
        LearnedSignalInput {
            task_id: 21,
            session_id: sess.to_owned(),
            text: "Browser download APIs create Blob URLs that must be manually revoked via URL.revokeObjectURL() or they leak memory until page reload.".to_owned(),
            kind: "discovery".to_owned(),
            scope: "feature".to_owned(),
            rationale: Some("Without cleanup, every export leaks ~50KB. User with 100 exports = 5MB leaked RAM.".to_owned()),
        },
    )
    .unwrap();

    db.insert_learned_signal(
        Some("backend"),
        LearnedSignalInput {
            task_id: 21,
            session_id: sess.to_owned(),
            text: "Wrap all localStorage operations in try/catch with LRU eviction fallback — handles quota errors gracefully.".to_owned(),
            kind: "convention".to_owned(),
            scope: "task".to_owned(),
            rationale: None,
        },
    )
    .unwrap();

    db.insert_learned_signal(
        Some("data"),
        LearnedSignalInput {
            task_id: 21,
            session_id: sess.to_owned(),
            text: "All export filenames follow pattern: appname-entity-timestamp.ext (e.g., ralph-bookmarks-2025-01-22.json).".to_owned(),
            kind: "convention".to_owned(),
            scope: "feature".to_owned(),
            rationale: None,
        },
    )
    .unwrap();

    // 12-13. SUGGEST verb (2 kind variants) - Different disciplines suggest improvements
    db.insert_suggest_signal(
        Some("frontend"),
        SuggestSignalInput {
            task_id: 21,
            session_id: sess.to_owned(),
            what: "Add Copy to Clipboard button next to Download — users want to paste bookmark JSON into Slack without saving a file.".to_owned(),
            kind: "alternative".to_owned(),
            why: "40% of exports in analytics are followed by manual file-open-copy-paste. Direct clipboard = better UX.".to_owned(),
        },
    )
    .unwrap();

    db.insert_suggest_signal(
        Some("backend"),
        SuggestSignalInput {
            task_id: 21,
            session_id: sess.to_owned(),
            what: "Add scheduled auto-export that backs up bookmarks to user's chosen cloud storage every 24 hours.".to_owned(),
            kind: "new_task".to_owned(),
            why: "Users in support threads frequently ask about backup/sync. Auto-export prevents data loss.".to_owned(),
        },
    )
    .unwrap();

    // 14-15. BLOCKED verb (2 kind variants) - Different disciplines report blockers
    db.insert_blocked_signal(
        Some("frontend"),
        BlockedSignalInput {
            task_id: 21,
            session_id: sess.to_owned(),
            on: "Need backend API endpoint POST /export/stream for server-side export generation.".to_owned(),
            kind: "upstream_task".to_owned(),
            detail: Some("Client-side export works for <1000 bookmarks but crashes tab with larger datasets. Need streaming response.".to_owned()),
        },
    )
    .unwrap();

    db.insert_blocked_signal(
        Some("platform"),
        BlockedSignalInput {
            task_id: 21,
            session_id: sess.to_owned(),
            on: "Bun v1.2+ required for File System Access API — CI still running Bun v1.0.15."
                .to_owned(),
            kind: "external".to_owned(),
            detail: None,
        },
    )
    .unwrap();

    println!(
        "\n✓ Created 04-desktop-dev fixture at: {}",
        fixture_path.display()
    );
    println!(
        "  5 features, 21 tasks (4 completed, 3 in_progress, 11 pending, 2 blocked, 1 skipped)"
    );
    println!("  15 comment examples on task 21 showing all 8 verbs with variants");
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
