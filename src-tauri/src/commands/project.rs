use super::state::{AppState, ToStringErr};
use crate::errors::{codes, ralph_err};
use ralph_macros::ipc_type;
use sqlite_db::SqliteDb;
use std::path::PathBuf;
use tauri::State;

const MAX_SCAN_DEPTH: usize = 5;
const MAX_PROJECTS: usize = 100;
const EXCLUDED_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "target",
    "build",
    "dist",
    ".next",
    ".venv",
    "venv",
    "__pycache__",
    ".cache",
    "tmp",
    "temp",
    ".tmp",
    "vendor",
    ".idea",
    ".vscode",
    "Library",
    "Applications",
];

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize)]
pub struct RalphProject {
    pub name: String,
    pub path: String,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    pub title: String,
    pub description: Option<String>,
    pub created: Option<String>,
}

#[tauri::command]
pub fn validate_project_path(path: String) -> Result<(), String> {
    let path = PathBuf::from(&path);

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
        return ralph_err!(
            codes::PROJECT_PATH,
            "No .ralph/db/ralph.db found. Initialize with:\n  ralph --init \"{}\"",
            path.display()
        );
    }

    Ok(())
}

#[tauri::command]
pub fn initialize_ralph_project(path: String, project_title: String) -> Result<(), String> {
    let path = PathBuf::from(&path);

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
        crate::errors::RalphError {
            code: codes::PROJECT_INIT,
            message: format!("Failed to create .ralph/ directory: {e}"),
        }
        .to_string()
    })?;

    let db_dir = ralph_dir.join("db");
    std::fs::create_dir(&db_dir).map_err(|e| {
        crate::errors::RalphError {
            code: codes::PROJECT_INIT,
            message: format!("Failed to create .ralph/db/ directory: {e}"),
        }
        .to_string()
    })?;

    let db_path = db_dir.join("ralph.db");
    let db = SqliteDb::open(&db_path)?;
    db.seed_defaults()?;
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
        crate::errors::RalphError {
            code: codes::FILESYSTEM,
            message: format!("Failed to create CLAUDE.RALPH.md: {e}"),
        }
        .to_string()
    })?;

    Ok(())
}

pub fn lock_project_validated(state: &AppState, path: String) -> Result<(), String> {
    let canonical_path = std::fs::canonicalize(&path).map_err(|e| {
        crate::errors::RalphError {
            code: codes::PROJECT_PATH,
            message: format!("Failed to resolve path: {e}"),
        }
        .to_string()
    })?;

    let mut locked = state.locked_project.lock().err_str(codes::INTERNAL)?;
    if locked.is_some() {
        return ralph_err!(
            codes::PROJECT_LOCK,
            "Project already locked for this session"
        );
    }

    let db_path = canonical_path.join(".ralph").join("db").join("ralph.db");
    let db = SqliteDb::open(&db_path)?;

    let mut db_guard = state.db.lock().err_str(codes::INTERNAL)?;
    *db_guard = Some(db);

    *locked = Some(canonical_path);
    Ok(())
}

#[tauri::command]
pub fn set_locked_project(state: State<'_, AppState>, path: String) -> Result<(), String> {
    validate_project_path(path.clone())?;
    lock_project_validated(&state, path)
}

#[tauri::command]
pub fn get_locked_project(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let locked = state.locked_project.lock().err_str(codes::INTERNAL)?;
    Ok(locked.as_ref().map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub fn start_loop() -> Result<(), String> {
    ralph_err!(codes::LOOP_ENGINE, "Not implemented")
}

#[tauri::command]
pub fn pause_loop() -> Result<(), String> {
    ralph_err!(codes::LOOP_ENGINE, "Not implemented")
}

#[tauri::command]
pub fn resume_loop() -> Result<(), String> {
    ralph_err!(codes::LOOP_ENGINE, "Not implemented")
}

#[tauri::command]
pub fn stop_loop() -> Result<(), String> {
    ralph_err!(codes::LOOP_ENGINE, "Not implemented")
}

#[tauri::command]
pub fn get_loop_state() -> Result<(), String> {
    ralph_err!(codes::LOOP_ENGINE, "Not implemented")
}

// TODO: Rename these commands to reflect sequential task execution rather than "loop"
// Commands: start_execution, pause_execution, resume_execution, stop_execution, get_execution_state

#[tauri::command]
pub fn scan_for_ralph_projects(root_dir: Option<String>) -> Result<Vec<RalphProject>, String> {
    let scan_path = if let Some(dir) = root_dir {
        PathBuf::from(dir)
    } else {
        dirs::home_dir().ok_or_else(|| {
            crate::errors::RalphError {
                code: codes::FILESYSTEM,
                message: "Failed to get home directory".to_owned(),
            }
            .to_string()
        })?
    };

    let mut projects = Vec::new();

    fn scan_recursive(
        path: &PathBuf,
        projects: &mut Vec<RalphProject>,
        depth: usize,
        max_depth: usize,
        max_projects: usize,
    ) {
        if depth > max_depth || projects.len() >= max_projects {
            return;
        }

        if !path.is_dir() {
            return;
        }

        let ralph_dir = path.join(".ralph");
        if ralph_dir.exists() && ralph_dir.is_dir() {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_owned();

            projects.push(RalphProject {
                name,
                path: path.to_string_lossy().to_string(),
            });

            if projects.len() >= max_projects {
                return;
            }
        }

        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        let entry_path = entry.path();

                        if let Some(dir_name) = entry_path.file_name().and_then(|n| n.to_str()) {
                            if EXCLUDED_DIRS.contains(&dir_name) {
                                continue;
                            }
                        }

                        scan_recursive(&entry_path, projects, depth + 1, max_depth, max_projects);

                        if projects.len() >= max_projects {
                            return;
                        }
                    }
                }
            }
        }
    }

    scan_recursive(&scan_path, &mut projects, 0, MAX_SCAN_DEPTH, MAX_PROJECTS);

    Ok(projects)
}

#[tauri::command]
pub fn get_current_dir() -> Result<String, String> {
    let path = dirs::home_dir().ok_or_else(|| {
        crate::errors::RalphError {
            code: codes::FILESYSTEM,
            message: "Failed to get home directory".to_owned(),
        }
        .to_string()
    })?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_project_info(state: State<'_, AppState>) -> Result<ProjectInfo, String> {
    let db = super::state::get_db(&state)?;
    let info = db.get_project_info();
    Ok(ProjectInfo {
        title: info.title.clone(),
        description: info.description.clone(),
        created: info.created,
    })
}
