use sqlite_db::SqliteDb;
use std::fs;
use std::path::PathBuf;

pub(crate) fn stack_02_launch_defaults(
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

/// Test-support helper to initialize fixture projects with Ralph metadata, DB, and seeded disciplines.
pub(crate) fn initialize_project_for_fixture(
    path: PathBuf,
    project_title: String,
    use_undetect: bool,
    clock: Option<Box<dyn sqlite_db::Clock>>,
) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("Directory not found: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("Not a directory: {}", path.display()));
    }

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

    let images_dir = ralph_dir.join("images").join("disciplines");
    let _ = fs::create_dir_all(&images_dir);

    let db_path = db_dir.join("ralph.db");
    let db = SqliteDb::open(&db_path, clock)?;

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
