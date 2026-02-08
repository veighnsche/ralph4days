use super::state::AppState;
use ralph_errors::{codes, ralph_err, RalphResultExt, ToStringErr};
use ralph_macros::ipc_type;
use sqlite_db::SqliteDb;
use std::path::PathBuf;
use tauri::{Manager, State};

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
#[tracing::instrument]
pub fn validate_project_path(path: String) -> Result<(), String> {
    tracing::debug!("Validating project path");
    let path = PathBuf::from(&path);

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

fn seed_disciplines_for_stack(db: &SqliteDb, stack: u8) -> Result<(), String> {
    let defs = predefined_disciplines::get_disciplines_for_stack(stack);
    if defs.is_empty() && stack != 0 {
        return ralph_err!(
            codes::DISCIPLINE_OPS,
            "No disciplines defined for stack {stack}"
        );
    }
    for d in defs {
        let skills_json = serde_json::to_string(&d.skills).unwrap_or_else(|_| "[]".to_owned());
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
        })?;
    }
    Ok(())
}

#[tauri::command]
#[tracing::instrument]
pub fn initialize_ralph_project(
    path: String,
    project_title: String,
    stack: u8,
) -> Result<(), String> {
    tracing::info!("Initializing Ralph project with stack {}", stack);
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

    std::fs::create_dir(&ralph_dir)
        .ralph_err(codes::PROJECT_INIT, "Failed to create .ralph/ directory")?;

    let db_dir = ralph_dir.join("db");
    std::fs::create_dir(&db_dir)
        .ralph_err(codes::PROJECT_INIT, "Failed to create .ralph/db/ directory")?;

    let db_path = db_dir.join("ralph.db");
    let db = SqliteDb::open(&db_path)?;
    seed_disciplines_for_stack(&db, stack)?;
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
        .ralph_err(codes::FILESYSTEM, "Failed to create CLAUDE.RALPH.md")?;

    Ok(())
}

pub fn lock_project_validated(state: &AppState, path: String) -> Result<(), String> {
    let canonical_path =
        std::fs::canonicalize(&path).ralph_err(codes::PROJECT_PATH, "Failed to resolve path")?;

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
            ralph_errors::err_string(codes::FILESYSTEM, "Failed to get home directory")
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
        ralph_errors::err_string(codes::FILESYSTEM, "Failed to get home directory")
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

#[tauri::command]
pub fn close_splash(app: tauri::AppHandle) {
    if let Some(splash) = app.get_webview_window("splash") {
        let _ = splash.close();
    }
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.show();
        let _ = main.set_focus();
    }
}
