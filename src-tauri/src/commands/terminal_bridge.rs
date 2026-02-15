use super::remote_proxy::{remote_invoke_args, remote_invoke_no_args};
use super::state::{AppState, CommandContext};
use crate::event_sink::TauriEventSink;
use crate::terminal::{
    TerminalBridgeEmitSystemMessageArgs, TerminalBridgeListModelFormTreeResult,
    TerminalBridgeReplayOutputArgs, TerminalBridgeReplayOutputResult, TerminalBridgeResizeArgs,
    TerminalBridgeResolveTaskLaunchArgs, TerminalBridgeResolvedLaunchConfig,
    TerminalBridgeSendInputArgs, TerminalBridgeSetStreamModeArgs,
    TerminalBridgeStartHumanSessionArgs, TerminalBridgeStartHumanSessionResult,
    TerminalBridgeStartSessionArgs, TerminalBridgeStartTaskSessionArgs,
    TerminalBridgeTerminateArgs,
};
use ralph_contracts::transport::EventSink;
use ralph_errors::{codes, ToStringErr};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, State};

fn locked_project_path(state: &State<'_, AppState>) -> Result<PathBuf, String> {
    CommandContext::from_tauri_state(state).locked_project_path()
}

fn api_server_port(state: &AppState) -> Result<Option<u16>, String> {
    Ok(*state.api_server_port.lock().err_str(codes::INTERNAL)?)
}

#[tauri::command]
pub async fn terminal_start_session(
    app: AppHandle,
    state: State<'_, AppState>,
    args: TerminalBridgeStartSessionArgs,
) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "terminal_start_session", args).await;
    }

    let project_path = locked_project_path(&state)?;
    let sink: Arc<dyn EventSink> = Arc::new(TauriEventSink::new(app));
    let ctx = ralph_backend::terminal_bridge::TerminalBridgeCtx {
        pty_manager: &state.inner().pty_manager,
        sink,
        locked_project_path: project_path.as_path(),
        db: &state.inner().db,
        codebase_snapshot: &state.inner().codebase_snapshot,
        mcp_dir: state.inner().mcp_dir.as_path(),
        api_server_port: api_server_port(state.inner())?,
    };
    ralph_backend::terminal_bridge::terminal_start_session(&ctx, args)
}

#[tauri::command]
pub async fn terminal_start_task_session(
    app: AppHandle,
    state: State<'_, AppState>,
    args: TerminalBridgeStartTaskSessionArgs,
) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "terminal_start_task_session", args).await;
    }

    let project_path = locked_project_path(&state)?;
    let sink: Arc<dyn EventSink> = Arc::new(TauriEventSink::new(app));
    let ctx = ralph_backend::terminal_bridge::TerminalBridgeCtx {
        pty_manager: &state.inner().pty_manager,
        sink,
        locked_project_path: project_path.as_path(),
        db: &state.inner().db,
        codebase_snapshot: &state.inner().codebase_snapshot,
        mcp_dir: state.inner().mcp_dir.as_path(),
        api_server_port: api_server_port(state.inner())?,
    };
    ralph_backend::terminal_bridge::terminal_start_task_session(&ctx, args)
}

#[tauri::command]
pub async fn terminal_resolve_task_launch_config(
    state: State<'_, AppState>,
    args: TerminalBridgeResolveTaskLaunchArgs,
) -> Result<TerminalBridgeResolvedLaunchConfig, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "terminal_resolve_task_launch_config", args).await;
    }

    CommandContext::from_tauri_state(&state).db(|db| {
        ralph_backend::terminal::resolve_task_launch_config(db, args.task_id, args.defaults)
    })
}

#[tauri::command]
pub async fn terminal_send_input(
    state: State<'_, AppState>,
    args: TerminalBridgeSendInputArgs,
) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "terminal_send_input", args).await;
    }

    ralph_backend::terminal_bridge::terminal_send_input(&state.inner().pty_manager, args)
}

#[tauri::command]
pub async fn terminal_resize(
    state: State<'_, AppState>,
    args: TerminalBridgeResizeArgs,
) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "terminal_resize", args).await;
    }

    ralph_backend::terminal_bridge::terminal_resize(&state.inner().pty_manager, args)
}

#[tauri::command]
pub async fn terminal_terminate(
    state: State<'_, AppState>,
    args: TerminalBridgeTerminateArgs,
) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "terminal_terminate", args).await;
    }

    ralph_backend::terminal_bridge::terminal_terminate(&state.inner().pty_manager, args)
}

#[tauri::command]
pub async fn terminal_set_stream_mode(
    state: State<'_, AppState>,
    args: TerminalBridgeSetStreamModeArgs,
) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "terminal_set_stream_mode", args).await;
    }

    ralph_backend::terminal_bridge::terminal_set_stream_mode(&state.inner().pty_manager, args)
}

#[tauri::command]
pub async fn terminal_replay_output(
    state: State<'_, AppState>,
    args: TerminalBridgeReplayOutputArgs,
) -> Result<TerminalBridgeReplayOutputResult, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "terminal_replay_output", args).await;
    }

    ralph_backend::terminal_bridge::terminal_replay_output(&state.inner().pty_manager, args)
}

#[tauri::command]
pub fn terminal_emit_system_message(
    app: AppHandle,
    args: TerminalBridgeEmitSystemMessageArgs,
) -> Result<(), String> {
    let sink = TauriEventSink::new(app);
    ralph_backend::terminal_bridge::emit_system_message(&sink, args.session_id, args.text)
}

#[tauri::command]
pub async fn terminal_start_human_session(
    app: AppHandle,
    state: State<'_, AppState>,
    args: TerminalBridgeStartHumanSessionArgs,
) -> Result<TerminalBridgeStartHumanSessionResult, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "terminal_start_human_session", args).await;
    }

    let project_path = locked_project_path(&state)?;
    let sink: Arc<dyn EventSink> = Arc::new(TauriEventSink::new(app));
    let ctx = ralph_backend::terminal_bridge::TerminalBridgeCtx {
        pty_manager: &state.inner().pty_manager,
        sink,
        locked_project_path: project_path.as_path(),
        db: &state.inner().db,
        codebase_snapshot: &state.inner().codebase_snapshot,
        mcp_dir: state.inner().mcp_dir.as_path(),
        api_server_port: api_server_port(state.inner())?,
    };
    ralph_backend::terminal_bridge::terminal_start_human_session(&ctx, args)
}

#[tauri::command]
pub async fn terminal_list_model_form_tree(
    state: State<'_, AppState>,
) -> Result<TerminalBridgeListModelFormTreeResult, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "terminal_list_model_form_tree").await;
    }

    Ok(ralph_backend::terminal_bridge::terminal_list_model_form_tree())
}
