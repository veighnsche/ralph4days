use super::executor::{dispatch_args, dispatch_no_args, PlatformArg, PlatformOut};
use super::state::AppState;
#[cfg(not(mobile))]
use crate::event_sink::TauriEventSink;
use ralph_contracts::terminal_bridge::{
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
    app: AppHandle,
    state: State<'_, AppState>,
    args: PlatformArg<TerminalBridgeStartSessionArgs>,
) -> RalphResult<()> {
    dispatch_args(state.inner(), "terminal_start_session", args, |args| {
        local::terminal_start_session(&state, app, args)
    })
    .await
}

#[tauri::command]
pub async fn terminal_start_task_session(
    app: AppHandle,
    state: State<'_, AppState>,
    args: PlatformArg<TerminalBridgeStartTaskSessionArgs>,
) -> RalphResult<()> {
    dispatch_args(state.inner(), "terminal_start_task_session", args, |args| {
        local::terminal_start_task_session(&state, app, args)
    })
    .await
}

#[tauri::command]
pub async fn terminal_resolve_task_launch_config(
    state: State<'_, AppState>,
    args: PlatformArg<TerminalBridgeResolveTaskLaunchArgs>,
) -> RalphResult<PlatformOut<TerminalBridgeResolvedLaunchConfig>> {
    dispatch_args(
        state.inner(),
        "terminal_resolve_task_launch_config",
        args,
        |args| local::terminal_resolve_task_launch_config(&state, args),
    )
    .await
}

#[tauri::command]
pub async fn terminal_send_input(
    state: State<'_, AppState>,
    args: PlatformArg<TerminalBridgeSendInputArgs>,
) -> RalphResult<()> {
    dispatch_args(state.inner(), "terminal_send_input", args, |args| {
        local::terminal_send_input(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn terminal_resize(
    state: State<'_, AppState>,
    args: PlatformArg<TerminalBridgeResizeArgs>,
) -> RalphResult<()> {
    dispatch_args(state.inner(), "terminal_resize", args, |args| {
        local::terminal_resize(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn terminal_terminate(
    state: State<'_, AppState>,
    args: PlatformArg<TerminalBridgeTerminateArgs>,
) -> RalphResult<()> {
    dispatch_args(state.inner(), "terminal_terminate", args, |args| {
        local::terminal_terminate(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn terminal_set_stream_mode(
    state: State<'_, AppState>,
    args: PlatformArg<TerminalBridgeSetStreamModeArgs>,
) -> RalphResult<()> {
    dispatch_args(state.inner(), "terminal_set_stream_mode", args, |args| {
        local::terminal_set_stream_mode(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn terminal_replay_output(
    state: State<'_, AppState>,
    args: PlatformArg<TerminalBridgeReplayOutputArgs>,
) -> RalphResult<PlatformOut<TerminalBridgeReplayOutputResult>> {
    dispatch_args(state.inner(), "terminal_replay_output", args, |args| {
        local::terminal_replay_output(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn terminal_emit_system_message(
    app: AppHandle,
    state: State<'_, AppState>,
    args: PlatformArg<TerminalBridgeEmitSystemMessageArgs>,
) -> RalphResult<()> {
    dispatch_args(
        state.inner(),
        "terminal_emit_system_message",
        args,
        |args| local::terminal_emit_system_message(app, args),
    )
    .await
}

#[tauri::command]
pub async fn terminal_start_human_session(
    app: AppHandle,
    state: State<'_, AppState>,
    args: PlatformArg<TerminalBridgeStartHumanSessionArgs>,
) -> RalphResult<PlatformOut<TerminalBridgeStartHumanSessionResult>> {
    dispatch_args(
        state.inner(),
        "terminal_start_human_session",
        args,
        |args| local::terminal_start_human_session(&state, app, args),
    )
    .await
}

#[tauri::command]
pub async fn terminal_list_model_form_tree(
    state: State<'_, AppState>,
) -> RalphResult<PlatformOut<TerminalBridgeListModelFormTreeResult>> {
    dispatch_no_args(state.inner(), "terminal_list_model_form_tree", || {
        #[cfg(not(mobile))]
        {
            Ok(ralph_backend::terminal_bridge::terminal_list_model_form_tree())
        }

        #[cfg(mobile)]
        {
            local::unreachable_local("terminal_list_model_form_tree")
        }
    })
    .await
}

mod local {
    use super::*;

    pub(super) fn terminal_start_session(
        state: &State<'_, AppState>,
        app: AppHandle,
        args: PlatformArg<TerminalBridgeStartSessionArgs>,
    ) -> RalphResult<()> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            use ralph_contracts::transport::EventSink;
            use ralph_errors::{codes, RalphResultExt};
            use std::sync::Arc;

            let project_path = CommandContext::from_tauri_state(state).locked_project_path()?;
            let sink: Arc<dyn EventSink> = Arc::new(TauriEventSink::new(app));
            let api_server_port = *state
                .inner()
                .api_server_port
                .lock()
                .ralph_err(codes::INTERNAL, "API server port mutex poisoned")?;

            let ctx = ralph_backend::terminal_bridge::TerminalBridgeCtx {
                pty_manager: &state.inner().pty_manager,
                sink,
                locked_project_path: project_path.as_path(),
                db: &state.inner().db,
                codebase_snapshot: &state.inner().codebase_snapshot,
                mcp_dir: state.inner().mcp_dir.as_path(),
                api_server_port,
            };
            ralph_backend::terminal_bridge::terminal_start_session(&ctx, args)
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = app;
            let _ = args;
            unreachable_local("terminal_start_session")
        }
    }

    pub(super) fn terminal_start_task_session(
        state: &State<'_, AppState>,
        app: AppHandle,
        args: PlatformArg<TerminalBridgeStartTaskSessionArgs>,
    ) -> RalphResult<()> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            use ralph_contracts::transport::EventSink;
            use ralph_errors::{codes, RalphResultExt};
            use std::sync::Arc;

            let project_path = CommandContext::from_tauri_state(state).locked_project_path()?;
            let sink: Arc<dyn EventSink> = Arc::new(TauriEventSink::new(app));
            let api_server_port = *state
                .inner()
                .api_server_port
                .lock()
                .ralph_err(codes::INTERNAL, "API server port mutex poisoned")?;

            let ctx = ralph_backend::terminal_bridge::TerminalBridgeCtx {
                pty_manager: &state.inner().pty_manager,
                sink,
                locked_project_path: project_path.as_path(),
                db: &state.inner().db,
                codebase_snapshot: &state.inner().codebase_snapshot,
                mcp_dir: state.inner().mcp_dir.as_path(),
                api_server_port,
            };
            ralph_backend::terminal_bridge::terminal_start_task_session(&ctx, args)
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = app;
            let _ = args;
            unreachable_local("terminal_start_task_session")
        }
    }

    pub(super) fn terminal_resolve_task_launch_config(
        state: &State<'_, AppState>,
        args: PlatformArg<TerminalBridgeResolveTaskLaunchArgs>,
    ) -> RalphResult<PlatformOut<TerminalBridgeResolvedLaunchConfig>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;

            CommandContext::from_tauri_state(state).db(|db| {
                ralph_backend::terminal::resolve_task_launch_config(db, args.task_id, args.defaults)
            })
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("terminal_resolve_task_launch_config")
        }
    }

    pub(super) fn terminal_send_input(
        state: &State<'_, AppState>,
        args: PlatformArg<TerminalBridgeSendInputArgs>,
    ) -> RalphResult<()> {
        #[cfg(not(mobile))]
        {
            ralph_backend::terminal_bridge::terminal_send_input(&state.inner().pty_manager, args)
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("terminal_send_input")
        }
    }

    pub(super) fn terminal_resize(
        state: &State<'_, AppState>,
        args: PlatformArg<TerminalBridgeResizeArgs>,
    ) -> RalphResult<()> {
        #[cfg(not(mobile))]
        {
            ralph_backend::terminal_bridge::terminal_resize(&state.inner().pty_manager, args)
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("terminal_resize")
        }
    }

    pub(super) fn terminal_terminate(
        state: &State<'_, AppState>,
        args: PlatformArg<TerminalBridgeTerminateArgs>,
    ) -> RalphResult<()> {
        #[cfg(not(mobile))]
        {
            ralph_backend::terminal_bridge::terminal_terminate(&state.inner().pty_manager, args)
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("terminal_terminate")
        }
    }

    pub(super) fn terminal_set_stream_mode(
        state: &State<'_, AppState>,
        args: PlatformArg<TerminalBridgeSetStreamModeArgs>,
    ) -> RalphResult<()> {
        #[cfg(not(mobile))]
        {
            ralph_backend::terminal_bridge::terminal_set_stream_mode(
                &state.inner().pty_manager,
                args,
            )
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("terminal_set_stream_mode")
        }
    }

    pub(super) fn terminal_replay_output(
        state: &State<'_, AppState>,
        args: PlatformArg<TerminalBridgeReplayOutputArgs>,
    ) -> RalphResult<PlatformOut<TerminalBridgeReplayOutputResult>> {
        #[cfg(not(mobile))]
        {
            ralph_backend::terminal_bridge::terminal_replay_output(&state.inner().pty_manager, args)
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("terminal_replay_output")
        }
    }

    pub(super) fn terminal_emit_system_message(
        app: AppHandle,
        args: PlatformArg<TerminalBridgeEmitSystemMessageArgs>,
    ) -> RalphResult<()> {
        #[cfg(not(mobile))]
        {
            let sink = TauriEventSink::new(app);
            ralph_backend::terminal_bridge::emit_system_message(&sink, args.session_id, args.text)
        }

        #[cfg(mobile)]
        {
            let _ = app;
            let _ = args;
            unreachable_local("terminal_emit_system_message")
        }
    }

    pub(super) fn terminal_start_human_session(
        state: &State<'_, AppState>,
        app: AppHandle,
        args: PlatformArg<TerminalBridgeStartHumanSessionArgs>,
    ) -> RalphResult<PlatformOut<TerminalBridgeStartHumanSessionResult>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            use ralph_contracts::transport::EventSink;
            use ralph_errors::{codes, RalphResultExt};
            use std::sync::Arc;

            let project_path = CommandContext::from_tauri_state(state).locked_project_path()?;
            let sink: Arc<dyn EventSink> = Arc::new(TauriEventSink::new(app));
            let api_server_port = *state
                .inner()
                .api_server_port
                .lock()
                .ralph_err(codes::INTERNAL, "API server port mutex poisoned")?;

            let ctx = ralph_backend::terminal_bridge::TerminalBridgeCtx {
                pty_manager: &state.inner().pty_manager,
                sink,
                locked_project_path: project_path.as_path(),
                db: &state.inner().db,
                codebase_snapshot: &state.inner().codebase_snapshot,
                mcp_dir: state.inner().mcp_dir.as_path(),
                api_server_port,
            };
            ralph_backend::terminal_bridge::terminal_start_human_session(&ctx, args)
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = app;
            let _ = args;
            unreachable_local("terminal_start_human_session")
        }
    }

    #[cfg(mobile)]
    pub(super) fn unreachable_local<TResult>(command: &str) -> RalphResult<TResult> {
        use ralph_errors::{codes, ralph_err};
        ralph_err!(
            codes::INTERNAL,
            "Local execution path reached on mobile for '{}'",
            command
        )
    }
}
