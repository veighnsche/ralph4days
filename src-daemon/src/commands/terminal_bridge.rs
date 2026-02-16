use service_terminal::terminal_bridge;
use core_contracts::terminal_bridge::{
    TerminalBridgeEmitSystemMessageArgs, TerminalBridgeListModelFormTreeResult,
    TerminalBridgeReplayOutputArgs, TerminalBridgeReplayOutputResult, TerminalBridgeResizeArgs,
    TerminalBridgeResolveTaskLaunchArgs, TerminalBridgeResolvedLaunchConfig,
    TerminalBridgeSendInputArgs, TerminalBridgeSetStreamModeArgs,
    TerminalBridgeStartHumanSessionArgs, TerminalBridgeStartHumanSessionResult,
    TerminalBridgeStartSessionArgs, TerminalBridgeStartTaskSessionArgs,
    TerminalBridgeTerminateArgs,
};
use core_errors::{codes, RalphResult, RalphResultExt};
use std::sync::Arc;

use crate::rpc_codec::{decode_args, encode_result, require_null_payload};
use crate::state::AppState;

fn api_server_port(state: &AppState) -> RalphResult<Option<u16>> {
    let port = *state
        .api_server_port
        .lock()
        .ralph_err(codes::INTERNAL, "API server port mutex poisoned")?;
    Ok(port)
}

pub fn terminal_start_session(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: TerminalBridgeStartSessionArgs = decode_args("terminal_start_session", payload)?;
    let project_path = service_project::session::locked_project_path(&state.locked_project)?;

    let ctx = terminal_bridge::TerminalBridgeCtx {
        pty_manager: &state.pty_manager,
        sink: Arc::clone(&state.event_sink),
        locked_project_path: project_path.as_path(),
        db: &state.db,
        codebase_snapshot: &state.codebase_snapshot,
        mcp_dir: state.mcp_dir.as_path(),
        api_server_port: api_server_port(state)?,
    };
    terminal_bridge::terminal_start_session(&ctx, args)?;
    Ok(serde_json::Value::Null)
}

pub fn terminal_start_task_session(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: TerminalBridgeStartTaskSessionArgs =
        decode_args("terminal_start_task_session", payload)?;
    let project_path = service_project::session::locked_project_path(&state.locked_project)?;

    let ctx = terminal_bridge::TerminalBridgeCtx {
        pty_manager: &state.pty_manager,
        sink: Arc::clone(&state.event_sink),
        locked_project_path: project_path.as_path(),
        db: &state.db,
        codebase_snapshot: &state.codebase_snapshot,
        mcp_dir: state.mcp_dir.as_path(),
        api_server_port: api_server_port(state)?,
    };
    terminal_bridge::terminal_start_task_session(&ctx, args)?;
    Ok(serde_json::Value::Null)
}

pub fn terminal_resolve_task_launch_config(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: TerminalBridgeResolveTaskLaunchArgs =
        decode_args("terminal_resolve_task_launch_config", payload)?;
    let resolved: TerminalBridgeResolvedLaunchConfig =
        service_project::session::with_db(&state.db, |db| {
            service_terminal::terminal::resolve_task_launch_config(db, args.task_id, args.defaults)
        })?;
    encode_result("terminal_resolve_task_launch_config", resolved)
}

pub fn terminal_start_human_session(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: TerminalBridgeStartHumanSessionArgs =
        decode_args("terminal_start_human_session", payload)?;
    let project_path = service_project::session::locked_project_path(&state.locked_project)?;

    let ctx = terminal_bridge::TerminalBridgeCtx {
        pty_manager: &state.pty_manager,
        sink: Arc::clone(&state.event_sink),
        locked_project_path: project_path.as_path(),
        db: &state.db,
        codebase_snapshot: &state.codebase_snapshot,
        mcp_dir: state.mcp_dir.as_path(),
        api_server_port: api_server_port(state)?,
    };
    let result: TerminalBridgeStartHumanSessionResult =
        terminal_bridge::terminal_start_human_session(&ctx, args)?;
    encode_result("terminal_start_human_session", result)
}

pub fn terminal_list_model_form_tree(payload: serde_json::Value) -> RalphResult<serde_json::Value> {
    require_null_payload("terminal_list_model_form_tree", payload)?;
    let tree: TerminalBridgeListModelFormTreeResult = terminal_bridge::terminal_list_model_form_tree()?;
    encode_result("terminal_list_model_form_tree", tree)
}

pub fn terminal_send_input(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: TerminalBridgeSendInputArgs = decode_args("terminal_send_input", payload)?;
    terminal_bridge::terminal_send_input(&state.pty_manager, args)?;
    Ok(serde_json::Value::Null)
}

pub fn terminal_resize(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: TerminalBridgeResizeArgs = decode_args("terminal_resize", payload)?;
    terminal_bridge::terminal_resize(&state.pty_manager, args)?;
    Ok(serde_json::Value::Null)
}

pub fn terminal_set_stream_mode(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: TerminalBridgeSetStreamModeArgs = decode_args("terminal_set_stream_mode", payload)?;
    terminal_bridge::terminal_set_stream_mode(&state.pty_manager, args)?;
    Ok(serde_json::Value::Null)
}

pub fn terminal_replay_output(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: TerminalBridgeReplayOutputArgs = decode_args("terminal_replay_output", payload)?;
    let result: TerminalBridgeReplayOutputResult =
        terminal_bridge::terminal_replay_output(&state.pty_manager, args)?;
    encode_result("terminal_replay_output", result)
}

pub fn terminal_emit_system_message(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: TerminalBridgeEmitSystemMessageArgs =
        decode_args("terminal_emit_system_message", payload)?;
    terminal_bridge::emit_system_message(state.event_sink.as_ref(), args.session_id, args.text)?;
    Ok(serde_json::Value::Null)
}

pub fn terminal_terminate(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: TerminalBridgeTerminateArgs = decode_args("terminal_terminate", payload)?;
    terminal_bridge::terminal_terminate(&state.pty_manager, args)?;
    Ok(serde_json::Value::Null)
}
