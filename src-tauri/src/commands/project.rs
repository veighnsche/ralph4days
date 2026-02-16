use super::remote_proxy::{remote_invoke_args, remote_invoke_no_args};
use super::state::{AppState, CommandContext};
use ralph_backend::project_scan;
use ralph_contracts::project::{
    ProjectInfo, ProjectInitializeArgs, ProjectScanArgs, ProjectValidatePathArgs, RalphProject,
    RecentProject,
};
use ralph_contracts::session::ProjectLockSetArgs;
use ralph_errors::{codes, ralph_err, RalphResult, RalphResultExt};
use std::path::PathBuf;
use tauri::{Manager, State};

#[tauri::command]
#[tracing::instrument(skip(state))]
pub async fn project_validate_path(
    state: State<'_, AppState>,
    args: ProjectValidatePathArgs,
) -> RalphResult<()> {
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
) -> RalphResult<()> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "project_initialize", args).await;
    }

    ralph_backend::project::project_initialize(args)
}

pub fn project_lock_validated(state: &AppState, path: String) -> RalphResult<()> {
    let data_dir = state.xdg.ensure_data()?;
    let _canonical_path = ralph_backend::session::project_lock_set_and_record_recent(
        &state.locked_project,
        &state.db,
        data_dir,
        ProjectLockSetArgs { path },
    )?;

    Ok(())
}

#[tauri::command]
pub async fn project_lock_set(
    state: State<'_, AppState>,
    args: ProjectLockSetArgs,
) -> RalphResult<()> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "project_lock_set", args).await;
    }

    project_lock_validated(&state, args.path)
}

#[tauri::command]
pub async fn project_lock_get(state: State<'_, AppState>) -> RalphResult<Option<String>> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "project_lock_get").await;
    }

    ralph_backend::session::project_lock_get(&state.locked_project)
}

#[tauri::command]
pub async fn project_recent_list(state: State<'_, AppState>) -> RalphResult<Vec<RecentProject>> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "project_recent_list").await;
    }

    let data_dir = state.inner().xdg.ensure_data()?;
    project_scan::recents_load(data_dir)
}

#[tauri::command]
pub async fn execution_start(state: State<'_, AppState>) -> RalphResult<()> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "execution_start").await;
    }

    ralph_err!(codes::LOOP_ENGINE, "Not implemented")
}

#[tauri::command]
pub async fn execution_pause(state: State<'_, AppState>) -> RalphResult<()> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "execution_pause").await;
    }

    ralph_err!(codes::LOOP_ENGINE, "Not implemented")
}

#[tauri::command]
pub async fn execution_resume(state: State<'_, AppState>) -> RalphResult<()> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "execution_resume").await;
    }

    ralph_err!(codes::LOOP_ENGINE, "Not implemented")
}

#[tauri::command]
pub async fn execution_stop(state: State<'_, AppState>) -> RalphResult<()> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "execution_stop").await;
    }

    ralph_err!(codes::LOOP_ENGINE, "Not implemented")
}

#[tauri::command]
pub async fn execution_state_get(state: State<'_, AppState>) -> RalphResult<()> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "execution_state_get").await;
    }

    ralph_err!(codes::LOOP_ENGINE, "Not implemented")
}

#[tauri::command]
pub async fn project_scan(
    state: State<'_, AppState>,
    args: ProjectScanArgs,
) -> RalphResult<Vec<RalphProject>> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "project_scan", args).await;
    }

    project_scan::project_scan(args)
}

#[tauri::command]
pub async fn system_home_dir_get(state: State<'_, AppState>) -> RalphResult<String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "system_home_dir_get").await;
    }

    let path = dirs::home_dir().ok_or_else(|| {
        ralph_errors::err_string(codes::FILESYSTEM, "Failed to get home directory")
    })?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn project_info_get(state: State<'_, AppState>) -> RalphResult<ProjectInfo> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "project_info_get").await;
    }

    CommandContext::from_tauri_state(&state).db(project_scan::project_info_get)
}

#[cfg(not(mobile))]
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
#[cfg(mobile)]
pub fn window_splash_close(app: tauri::AppHandle) -> RalphResult<()> {
    let _ = app;
    ralph_err!(
        codes::INTERNAL,
        "window_splash_close is unsupported on mobile"
    )
}

#[cfg(not(mobile))]
#[tauri::command]
pub fn window_open_new() -> RalphResult<()> {
    let exe = std::env::current_exe()
        .ralph_err(codes::INTERNAL, "Failed to get current executable path")?;
    std::process::Command::new(exe)
        .spawn()
        .ralph_err(codes::INTERNAL, "Failed to spawn new window")?;
    Ok(())
}

#[cfg(mobile)]
#[tauri::command]
pub fn window_open_new() -> RalphResult<()> {
    ralph_err!(codes::INTERNAL, "window_open_new is unsupported on mobile")
}
