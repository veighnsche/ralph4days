use super::remote_proxy::{remote_invoke_args, remote_invoke_no_args};
use super::state::AppState;
use ralph_backend::terminal::{
    TerminalBridgeEmitSystemMessageArgs, TerminalBridgeListModelFormTreeResult,
    TerminalBridgeReplayOutputArgs, TerminalBridgeReplayOutputResult, TerminalBridgeResizeArgs,
    TerminalBridgeResolveTaskLaunchArgs, TerminalBridgeResolvedLaunchConfig,
    TerminalBridgeSendInputArgs, TerminalBridgeSetStreamModeArgs,
    TerminalBridgeStartHumanSessionArgs, TerminalBridgeStartHumanSessionResult,
    TerminalBridgeStartSessionArgs, TerminalBridgeStartTaskSessionArgs,
    TerminalBridgeTerminateArgs,
};
use ralph_errors::RalphResult;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn terminal_start_session(
    _app: AppHandle,
    state: State<'_, AppState>,
    args: TerminalBridgeStartSessionArgs,
) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "terminal_start_session", args).await
}

#[tauri::command]
pub async fn terminal_start_task_session(
    _app: AppHandle,
    state: State<'_, AppState>,
    args: TerminalBridgeStartTaskSessionArgs,
) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "terminal_start_task_session", args).await
}

#[tauri::command]
pub async fn terminal_resolve_task_launch_config(
    state: State<'_, AppState>,
    args: TerminalBridgeResolveTaskLaunchArgs,
) -> RalphResult<TerminalBridgeResolvedLaunchConfig> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "terminal_resolve_task_launch_config", args).await
}

#[tauri::command]
pub async fn terminal_send_input(
    state: State<'_, AppState>,
    args: TerminalBridgeSendInputArgs,
) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "terminal_send_input", args).await
}

#[tauri::command]
pub async fn terminal_resize(
    state: State<'_, AppState>,
    args: TerminalBridgeResizeArgs,
) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "terminal_resize", args).await
}

#[tauri::command]
pub async fn terminal_terminate(
    state: State<'_, AppState>,
    args: TerminalBridgeTerminateArgs,
) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "terminal_terminate", args).await
}

#[tauri::command]
pub async fn terminal_set_stream_mode(
    state: State<'_, AppState>,
    args: TerminalBridgeSetStreamModeArgs,
) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "terminal_set_stream_mode", args).await
}

#[tauri::command]
pub async fn terminal_replay_output(
    state: State<'_, AppState>,
    args: TerminalBridgeReplayOutputArgs,
) -> RalphResult<TerminalBridgeReplayOutputResult> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "terminal_replay_output", args).await
}

#[tauri::command]
pub async fn terminal_emit_system_message(
    _app: AppHandle,
    state: State<'_, AppState>,
    args: TerminalBridgeEmitSystemMessageArgs,
) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "terminal_emit_system_message", args).await
}

#[tauri::command]
pub async fn terminal_start_human_session(
    _app: AppHandle,
    state: State<'_, AppState>,
    args: TerminalBridgeStartHumanSessionArgs,
) -> RalphResult<TerminalBridgeStartHumanSessionResult> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "terminal_start_human_session", args).await
}

#[tauri::command]
pub async fn terminal_list_model_form_tree(
    state: State<'_, AppState>,
) -> RalphResult<TerminalBridgeListModelFormTreeResult> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_no_args(&rpc, "terminal_list_model_form_tree").await
}
