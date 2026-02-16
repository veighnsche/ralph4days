use super::remote_proxy::{remote_invoke_args, remote_invoke_no_args};
use super::state::AppState;
use ralph_errors::RalphResult;
use tauri::State;

#[tauri::command]
pub async fn tasks_create(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "tasks_create", args).await
}

#[tauri::command]
pub async fn tasks_update(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "tasks_update", args).await
}

#[tauri::command]
pub async fn tasks_set_status(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "tasks_set_status", args).await
}

#[tauri::command]
pub async fn tasks_delete(state: State<'_, AppState>, args: serde_json::Value) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "tasks_delete", args).await
}

#[tauri::command]
pub async fn tasks_signal_add(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "tasks_signal_add", args).await
}

#[tauri::command]
pub async fn tasks_signal_update(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "tasks_signal_update", args).await
}

#[tauri::command]
pub async fn tasks_signal_delete(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "tasks_signal_delete", args).await
}

#[tauri::command]
pub async fn tasks_list(state: State<'_, AppState>) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_no_args(&rpc, "tasks_list").await
}

#[tauri::command]
pub async fn tasks_get(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "tasks_get", args).await
}

#[tauri::command]
pub async fn tasks_list_items(state: State<'_, AppState>) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_no_args(&rpc, "tasks_list_items").await
}

#[tauri::command]
pub async fn tasks_signal_summaries_get(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "tasks_signal_summaries_get", args).await
}

#[tauri::command]
pub async fn tasks_ask_answer(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "tasks_ask_answer", args).await
}

#[tauri::command]
pub async fn tasks_comment_reply_add(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "tasks_comment_reply_add", args).await
}

#[tauri::command]
pub async fn tasks_signal_comment_add(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "tasks_signal_comment_add", args).await
}

#[tauri::command]
pub async fn tasks_signal_comment_update(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "tasks_signal_comment_update", args).await
}

#[tauri::command]
pub async fn tasks_signal_comment_delete(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "tasks_signal_comment_delete", args).await
}

#[tauri::command]
pub async fn tasks_signal_comments_list(
    state: State<'_, AppState>,
    args: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "tasks_signal_comments_list", args).await
}
