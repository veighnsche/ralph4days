use super::remote_proxy::{remote_invoke_args, remote_invoke_no_args};
use super::state::AppState;
use ralph_backend::project::{ProjectInitializeArgs, ProjectValidatePathArgs};
use ralph_backend::project_contract::{ProjectInfo, ProjectScanArgs, RalphProject, RecentProject};
use ralph_backend::session::ProjectLockSetArgs;
use ralph_errors::{codes, ralph_err, RalphResult};
use tauri::State;

#[tauri::command]
#[tracing::instrument(skip(state))]
pub async fn project_validate_path(
    state: State<'_, AppState>,
    args: ProjectValidatePathArgs,
) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "project_validate_path", args).await
}

#[tauri::command]
#[tracing::instrument(skip(state))]
pub async fn project_initialize(
    state: State<'_, AppState>,
    args: ProjectInitializeArgs,
) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "project_initialize", args).await
}

#[tauri::command]
pub async fn project_lock_set(
    state: State<'_, AppState>,
    args: ProjectLockSetArgs,
) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "project_lock_set", args).await
}

#[tauri::command]
pub async fn project_lock_get(state: State<'_, AppState>) -> RalphResult<Option<String>> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_no_args(&rpc, "project_lock_get").await
}

#[tauri::command]
pub async fn project_recent_list(state: State<'_, AppState>) -> RalphResult<Vec<RecentProject>> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_no_args(&rpc, "project_recent_list").await
}

#[tauri::command]
pub async fn execution_start(state: State<'_, AppState>) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_no_args(&rpc, "execution_start").await
}

#[tauri::command]
pub async fn execution_pause(state: State<'_, AppState>) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_no_args(&rpc, "execution_pause").await
}

#[tauri::command]
pub async fn execution_resume(state: State<'_, AppState>) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_no_args(&rpc, "execution_resume").await
}

#[tauri::command]
pub async fn execution_stop(state: State<'_, AppState>) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_no_args(&rpc, "execution_stop").await
}

#[tauri::command]
pub async fn execution_state_get(state: State<'_, AppState>) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_no_args(&rpc, "execution_state_get").await
}

#[tauri::command]
pub async fn project_scan(
    state: State<'_, AppState>,
    args: ProjectScanArgs,
) -> RalphResult<Vec<RalphProject>> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "project_scan", args).await
}

#[tauri::command]
pub async fn system_home_dir_get(state: State<'_, AppState>) -> RalphResult<String> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_no_args(&rpc, "system_home_dir_get").await
}

#[tauri::command]
pub async fn project_info_get(state: State<'_, AppState>) -> RalphResult<ProjectInfo> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_no_args(&rpc, "project_info_get").await
}

#[tauri::command]
pub fn window_splash_close(app: tauri::AppHandle) -> RalphResult<()> {
    let _ = app;
    ralph_err!(
        codes::INTERNAL,
        "window_splash_close is unsupported on mobile"
    )
}

#[tauri::command]
pub fn window_open_new() -> RalphResult<()> {
    ralph_err!(codes::INTERNAL, "window_open_new is unsupported on mobile")
}
