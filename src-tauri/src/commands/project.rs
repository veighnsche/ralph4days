use super::remote_proxy::{remote_invoke_args, remote_invoke_no_args};
use super::state::{AppState, CommandContext};
use ralph_backend::project::{ProjectInitializeArgs, ProjectValidatePathArgs};
use ralph_backend::project_contract::RecentProject;
use ralph_backend::session::ProjectLockSetArgs;
use ralph_errors::{codes, ralph_err, RalphResultExt};
use ralph_macros::ipc_type;
use serde::{Deserialize, Serialize};
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RalphProject {
    pub name: String,
    pub path: String,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectInfo {
    pub title: String,
    pub description: Option<String>,
    pub created: Option<String>,
}

#[tauri::command]
#[tracing::instrument(skip(state))]
pub async fn project_validate_path(
    state: State<'_, AppState>,
    args: ProjectValidatePathArgs,
) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "project_validate_path", args).await;
    }

    let path = PathBuf::from(&args.path);
    ralph_backend::project::validate_project_path(&path)
}

#[tauri::command]
#[tracing::instrument(skip(state))]
pub async fn project_initialize(
    state: State<'_, AppState>,
    args: ProjectInitializeArgs,
) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "project_initialize", args).await;
    }

    ralph_backend::project::project_initialize(args)
}

pub fn project_lock_validated(state: &AppState, path: String) -> Result<(), String> {
    let canonical_path = ralph_backend::session::project_lock_set(
        &state.locked_project,
        &state.db,
        ProjectLockSetArgs { path },
    )?;

    let project_name = canonical_path
        .file_name()
        .map_or_else(|| "Unknown".to_owned(), |n| n.to_string_lossy().to_string());
    let _ = crate::recent_projects::add(
        &state.xdg,
        canonical_path.to_string_lossy().to_string(),
        project_name,
    );

    Ok(())
}

#[tauri::command]
pub async fn project_lock_set(
    state: State<'_, AppState>,
    args: ProjectLockSetArgs,
) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "project_lock_set", args).await;
    }

    project_lock_validated(&state, args.path)
}

#[tauri::command]
pub async fn project_lock_get(state: State<'_, AppState>) -> Result<Option<String>, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "project_lock_get").await;
    }

    ralph_backend::session::project_lock_get(&state.locked_project)
}

#[tauri::command]
pub async fn project_recent_list(state: State<'_, AppState>) -> Result<Vec<RecentProject>, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "project_recent_list").await;
    }

    crate::recent_projects::load(&state.xdg).map(|projects| {
        projects
            .into_iter()
            .map(|p| RecentProject {
                path: p.path,
                name: p.name,
                last_opened: p.last_opened,
            })
            .collect()
    })
}

#[tauri::command]
pub async fn execution_start(state: State<'_, AppState>) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "execution_start").await;
    }

    ralph_err!(codes::LOOP_ENGINE, "Not implemented")
}

#[tauri::command]
pub async fn execution_pause(state: State<'_, AppState>) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "execution_pause").await;
    }

    ralph_err!(codes::LOOP_ENGINE, "Not implemented")
}

#[tauri::command]
pub async fn execution_resume(state: State<'_, AppState>) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "execution_resume").await;
    }

    ralph_err!(codes::LOOP_ENGINE, "Not implemented")
}

#[tauri::command]
pub async fn execution_stop(state: State<'_, AppState>) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "execution_stop").await;
    }

    ralph_err!(codes::LOOP_ENGINE, "Not implemented")
}

#[tauri::command]
pub async fn execution_state_get(state: State<'_, AppState>) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "execution_state_get").await;
    }

    ralph_err!(codes::LOOP_ENGINE, "Not implemented")
}

#[tauri::command]
pub async fn project_scan(
    state: State<'_, AppState>,
    args: ProjectScanArgs,
) -> Result<Vec<RalphProject>, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "project_scan", args).await;
    }

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
pub async fn system_home_dir_get(state: State<'_, AppState>) -> Result<String, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "system_home_dir_get").await;
    }

    let path = dirs::home_dir().ok_or_else(|| {
        ralph_errors::err_string(codes::FILESYSTEM, "Failed to get home directory")
    })?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn project_info_get(state: State<'_, AppState>) -> Result<ProjectInfo, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "project_info_get").await;
    }

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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectScanArgs {
    pub root_dir: Option<String>,
}
