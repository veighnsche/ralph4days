use super::remote_proxy::{remote_invoke_args, remote_invoke_no_args};
use super::state::AppState;
use ralph_errors::RalphResult;
use tauri::State;

#[tauri::command]
pub async fn disciplines_list(state: State<'_, AppState>) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_no_args(&rpc, "disciplines_list").await
}

#[tauri::command]
pub async fn subsystems_list(state: State<'_, AppState>) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_no_args(&rpc, "subsystems_list").await
}

#[tauri::command]
pub async fn subsystems_create(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "subsystems_create", args).await
}

#[tauri::command]
pub async fn subsystems_update(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "subsystems_update", args).await
}

#[tauri::command]
pub async fn subsystems_comment_add(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "subsystems_comment_add", args).await
}

#[tauri::command]
pub async fn subsystems_comment_update(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "subsystems_comment_update", args).await
}

#[tauri::command]
pub async fn subsystems_comment_delete(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "subsystems_comment_delete", args).await
}

#[tauri::command]
pub async fn disciplines_create(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "disciplines_create", args).await
}

#[tauri::command]
pub async fn disciplines_update(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "disciplines_update", args).await
}

#[tauri::command]
pub async fn subsystems_delete(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "subsystems_delete", args).await
}

#[tauri::command]
pub async fn disciplines_delete(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "disciplines_delete", args).await
}

#[tauri::command]
pub async fn stacks_metadata_list(state: State<'_, AppState>) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_no_args(&rpc, "stacks_metadata_list").await
}

#[tauri::command]
pub async fn disciplines_image_data_get(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "disciplines_image_data_get", args).await
}

#[tauri::command]
pub async fn disciplines_cropped_image_get(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "disciplines_cropped_image_get", args).await
}
