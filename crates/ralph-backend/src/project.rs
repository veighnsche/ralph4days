use ralph_errors::{codes, ralph_err};
use ralph_macros::ipc_type;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::path::PathBuf;

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectValidatePathArgs {
    pub path: String,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectInitializeArgs {
    pub path: String,
    pub project_title: String,
    pub stack: u8,
}

fn seed_disciplines_for_stack(
    db: &sqlite_db::SqliteDb,
    stack: u8,
    ralph_dir: &Path,
) -> Result<(), String> {
    let defs = predefined_disciplines::get_disciplines_for_stack(stack);
    if defs.is_empty() && stack != 0 {
        return ralph_err!(
            codes::DISCIPLINE_OPS,
            "No disciplines defined for stack {stack}"
        );
    }

    let images_dir = ralph_dir.join("images").join("disciplines");
    std::fs::create_dir_all(&images_dir).map_err(|e| {
        ralph_errors::err_string(
            codes::FILESYSTEM,
            format!("Failed to create {}: {e}", images_dir.display()),
        )
    })?;

    for d in &defs {
        let skills_json = serde_json::to_string(&d.skills).map_err(|e| {
            ralph_errors::err_string(
                codes::DISCIPLINE_OPS,
                format!(
                    "Failed to serialize skills for discipline '{}': {e}",
                    d.name
                ),
            )
        })?;

        let image_path = match predefined_disciplines::get_discipline_image(stack, &d.name) {
            Some(bytes) => {
                let rel = format!("images/disciplines/{}.png", d.name);
                let abs = ralph_dir.join(&rel);
                std::fs::write(&abs, bytes).map_err(|e| {
                    ralph_errors::err_string(
                        codes::FILESYSTEM,
                        format!("Failed to write discipline image '{}': {e}", abs.display()),
                    )
                })?;
                Some(rel)
            }
            None => None,
        };

        let crops_json = d
            .crops
            .as_ref()
            .map(|crops| {
                serde_json::to_string(crops).map_err(|e| {
                    ralph_errors::err_string(
                        codes::DISCIPLINE_OPS,
                        format!("Failed to serialize crops for discipline '{}': {e}", d.name),
                    )
                })
            })
            .transpose()?;

        let image_prompt_json = d
            .image_prompt
            .as_ref()
            .map(|prompt| {
                serde_json::to_string(prompt).map_err(|e| {
                    ralph_errors::err_string(
                        codes::DISCIPLINE_OPS,
                        format!(
                            "Failed to serialize image_prompt for discipline '{}': {e}",
                            d.name
                        ),
                    )
                })
            })
            .transpose()?;

        db.create_discipline(sqlite_db::DisciplineInput {
            name: d.name.clone(),
            display_name: d.display_name.clone(),
            acronym: d.acronym.clone(),
            icon: d.icon.clone(),
            color: d.color.clone(),
            description: d.description.clone(),
            system_prompt: Some(d.system_prompt.clone()),
            agent: None,
            model: None,
            effort: None,
            thinking: None,
            skills: skills_json,
            conventions: Some(d.conventions.clone()),
            mcp_servers: "[]".to_owned(),
            image_path,
            crops: crops_json,
            image_prompt: image_prompt_json,
        })?;
    }

    Ok(())
}

pub fn project_initialize(args: ProjectInitializeArgs) -> Result<(), String> {
    let stack = args.stack;
    let project_title = args.project_title.clone();
    let path = PathBuf::from(&args.path);

    if !path.exists() {
        return ralph_err!(
            codes::PROJECT_PATH,
            "Directory not found: {}",
            path.display()
        );
    }
    if !path.is_dir() {
        return ralph_err!(codes::PROJECT_PATH, "Not a directory: {}", path.display());
    }

    let ralph_dir = path.join(".ralph");
    if ralph_dir.exists() {
        return ralph_err!(
            codes::PROJECT_INIT,
            ".ralph/ already exists at {}",
            path.display()
        );
    }

    std::fs::create_dir(&ralph_dir).map_err(|e| {
        ralph_errors::err_string(
            codes::PROJECT_INIT,
            format!("Failed to create .ralph/ directory: {e}"),
        )
    })?;

    let db_dir = ralph_dir.join("db");
    std::fs::create_dir(&db_dir).map_err(|e| {
        ralph_errors::err_string(
            codes::PROJECT_INIT,
            format!("Failed to create .ralph/db/ directory: {e}"),
        )
    })?;

    let db_path = db_dir.join("ralph.db");
    let db = sqlite_db::SqliteDb::open(&db_path, None)?;
    seed_disciplines_for_stack(&db, stack, &ralph_dir)?;

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

    std::fs::write(&claude_path, claude_template).map_err(|e| {
        ralph_errors::err_string(
            codes::FILESYSTEM,
            format!("Failed to create CLAUDE.RALPH.md: {e}"),
        )
    })?;

    Ok(())
}

pub fn validate_project_path(path: &Path) -> Result<(), String> {
    tracing::debug!(path = %path.display(), "Validating project path");

    if !path.exists() {
        tracing::error!(path = %path.display(), "Directory not found");
        return ralph_err!(
            codes::PROJECT_PATH,
            "Directory not found: {}",
            path.display()
        );
    }
    if !path.is_dir() {
        return ralph_err!(codes::PROJECT_PATH, "Not a directory: {}", path.display());
    }

    let ralph_dir = path.join(".ralph");
    if !ralph_dir.exists() {
        return ralph_err!(
            codes::PROJECT_PATH,
            "No .ralph/ folder. Initialize with:\n  ralph --init \"{}\"",
            path.display()
        );
    }
    if !ralph_dir.is_dir() {
        return ralph_err!(
            codes::PROJECT_PATH,
            "{} exists but is not a directory",
            ralph_dir.display()
        );
    }

    let db_file = ralph_dir.join("db").join("ralph.db");
    if !db_file.exists() {
        tracing::error!(path = %path.display(), "No .ralph/db/ralph.db found");
        return ralph_err!(
            codes::PROJECT_PATH,
            "No .ralph/db/ralph.db found. Initialize with:\n  ralph --init \"{}\"",
            path.display()
        );
    }

    tracing::info!(path = %path.display(), "Project path validated successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn project_initialize_creates_ralph_layout() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("proj");
        std::fs::create_dir(&root).unwrap();

        project_initialize(ProjectInitializeArgs {
            path: root.to_string_lossy().to_string(),
            project_title: "My Project".to_owned(),
            stack: 0,
        })
        .unwrap();

        assert!(root.join(".ralph").is_dir());
        assert!(root.join(".ralph").join("db").is_dir());
        assert!(root.join(".ralph").join("db").join("ralph.db").exists());
        assert!(root.join(".ralph").join("CLAUDE.RALPH.md").exists());
    }

    #[test]
    fn validate_project_path_errors_when_missing_directory() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("missing");
        let err = validate_project_path(&missing).unwrap_err();
        assert!(err.contains("[R-1000]"));
        assert!(err.contains("Directory not found"));
    }

    #[test]
    fn validate_project_path_errors_when_not_directory() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("file");
        std::fs::write(&file, "x").unwrap();
        let err = validate_project_path(&file).unwrap_err();
        assert!(err.contains("[R-1000]"));
        assert!(err.contains("Not a directory"));
    }

    #[test]
    fn validate_project_path_errors_when_missing_ralph_dir() {
        let dir = tempdir().unwrap();
        let err = validate_project_path(dir.path()).unwrap_err();
        assert!(err.contains("[R-1000]"));
        assert!(err.contains("No .ralph/ folder"));
    }

    #[test]
    fn validate_project_path_errors_when_missing_db_file() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".ralph")).unwrap();
        let err = validate_project_path(dir.path()).unwrap_err();
        assert!(err.contains("[R-1000]"));
        assert!(err.contains("No .ralph/db/ralph.db found"));
    }

    #[test]
    fn validate_project_path_ok_when_db_file_exists() {
        let dir = tempdir().unwrap();
        let db_dir = dir.path().join(".ralph").join("db");
        std::fs::create_dir_all(&db_dir).unwrap();
        std::fs::write(db_dir.join("ralph.db"), "").unwrap();
        validate_project_path(dir.path()).unwrap();
    }
}
