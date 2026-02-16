use super::remote_proxy::{remote_invoke_args, remote_invoke_no_args};
use super::state::AppState;
use ralph_errors::RalphResult;
use tauri::State;

#[tauri::command]
pub async fn agent_sessions_create_human(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "agent_sessions_create_human", args).await
}

#[tauri::command]
pub async fn agent_sessions_update_human(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "agent_sessions_update_human", args).await
}

#[tauri::command]
pub async fn agent_sessions_delete_human(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "agent_sessions_delete_human", args).await
}

#[tauri::command]
pub async fn agent_sessions_get(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "agent_sessions_get", args).await
}

#[tauri::command]
pub async fn agent_sessions_list_human(
    state: State<'_, AppState>,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_no_args(&rpc, "agent_sessions_list_human").await
}
