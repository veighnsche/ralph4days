use super::helpers::{initialize_project_for_fixture, open_fixture_db};
use rusqlite::{params, Connection};
use sqlite_db::{SubsystemInput, TaskInput, TaskProvenance, TaskStatus};
use std::fs;
use std::path::PathBuf;

pub(crate) fn generate_fixture_05_desktop_templates() {
    println!("\n=== Generating fixture: 05-desktop-templates ===");

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("fixtures");
    fs::create_dir_all(&fixtures_dir).unwrap();

    let fixture_path = fixtures_dir.join("05-desktop-templates");
    if fixture_path.exists() {
        fs::remove_dir_all(&fixture_path).unwrap();
    }
    fs::create_dir_all(&fixture_path).unwrap();

    let readme = "# Desktop Templates

**Purpose**: Desktop stack project with routine task templates (no hook system required yet).

This fixture seeds `task_details` + `task_templates` for multiple disciplines, then creates
a few runtime tasks instantiated from those templates (pull model).

## Usage

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures generate_fixture_05_desktop_templates -- --nocapture
just reset-mock
just dev-mock 05-desktop-templates
```

## Contents

- `.undetect-ralph/db/ralph.db`
  - 3 subsystems
  - 6 active task templates bound to disciplines
  - 3 runtime tasks instantiated from templates

## Intent

- Demonstrate reusable routine templates before hooks exist
- Keep templates as persistent pending definitions
- Show runtime tasks pulling from templates via `template_id`
";
    fs::write(fixture_path.join("README.md"), readme).unwrap();

    initialize_project_for_fixture(fixture_path.clone(), "Desktop Templates".to_owned(), true)
        .unwrap();

    let db = open_fixture_db(&fixture_path);
    db.create_subsystem(SubsystemInput {
        name: "project-hygiene".to_owned(),
        display_name: "Project Hygiene".to_owned(),
        acronym: "HYGN".to_owned(),
        description: Some("Recurring routine work for consistency and quality.".to_owned()),
    })
    .unwrap();
    db.create_subsystem(SubsystemInput {
        name: "release-ops".to_owned(),
        display_name: "Release Ops".to_owned(),
        acronym: "RLOP".to_owned(),
        description: Some("Routine release readiness and validation.".to_owned()),
    })
    .unwrap();
    db.create_subsystem(SubsystemInput {
        name: "knowledge-base".to_owned(),
        display_name: "Knowledge Base".to_owned(),
        acronym: "KNOW".to_owned(),
        description: Some("Routine docs and learnings upkeep.".to_owned()),
    })
    .unwrap();

    let db_path = fixture_path
        .join(".undetect-ralph")
        .join("db")
        .join("ralph.db");
    let conn = Connection::open(&db_path).unwrap();

    let templates = [
        (
            "quality",
            "Run routine regression sweep",
            Some("Execute routine smoke + core regression checks before merge.".to_owned()),
            Some("high".to_owned()),
            Some("Focus on high-risk paths first.".to_owned()),
            Some(3_u32),
            Some("claude".to_owned()),
            Some("opus".to_owned()),
            Some("high".to_owned()),
            Some(true),
            Some("Enumerate critical user journeys and validate each one.".to_owned()),
        ),
        (
            "security",
            "Routine dependency audit",
            Some("Audit dependency changes and flag vulnerable packages.".to_owned()),
            Some("high".to_owned()),
            Some("Prioritize direct dependencies.".to_owned()),
            Some(2_u32),
            Some("claude".to_owned()),
            Some("opus".to_owned()),
            Some("high".to_owned()),
            Some(true),
            Some("Identify changed deps, then compare against known advisories.".to_owned()),
        ),
        (
            "backend",
            "Routine contract compatibility check",
            Some("Validate backend API contracts against current frontend usage.".to_owned()),
            Some("medium".to_owned()),
            Some("Flag breaking changes explicitly.".to_owned()),
            Some(2_u32),
            Some("codex".to_owned()),
            Some("gpt-5.3-codex".to_owned()),
            None,
            Some(true),
            Some("Compare current API shapes with consumer expectations.".to_owned()),
        ),
        (
            "frontend",
            "Routine UX polish pass",
            Some("Run a standard UX polish pass across active flows.".to_owned()),
            Some("medium".to_owned()),
            Some("Focus on friction and clarity.".to_owned()),
            Some(2_u32),
            Some("codex".to_owned()),
            Some("gpt-5.3-codex".to_owned()),
            None,
            Some(true),
            Some("Document friction points and propose precise UI adjustments.".to_owned()),
        ),
        (
            "documentation",
            "Routine changelog update",
            Some("Capture user-facing changes and operational notes.".to_owned()),
            Some("low".to_owned()),
            Some("Keep entries concise and actionable.".to_owned()),
            Some(1_u32),
            None,
            None,
            None,
            None,
            Some("Summarize changes by impact and rollout notes.".to_owned()),
        ),
        (
            "data",
            "Routine schema sanity check",
            Some("Validate constraints and key indexes still reflect workload.".to_owned()),
            Some("medium".to_owned()),
            Some("Include migration edge cases.".to_owned()),
            Some(2_u32),
            Some("claude".to_owned()),
            Some("sonnet".to_owned()),
            None,
            Some(true),
            Some("Review constraints/indexes and test representative queries.".to_owned()),
        ),
    ];

    let mut template_ids: Vec<i64> = vec![];
    for (
        discipline_name,
        title,
        description,
        priority,
        hints,
        estimated_turns,
        agent,
        model,
        effort,
        thinking,
        pseudocode,
    ) in templates
    {
        let discipline_id: i64 = conn
            .query_row(
                "SELECT id FROM disciplines WHERE name = ?1",
                [discipline_name],
                |row| row.get::<_, i64>(0),
            )
            .unwrap();

        conn.execute(
            "INSERT INTO task_details (discipline_id, title, description, priority, hints, estimated_turns, agent, model, effort, thinking, pseudocode, created) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                discipline_id,
                title,
                description,
                priority,
                hints,
                estimated_turns,
                agent,
                model,
                effort,
                thinking,
                pseudocode,
                "2026-01-01"
            ],
        )
        .unwrap();
        let details_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO task_templates (details_id, is_active, created) VALUES (?1, 1, ?2)",
            params![details_id, "2026-01-01"],
        )
        .unwrap();
        template_ids.push(conn.last_insert_rowid());
    }

    // Instantiate 3 runtime tasks from templates (pull model demonstration).
    let pulls = [
        ("project-hygiene", template_ids[0]), // quality routine
        ("release-ops", template_ids[1]),     // security routine
        ("knowledge-base", template_ids[4]),  // documentation routine
    ];

    for (subsystem_name, template_id) in pulls {
        let subsystem_id: i64 = conn
            .query_row(
                "SELECT id FROM subsystems WHERE name = ?1",
                [subsystem_name],
                |row| row.get::<_, i64>(0),
            )
            .unwrap();
        let details_id: i64 = conn
            .query_row(
                "SELECT details_id FROM task_templates WHERE id = ?1",
                [template_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap();

        conn.execute(
            "INSERT INTO runtime_tasks (subsystem_id, details_id, template_id, status, provenance, created) \
             VALUES (?1, ?2, ?3, 'pending', 'system', ?4)",
            params![subsystem_id, details_id, template_id, "2026-01-01"],
        )
        .unwrap();
    }

    // Add one fully manual task to show templates are optional.
    db.create_task(TaskInput {
        subsystem: "project-hygiene".to_owned(),
        discipline: "platform".to_owned(),
        title: "Manual infra cleanup checklist".to_owned(),
        description: Some("Ad-hoc maintenance task not tied to a template.".to_owned()),
        status: Some(TaskStatus::Pending),
        priority: Some(sqlite_db::Priority::Low),
        tags: vec!["maintenance".to_owned()],
        depends_on: vec![],
        acceptance_criteria: Some(vec!["Checklist is complete and logged".to_owned()]),
        context_files: vec![],
        output_artifacts: vec![],
        hints: None,
        estimated_turns: Some(1),
        provenance: Some(TaskProvenance::Human),
        agent: None,
        model: None,
        effort: None,
        thinking: None,
    })
    .unwrap();

    println!(
        "✓ Created 05-desktop-templates fixture at: {}",
        fixture_path.display()
    );
    println!("  6 active templates, 4 runtime tasks (3 pulled + 1 manual)");
}
