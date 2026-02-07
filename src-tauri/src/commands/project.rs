use super::state::{AppState, ToStringErr};
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

#[derive(Debug, Clone, serde::Serialize)]
pub struct RalphProject {
    pub name: String,
    pub path: String,
}

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
        return Err(format!("Directory not found: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("Not a directory: {}", path.display()));
    }

    let ralph_dir = path.join(".ralph");
    if !ralph_dir.exists() {
        return Err(format!(
            "No .ralph/ folder. Initialize with:\n  ralph --init \"{}\"",
            path.display()
        ));
    }
    if !ralph_dir.is_dir() {
        return Err(format!(
            "{} exists but is not a directory",
            ralph_dir.display()
        ));
    }

    let db_file = ralph_dir.join("db").join("ralph.db");
    if !db_file.exists() {
        return Err(format!(
            "No .ralph/db/ralph.db found. Initialize with:\n  ralph --init \"{}\"",
            path.display()
        ));
    }

    Ok(())
}

#[tauri::command]
pub fn initialize_ralph_project(path: String, project_title: String) -> Result<(), String> {
    let path = PathBuf::from(&path);

    if !path.exists() {
        return Err(format!("Directory not found: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("Not a directory: {}", path.display()));
    }

    let ralph_dir = path.join(".ralph");
    if ralph_dir.exists() {
        return Err(format!(".ralph/ already exists at {}", path.display()));
    }

    std::fs::create_dir(&ralph_dir)
        .map_err(|e| format!("Failed to create .ralph/ directory: {e}"))?;

    let db_dir = ralph_dir.join("db");
    std::fs::create_dir(&db_dir)
        .map_err(|e| format!("Failed to create .ralph/db/ directory: {e}"))?;

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

    std::fs::write(&claude_path, claude_template)
        .map_err(|e| format!("Failed to create CLAUDE.RALPH.md: {e}"))?;

    Ok(())
}

#[tauri::command]
pub fn set_locked_project(state: State<'_, AppState>, path: String) -> Result<(), String> {
    validate_project_path(path.clone())?;

    let canonical_path =
        std::fs::canonicalize(&path).map_err(|e| format!("Failed to resolve path: {e}"))?;

    let mut locked = state.locked_project.lock().err_str()?;
    if locked.is_some() {
        return Err("Project already locked for this session".to_owned());
    }

    let db_path = canonical_path.join(".ralph").join("db").join("ralph.db");
    let db = SqliteDb::open(&db_path)?;

    let snapshot = prompt_builder::snapshot::analyze(&canonical_path);
    let mut snap_guard = state.codebase_snapshot.lock().err_str()?;
    *snap_guard = Some(snapshot);

    let mut db_guard = state.db.lock().err_str()?;
    *db_guard = Some(db);

    *locked = Some(canonical_path);
    Ok(())
}

#[tauri::command]
pub fn get_locked_project(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let locked = state.locked_project.lock().err_str()?;
    Ok(locked.as_ref().map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub fn start_loop() -> Result<(), String> {
    Err("Not implemented".to_owned())
}

#[tauri::command]
pub fn pause_loop() -> Result<(), String> {
    Err("Not implemented".to_owned())
}

#[tauri::command]
pub fn resume_loop() -> Result<(), String> {
    Err("Not implemented".to_owned())
}

#[tauri::command]
pub fn stop_loop() -> Result<(), String> {
    Err("Not implemented".to_owned())
}

#[tauri::command]
pub fn get_loop_state() -> Result<(), String> {
    Err("Not implemented".to_owned())
}

#[tauri::command]
pub fn scan_for_ralph_projects(root_dir: Option<String>) -> Result<Vec<RalphProject>, String> {
    let scan_path = if let Some(dir) = root_dir {
        PathBuf::from(dir)
    } else {
        dirs::home_dir().ok_or("Failed to get home directory")?
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
    let path = dirs::home_dir().ok_or("Failed to get home directory")?;
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
