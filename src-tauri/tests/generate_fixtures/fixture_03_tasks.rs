use super::helpers::{initialize_project_for_fixture, open_fixture_db};
use sqlite_db::SubsystemInput;
use std::fs;
use std::path::PathBuf;

pub(crate) fn generate_fixture_03_desktop_tasks() {
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

**Purpose**: Desktop stack project with subsystems, tasks, and a couple routine templates.

This fixture shows a complete project ready for Ralph task execution to run.
It has subsystems defined and tasks created, all using Desktop stack disciplines.

## Usage

```bash
# Generate fixtures
cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures -- --nocapture

# Reset mock and use
just reset-mock
just dev-mock 03-desktop-tasks
```

## Contents

- `.undetect-ralph/db/ralph.db` - SQLite database (2 subsystems, 3 tasks, 2 templates, 8 Desktop disciplines)
- `.undetect-ralph/images/disciplines/` - Discipline portrait images

## Tasks

1. **authentication/backend** - Implement login API (high priority)
2. **authentication/frontend** - Build login form (depends on #1)
3. **user-profile/frontend** - Create profile page

## Use Cases

- Test task execution sequencing
- Monkey testing with real task data
- Verify task dependency handling
- Test multi-subsystem projects
";

    fs::write(fixture_path.join("README.md"), readme).unwrap();

    // Initialize
    initialize_project_for_fixture(fixture_path.clone(), "Desktop Tasks".to_owned(), true).unwrap();

    let db = open_fixture_db(&fixture_path);

    // Add subsystems
    db.create_subsystem(SubsystemInput {
        name: "authentication".to_owned(),
        display_name: "Authentication".to_owned(),
        acronym: "AUTH".to_owned(),
        ..Default::default()
    })
    .unwrap();
    db.create_subsystem(SubsystemInput {
        name: "user-profile".to_owned(),
        display_name: "User Profile".to_owned(),
        acronym: "USPR".to_owned(),
        ..Default::default()
    })
    .unwrap();

    // Seed a few routine templates (discipline-bound, reusable).
    {
        use rusqlite::{params, Connection};
        let conn = Connection::open(fixture_path.join(".undetect-ralph/db/ralph.db")).unwrap();
        let templates = [
            (
                "backend",
                "Routine auth contract check",
                "Verify auth request/response contracts before merge.",
                "high",
            ),
            (
                "frontend",
                "Routine form UX pass",
                "Run focused UX pass on auth forms and error states.",
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

    // Add tasks
    db.create_task(TaskInput {
        subsystem: "authentication".to_owned(),
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
        subsystem: "authentication".to_owned(),
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
        subsystem: "user-profile".to_owned(),
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
