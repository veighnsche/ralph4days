use super::remote_proxy::{remote_invoke_args, remote_invoke_no_args};
use super::state::{AppState, CommandContext};
use ralph_backend::project::{ProjectInitializeArgs, ProjectValidatePathArgs};
use ralph_backend::project_contract::{ProjectInfo, ProjectScanArgs, RalphProject, RecentProject};
use ralph_backend::project_scan;
use ralph_backend::session::ProjectLockSetArgs;
use ralph_errors::{codes, ralph_err, RalphResultExt};
use std::path::PathBuf;
use tauri::{Manager, State};

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
    let data_dir = state.xdg.ensure_data()?;
    if let Err(error) = project_scan::recents_add(
        data_dir,
        canonical_path.to_string_lossy().to_string(),
        project_name,
    ) {
        crate::diagnostics::emit_warning(
            "recent-projects",
            "write-failed",
            &format!("Failed to persist recent projects: {error}"),
        );
    }

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

    let data_dir = state.inner().xdg.ensure_data()?;
    project_scan::recents_load(data_dir)
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

    project_scan::project_scan(args)
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

    CommandContext::from_tauri_state(&state).db(project_scan::project_info_get)
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
