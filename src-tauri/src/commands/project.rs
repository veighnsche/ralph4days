use super::state::{AppState, CommandContext};
use ralph_errors::{codes, ralph_err, RalphResultExt, ToStringErr};
use ralph_macros::ipc_type;
use serde::{Deserialize, Serialize};
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
pub fn project_validate_path(args: ProjectValidatePathArgs) -> Result<(), String> {
    let path = PathBuf::from(&args.path);
    ralph_backend::project::validate_project_path(&path)
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectValidatePathArgs {
    pub path: String,
}

fn seed_disciplines_for_stack(
    db: &SqliteDb,
    stack: u8,
    ralph_dir: Option<&std::path::Path>,
) -> Result<(), String> {
    let defs = predefined_disciplines::get_disciplines_for_stack(stack);
    if defs.is_empty() && stack != 0 {
        return ralph_err!(
            codes::DISCIPLINE_OPS,
            "No disciplines defined for stack {stack}"
        );
    }

    if let Some(ralph_dir) = ralph_dir {
        let images_dir = ralph_dir.join("images").join("disciplines");
        let _ = std::fs::create_dir_all(&images_dir);
    }

    for d in &defs {
        let skills_json = serde_json::to_string(&d.skills).map_err(|error| {
            format!(
                "Failed to serialize skills for discipline '{}': {error}",
                d.name
            )
        })?;

        let image_path = ralph_dir.and_then(|ralph_dir| {
            predefined_disciplines::get_discipline_image(stack, &d.name).and_then(|bytes| {
                let rel = format!("images/disciplines/{}.png", d.name);
                let abs = ralph_dir.join(&rel);
                std::fs::write(&abs, bytes).is_ok().then_some(rel)
            })
        });

        let crops_json = d.crops.as_ref().and_then(|crops| {
            serde_json::to_string(crops).ok().or_else(|| {
                tracing::warn!(
                    discipline = %d.name,
                    "Failed to serialize crops; storing no crops"
                );
                None
            })
        });
        let image_prompt_json = d.image_prompt.as_ref().and_then(|prompt| {
            serde_json::to_string(prompt).ok().or_else(|| {
                tracing::warn!(
                    discipline = %d.name,
                    "Failed to serialize image_prompt; storing no prompt"
                );
                None
            })
        });

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

#[tauri::command]
#[tracing::instrument]
pub fn project_initialize(args: ProjectInitializeArgs) -> Result<(), String> {
    let stack = args.stack;
    tracing::info!("Initializing Ralph project with stack {}", stack);
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

    std::fs::create_dir(&ralph_dir)
        .ralph_err(codes::PROJECT_INIT, "Failed to create .ralph/ directory")?;

    let db_dir = ralph_dir.join("db");
    std::fs::create_dir(&db_dir)
        .ralph_err(codes::PROJECT_INIT, "Failed to create .ralph/db/ directory")?;

    let db_path = db_dir.join("ralph.db");
    let db = SqliteDb::open(&db_path, None)?;
    seed_disciplines_for_stack(&db, stack, Some(&ralph_dir))?;
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

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInitializeArgs {
    pub path: String,
    pub project_title: String,
    pub stack: u8,
}

pub fn project_lock_validated(state: &AppState, path: String) -> Result<(), String> {
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
    let db = SqliteDb::open(&db_path, None)?;

    let mut db_guard = state.db.lock().err_str(codes::INTERNAL)?;
    *db_guard = Some(db);

    let project_name = canonical_path
        .file_name()
        .map_or_else(|| "Unknown".to_owned(), |n| n.to_string_lossy().to_string());
    let _ = crate::recent_projects::add(
        &state.xdg,
        canonical_path.to_string_lossy().to_string(),
        project_name,
    );

    *locked = Some(canonical_path);
    Ok(())
}

#[tauri::command]
pub fn project_lock_set(
    state: State<'_, AppState>,
    args: ProjectLockSetArgs,
) -> Result<(), String> {
    project_validate_path(ProjectValidatePathArgs {
        path: args.path.clone(),
    })?;
    project_lock_validated(&state, args.path)
}

#[tauri::command]
pub fn project_lock_get(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let locked = CommandContext::from_tauri_state(&state).maybe_locked_project_path()?;
    Ok(locked.as_ref().map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub fn project_recent_list(
    state: State<'_, AppState>,
) -> Result<Vec<crate::recent_projects::RecentProject>, String> {
    crate::recent_projects::load(&state.xdg)
}

#[tauri::command]
pub fn execution_start() -> Result<(), String> {
    ralph_err!(codes::LOOP_ENGINE, "Not implemented")
}

#[tauri::command]
pub fn execution_pause() -> Result<(), String> {
    ralph_err!(codes::LOOP_ENGINE, "Not implemented")
}

#[tauri::command]
pub fn execution_resume() -> Result<(), String> {
    ralph_err!(codes::LOOP_ENGINE, "Not implemented")
}

#[tauri::command]
pub fn execution_stop() -> Result<(), String> {
    ralph_err!(codes::LOOP_ENGINE, "Not implemented")
}

#[tauri::command]
pub fn execution_state_get() -> Result<(), String> {
    ralph_err!(codes::LOOP_ENGINE, "Not implemented")
}

#[tauri::command]
pub fn project_scan(args: ProjectScanArgs) -> Result<Vec<RalphProject>, String> {
    let scan_path = if let Some(dir) = args.root_dir {
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
pub fn system_home_dir_get() -> Result<String, String> {
    let path = dirs::home_dir().ok_or_else(|| {
        ralph_errors::err_string(codes::FILESYSTEM, "Failed to get home directory")
    })?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn project_info_get(state: State<'_, AppState>) -> Result<ProjectInfo, String> {
    let info = CommandContext::from_tauri_state(&state).db(|db| Ok(db.get_project_info()))?;
    Ok(ProjectInfo {
        title: info.title.clone(),
        description: info.description.clone(),
        created: info.created,
    })
}

#[tauri::command]
pub fn window_splash_close(app: tauri::AppHandle) {
    if let Some(splash) = app.get_webview_window("splash") {
        let _ = splash.close();
    }
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.set_focus();
    }
}

#[tauri::command]
pub fn window_open_new() -> Result<(), String> {
    let exe = std::env::current_exe()
        .ralph_err(codes::INTERNAL, "Failed to get current executable path")?;
    std::process::Command::new(exe)
        .spawn()
        .ralph_err(codes::INTERNAL, "Failed to spawn new window")?;
    Ok(())
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectLockSetArgs {
    pub path: String,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectScanArgs {
    pub root_dir: Option<String>,
}
