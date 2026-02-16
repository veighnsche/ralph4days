use super::remote_proxy::{remote_invoke_args, remote_invoke_no_args};
use super::state::AppState;
use ralph_errors::RalphResult;
use tauri::State;

#[tauri::command]
pub async fn prompt_builder_preview(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "prompt_builder_preview", args).await
}

#[tauri::command]
pub async fn prompt_builder_config_list(
    state: State<'_, AppState>,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_no_args(&rpc, "prompt_builder_config_list").await
}

#[tauri::command]
pub async fn prompt_builder_config_get(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "prompt_builder_config_get", args).await
}

#[tauri::command]
pub async fn prompt_builder_config_save(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "prompt_builder_config_save", args).await
}

#[tauri::command]
pub async fn prompt_builder_config_delete(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "prompt_builder_config_delete", args).await
}
