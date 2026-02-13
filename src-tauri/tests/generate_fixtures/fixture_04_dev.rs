use super::helpers::{initialize_project_for_fixture, open_fixture_db};
use sqlite_db::{
    AddSubsystemCommentInput, AskSignalInput, BlockedSignalInput, DoneSignalInput, FlagSignalInput,
    LearnedSignalInput, PartialSignalInput, StuckSignalInput, SubsystemInput, SuggestSignalInput,
    TaskProvenance, TaskStatus,
};
use std::fs;
use std::path::PathBuf;

pub(crate) fn generate_fixture_04_desktop_dev() {
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

20 tasks across 5 subsystems and 7 disciplines, plus routine templates. Desktop stack (stack 2) with discipline images.
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
- **SubsystemsPage**: 5 subsystems with varied completion %
- **DisciplinesPage**: 8 Desktop stack disciplines with images
- **Task templates**: routine reusable templates bound to disciplines
- **Filters**: 14 distinct tags, every status/priority combo, text search on titles+descriptions
- **TaskIdDisplay**: multiple subsystem+discipline acronym combos
";

    fs::write(fixture_path.join("README.md"), readme).unwrap();

    // Initialize project structure + default disciplines
    initialize_project_for_fixture(fixture_path.clone(), "Bookmarks Manager".to_owned(), true)
        .unwrap();

    let db = open_fixture_db(&fixture_path);

    // --- Subsystems ---
    db.create_subsystem(SubsystemInput {
        name: "bookmark-crud".to_owned(),
        display_name: "Bookmark CRUD".to_owned(),
        acronym: "BKMK".to_owned(),
        description: Some("Core bookmark create, read, update, delete operations".to_owned()),
    })
    .unwrap();
    db.create_subsystem(SubsystemInput {
        name: "collections".to_owned(),
        display_name: "Collections".to_owned(),
        acronym: "COLL".to_owned(),
        description: Some("Organize bookmarks into named collections".to_owned()),
    })
    .unwrap();
    db.create_subsystem(SubsystemInput {
        name: "search".to_owned(),
        display_name: "Search".to_owned(),
        acronym: "SRCH".to_owned(),
        description: Some("Full-text search and filtering across bookmarks".to_owned()),
    })
    .unwrap();
    db.create_subsystem(SubsystemInput {
        name: "import-export".to_owned(),
        display_name: "Import Export".to_owned(),
        acronym: "IMEX".to_owned(),
        description: Some("Import from HTML, export to JSON".to_owned()),
    })
    .unwrap();
    db.create_subsystem(SubsystemInput {
        name: "settings".to_owned(),
        display_name: "Settings".to_owned(),
        acronym: "STNG".to_owned(),
        description: Some("User preferences and theme configuration".to_owned()),
    })
    .unwrap();

    // Seed routine templates used by disciplines.
    {
        use rusqlite::{params, Connection};
        let conn = Connection::open(fixture_path.join(".undetect-ralph/db/ralph.db")).unwrap();
        let templates = [
            (
                "quality",
                "Routine regression sweep",
                "Run smoke + core regression checks for active flows.",
                "high",
            ),
            (
                "security",
                "Routine dependency audit",
                "Audit dependency set and flag known vulnerabilities.",
                "high",
            ),
            (
                "documentation",
                "Routine changelog pass",
                "Summarize user-facing changes and rollout notes.",
                "low",
            ),
            (
                "data",
                "Routine schema sanity pass",
                "Review constraints/indexes against current workload.",
                "medium",
            ),
        ];
        for (discipline, title, description, priority) in templates {
            let discipline_id: i64 = conn
                .query_row(
                    "SELECT id FROM disciplines WHERE name = ?1",
                    [discipline],
                    |row| row.get::<_, i64>(0),
                )
                .unwrap();
            conn.execute(
                "INSERT INTO task_details (discipline_id, title, description, priority, created) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![discipline_id, title, description, priority, "2026-01-01"],
            )
            .unwrap();
            let details_id = conn.last_insert_rowid();
            conn.execute(
                "INSERT INTO task_templates (details_id, is_active, created) VALUES (?1, 1, ?2)",
                params![details_id, "2026-01-01"],
            )
            .unwrap();
        }
    }

    // --- Subsystem Comments ---
    db.add_subsystem_comment(AddSubsystemCommentInput {
        subsystem_name: "bookmark-crud".to_owned(),
        category: "design-decision".to_owned(),
        discipline: None,
        agent_task_id: None,
        body: "Use optimistic updates for bookmark creation to avoid UI lag".to_owned(),
        summary: Some("Use optimistic updates for creates".to_owned()),
        reason: Some("Network latency makes synchronous saves feel sluggish".to_owned()),
        source_iteration: None,
    })
    .unwrap();
    db.add_subsystem_comment(AddSubsystemCommentInput {
        subsystem_name: "bookmark-crud".to_owned(),
        category: "gotcha".to_owned(),
        discipline: Some("frontend".to_owned()),
        agent_task_id: Some(2),
        body: "Favicon URLs often 404 — always provide a fallback icon".to_owned(),
        summary: None,
        reason: None,
        source_iteration: Some(1),
    })
    .unwrap();
    db.add_subsystem_comment(AddSubsystemCommentInput {
        subsystem_name: "bookmark-crud".to_owned(),
        category: "convention".to_owned(),
        discipline: Some("backend".to_owned()),
        agent_task_id: Some(3),
        body: "All bookmark IDs are ULIDs, not auto-increment integers. This keeps them sortable by creation time without a separate timestamp index.".to_owned(),
        summary: Some("ULIDs for bookmark IDs".to_owned()),
        reason: Some("Sortable by creation time without extra index".to_owned()),
        source_iteration: Some(1),
    })
    .unwrap();
    db.add_subsystem_comment(AddSubsystemCommentInput {
        subsystem_name: "bookmark-crud".to_owned(),
        category: "architecture".to_owned(),
        discipline: Some("backend".to_owned()),
        agent_task_id: None,
        body: "Bookmark storage uses a write-ahead log pattern: mutations go to a WAL table first, then get compacted into the main bookmarks table on read. This avoids locking during writes.".to_owned(),
        summary: Some("WAL pattern for bookmark writes".to_owned()),
        reason: Some("Avoids write locks on the main table".to_owned()),
        source_iteration: None,
    })
    .unwrap();
    db.add_subsystem_comment(AddSubsystemCommentInput {
        subsystem_name: "bookmark-crud".to_owned(),
        category: "gotcha".to_owned(),
        discipline: Some("quality".to_owned()),
        agent_task_id: Some(4),
        body: "URL normalization strips trailing slashes and lowercases the hostname, but preserves path case. Two URLs that look different may be the same bookmark after normalization.".to_owned(),
        summary: None,
        reason: None,
        source_iteration: Some(2),
    })
    .unwrap();
    db.add_subsystem_comment(AddSubsystemCommentInput {
        subsystem_name: "bookmark-crud".to_owned(),
        category: "boundary".to_owned(),
        discipline: Some("security".to_owned()),
        agent_task_id: None,
        body: "Never store raw user-provided HTML in bookmark notes. All note content goes through DOMPurify before persistence.".to_owned(),
        summary: Some("Sanitize notes with DOMPurify".to_owned()),
        reason: Some("Prevents stored XSS via bookmark notes".to_owned()),
        source_iteration: None,
    })
    .unwrap();
    db.add_subsystem_comment(AddSubsystemCommentInput {
        subsystem_name: "bookmark-crud".to_owned(),
        category: "dependency".to_owned(),
        discipline: Some("frontend".to_owned()),
        agent_task_id: None,
        body: "The bookmark card component depends on the favicon proxy service. If the proxy is down, cards should render with a generic globe icon instead of breaking.".to_owned(),
        summary: Some("Favicon proxy fallback to globe icon".to_owned()),
        reason: None,
        source_iteration: None,
    })
    .unwrap();
    db.add_subsystem_comment(AddSubsystemCommentInput {
        subsystem_name: "bookmark-crud".to_owned(),
        category: "design-decision".to_owned(),
        discipline: Some("frontend".to_owned()),
        agent_task_id: Some(5),
        body: "Edit modal uses a sheet sliding in from the right, not a centered dialog. This keeps the bookmark list visible for context while editing.".to_owned(),
        summary: Some("Sheet for edit, not dialog".to_owned()),
        reason: Some("User can see the list while editing".to_owned()),
        source_iteration: Some(3),
    })
    .unwrap();
    db.add_subsystem_comment(AddSubsystemCommentInput {
        subsystem_name: "bookmark-crud".to_owned(),
        category: "gotcha".to_owned(),
        discipline: Some("data".to_owned()),
        agent_task_id: None,
        body: "Bulk delete must cascade to collection membership. Deleting a bookmark that belongs to 3 collections needs to clean up all 3 junction rows.".to_owned(),
        summary: None,
        reason: Some("Orphaned junction rows cause ghost counts in collection sidebar".to_owned()),
        source_iteration: None,
    })
    .unwrap();
    db.add_subsystem_comment(AddSubsystemCommentInput {
        subsystem_name: "bookmark-crud".to_owned(),
        category: "convention".to_owned(),
        discipline: Some("platform".to_owned()),
        agent_task_id: None,
        body: "All CRUD endpoints follow the pattern: POST /bookmarks, GET /bookmarks/:id, PATCH /bookmarks/:id, DELETE /bookmarks/:id. No PUT — partial updates only.".to_owned(),
        summary: Some("PATCH for updates, no PUT".to_owned()),
        reason: Some("Partial updates reduce payload size and merge conflicts".to_owned()),
        source_iteration: None,
    })
    .unwrap();
    db.add_subsystem_comment(AddSubsystemCommentInput {
        subsystem_name: "bookmark-crud".to_owned(),
        category: "architecture".to_owned(),
        discipline: Some("frontend".to_owned()),
        agent_task_id: Some(1),
        body: "Bookmark list uses virtual scrolling via tanstack-virtual. Only visible cards are in the DOM. This is critical — users with 5k+ bookmarks were hitting 2s render times without it.".to_owned(),
        summary: Some("Virtual scrolling for large lists".to_owned()),
        reason: Some("5k+ bookmarks caused 2s render without virtualization".to_owned()),
        source_iteration: Some(1),
    })
    .unwrap();
    db.add_subsystem_comment(AddSubsystemCommentInput {
        subsystem_name: "bookmark-crud".to_owned(),
        category: "boundary".to_owned(),
        discipline: Some("backend".to_owned()),
        agent_task_id: None,
        body: "Maximum bookmark title length is 500 chars, URL is 2048 chars, notes is 10000 chars. Enforce at both API and DB constraint level.".to_owned(),
        summary: Some("Field length limits: title 500, URL 2048, notes 10k".to_owned()),
        reason: None,
        source_iteration: None,
    })
    .unwrap();
    db.add_subsystem_comment(AddSubsystemCommentInput {
        subsystem_name: "bookmark-crud".to_owned(),
        category: "design-decision".to_owned(),
        discipline: None,
        agent_task_id: None,
        body: "Soft delete with 30-day trash retention. Deleted bookmarks move to a trash view and auto-purge after 30 days. Users can restore from trash.".to_owned(),
        summary: Some("Soft delete with 30-day trash".to_owned()),
        reason: Some("Prevents accidental permanent data loss".to_owned()),
        source_iteration: None,
    })
    .unwrap();
    db.add_subsystem_comment(AddSubsystemCommentInput {
        subsystem_name: "collections".to_owned(),
        category: "architecture".to_owned(),

        discipline: None,
        agent_task_id: None,
        body: "Collections are flat, not nested — no recursive trees".to_owned(),
        summary: Some("Flat collections only, no nesting".to_owned()),
        reason: Some("Keeps the data model simple and avoids infinite depth bugs".to_owned()),
        source_iteration: None,
    })
    .unwrap();
    db.add_subsystem_comment(AddSubsystemCommentInput {
        subsystem_name: "collections".to_owned(),
        category: "convention".to_owned(),

        discipline: None,
        agent_task_id: None,
        body: "Collection names are unique per user, case-insensitive".to_owned(),
        summary: None,
        reason: None,
        source_iteration: None,
    })
    .unwrap();
    db.add_subsystem_comment(AddSubsystemCommentInput {
        subsystem_name: "search".to_owned(),
        category: "design-decision".to_owned(),

        discipline: None,
        agent_task_id: None,
        body: "Use SQLite FTS5 for full-text search instead of client-side filtering".to_owned(),
        summary: Some("Use FTS5 for search".to_owned()),
        reason: Some("Scales better with large bookmark collections".to_owned()),
        source_iteration: None,
    })
    .unwrap();
    db.add_subsystem_comment(AddSubsystemCommentInput {
        subsystem_name: "search".to_owned(),
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
            subsystem: "bookmark-crud".to_owned(),
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
            subsystem: "bookmark-crud".to_owned(),
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
            subsystem: "bookmark-crud".to_owned(),
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
            subsystem: "bookmark-crud".to_owned(),
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
            subsystem: "bookmark-crud".to_owned(),
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
            subsystem: "bookmark-crud".to_owned(),
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
            subsystem: "bookmark-crud".to_owned(),
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
            subsystem: "collections".to_owned(),
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
        subsystem: "collections".to_owned(),
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
            subsystem: "collections".to_owned(),
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
            subsystem: "collections".to_owned(),
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
            subsystem: "collections".to_owned(),
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
        subsystem: "search".to_owned(),
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
            subsystem: "search".to_owned(),
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
            subsystem: "search".to_owned(),
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
            subsystem: "import-export".to_owned(),
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
            subsystem: "import-export".to_owned(),
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
            subsystem: "import-export".to_owned(),
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
            subsystem: "settings".to_owned(),
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
            subsystem: "settings".to_owned(),
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
            subsystem: "bookmark-crud".to_owned(),
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
            scope: "subsystem".to_owned(),
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
            scope: "subsystem".to_owned(),
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
        "  5 subsystems, 21 tasks (4 completed, 3 in_progress, 11 pending, 2 blocked, 1 skipped)"
    );
    println!("  15 comment examples on task 21 showing all 8 verbs with variants");
    println!("  → Task #21: MCP Signal Reference with ALL 8 VERBS + VARIANTS (15 signals)");
}
