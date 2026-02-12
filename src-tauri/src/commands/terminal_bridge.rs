use super::state::AppState;
use crate::terminal::{
    PtyOutputEvent, SessionConfig, TerminalBridgeEmitSystemMessageArgs, TerminalBridgeResizeArgs,
    TerminalBridgeSendInputArgs, TerminalBridgeStartSessionArgs,
    TerminalBridgeStartTaskSessionArgs, TerminalBridgeTerminateArgs, TERMINAL_BRIDGE_OUTPUT_EVENT,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use ralph_errors::{codes, ToStringErr};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

fn locked_project_path(state: &AppState) -> Result<PathBuf, String> {
    let locked = state.locked_project.lock().err_str(codes::INTERNAL)?;
    locked
        .as_ref()
        .cloned()
        .ok_or_else(|| ralph_errors::err_string(codes::PROJECT_LOCK, "No project locked"))
}

fn build_system_message_event(session_id: String, text: String) -> PtyOutputEvent {
    PtyOutputEvent {
        session_id,
        data: STANDARD.encode(text.as_bytes()),
    }
}

fn emit_system_message<R: tauri::Runtime>(
    app: &AppHandle<R>,
    session_id: String,
    text: String,
) -> Result<(), String> {
    let event = build_system_message_event(session_id, text);
    app.emit(TERMINAL_BRIDGE_OUTPUT_EVENT, event)
        .map_err(|e| e.to_string())
}

fn resolve_start_session_context(
    state: &AppState,
    mcp_mode: Option<&str>,
) -> Result<(PathBuf, Option<PathBuf>), String> {
    let project_path = locked_project_path(state)?;
    let mcp_config = if let Some(mode) = mcp_mode {
        Some(state.generate_mcp_config(mode, &project_path)?)
    } else {
        None
    };
    Ok((project_path, mcp_config))
}

fn resolve_start_task_session_context(
    state: &AppState,
    task_id: u32,
) -> Result<(PathBuf, PathBuf), String> {
    let project_path = locked_project_path(state)?;
    let mcp_config = state.generate_mcp_config_for_task(task_id, &project_path)?;
    Ok((project_path, mcp_config))
}

fn start_session_impl(
    app: AppHandle,
    state: &AppState,
    args: TerminalBridgeStartSessionArgs,
) -> Result<(), String> {
    let (project_path, mcp_config) =
        resolve_start_session_context(state, args.mcp_mode.as_deref())?;
    let config = SessionConfig {
        model: args.model,
        thinking: args.thinking,
    };

    state
        .pty_manager
        .create_session(app, args.session_id, &project_path, mcp_config, config)
}

fn start_task_session_impl(
    app: AppHandle,
    state: &AppState,
    args: TerminalBridgeStartTaskSessionArgs,
) -> Result<(), String> {
    let (project_path, mcp_config) = resolve_start_task_session_context(state, args.task_id)?;
    let config = SessionConfig {
        model: args.model,
        thinking: args.thinking,
    };

    state.pty_manager.create_session(
        app,
        args.session_id,
        &project_path,
        Some(mcp_config),
        config,
    )
}

fn send_input_impl(state: &AppState, args: TerminalBridgeSendInputArgs) -> Result<(), String> {
    state.pty_manager.send_input(&args.session_id, &args.data)
}

fn resize_impl(state: &AppState, args: TerminalBridgeResizeArgs) -> Result<(), String> {
    state
        .pty_manager
        .resize(&args.session_id, args.cols, args.rows)
}

fn terminate_impl(state: &AppState, args: TerminalBridgeTerminateArgs) -> Result<(), String> {
    state.pty_manager.terminate(&args.session_id)
}

fn emit_system_message_impl<R: tauri::Runtime>(
    app: &AppHandle<R>,
    args: TerminalBridgeEmitSystemMessageArgs,
) -> Result<(), String> {
    emit_system_message(app, args.session_id, args.text)
}

#[tauri::command]
pub fn terminal_bridge_start_session(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    mcp_mode: Option<String>,
    model: Option<String>,
    thinking: Option<bool>,
) -> Result<(), String> {
    start_session_impl(
        app,
        state.inner(),
        TerminalBridgeStartSessionArgs {
            session_id,
            mcp_mode,
            model,
            thinking,
        },
    )
}

#[tauri::command]
pub fn terminal_bridge_send_input(
    state: State<'_, AppState>,
    session_id: String,
    data: Vec<u8>,
) -> Result<(), String> {
    send_input_impl(
        state.inner(),
        TerminalBridgeSendInputArgs { session_id, data },
    )
}

#[tauri::command]
pub fn terminal_bridge_resize(
    state: State<'_, AppState>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    resize_impl(
        state.inner(),
        TerminalBridgeResizeArgs {
            session_id,
            cols,
            rows,
        },
    )
}

#[tauri::command]
pub fn terminal_bridge_terminate(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    terminate_impl(state.inner(), TerminalBridgeTerminateArgs { session_id })
}

#[tauri::command]
pub fn terminal_bridge_start_task_session(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    task_id: u32,
    model: Option<String>,
    thinking: Option<bool>,
) -> Result<(), String> {
    start_task_session_impl(
        app,
        state.inner(),
        TerminalBridgeStartTaskSessionArgs {
            session_id,
            task_id,
            model,
            thinking,
        },
    )
}

#[tauri::command]
pub fn terminal_bridge_emit_system_message(
    app: AppHandle,
    session_id: String,
    text: String,
) -> Result<(), String> {
    emit_system_message_impl(
        &app,
        TerminalBridgeEmitSystemMessageArgs { session_id, text },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::STANDARD;
    use std::path::PathBuf;
    use std::sync::mpsc;
    use std::time::Duration;
    use tauri::Listener;
    use tempfile::tempdir;

    #[test]
    fn locked_project_path_errors_when_unset() {
        let state = AppState::default();
        let err = locked_project_path(&state).unwrap_err();
        assert!(err.contains("No project locked"));
    }

    #[test]
    fn locked_project_path_returns_value_when_set() {
        let state = AppState::default();
        let expected = PathBuf::from("/tmp/ralph4days-test-project");
        *state.locked_project.lock().unwrap() = Some(expected.clone());
        let actual = locked_project_path(&state).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn build_system_message_event_encodes_text_as_base64() {
        let event =
            build_system_message_event("session-1".to_owned(), "[session started]\r\n".to_owned());
        assert_eq!(event.session_id, "session-1");
        let decoded = STANDARD.decode(event.data).unwrap();
        assert_eq!(String::from_utf8(decoded).unwrap(), "[session started]\r\n");
    }

    #[test]
    fn build_system_message_event_preserves_ansi_and_newlines() {
        let text = "\u{1b}[2m[session #001 started]\u{1b}[0m\r\n".to_owned();
        let event = build_system_message_event("session-ansi".to_owned(), text.clone());
        let decoded = STANDARD.decode(event.data).unwrap();
        assert_eq!(String::from_utf8(decoded).unwrap(), text);
    }

    #[test]
    fn terminal_bridge_emit_system_message_emits_output_event() {
        let app = tauri::test::mock_app();
        let (tx, rx) = mpsc::channel::<String>();
        let listener_id =
            app.listen_any(TERMINAL_BRIDGE_OUTPUT_EVENT, move |event: tauri::Event| {
                let _ = tx.send(event.payload().to_owned());
            });

        emit_system_message(
            &app.handle().clone(),
            "session-emission".to_owned(),
            "[session started]\r\n".to_owned(),
        )
        .unwrap();

        let payload_json = rx.recv_timeout(Duration::from_secs(1)).unwrap();
        let payload: serde_json::Value = serde_json::from_str(&payload_json).unwrap();
        assert_eq!(payload["session_id"], "session-emission");
        let decoded = STANDARD.decode(payload["data"].as_str().unwrap()).unwrap();
        assert_eq!(String::from_utf8(decoded).unwrap(), "[session started]\r\n");

        app.unlisten(listener_id);
    }

    #[test]
    fn send_input_impl_returns_missing_session_error() {
        let state = AppState::default();
        let err = send_input_impl(
            &state,
            TerminalBridgeSendInputArgs {
                session_id: "missing-session".to_owned(),
                data: b"echo hi\n".to_vec(),
            },
        )
        .unwrap_err();
        assert!(err.contains("No terminal bridge session: missing-session"));
    }

    #[test]
    fn resize_impl_returns_missing_session_error() {
        let state = AppState::default();
        let err = resize_impl(
            &state,
            TerminalBridgeResizeArgs {
                session_id: "missing-session".to_owned(),
                cols: 120,
                rows: 40,
            },
        )
        .unwrap_err();
        assert!(err.contains("No terminal bridge session: missing-session"));
    }

    #[test]
    fn terminate_impl_is_noop_for_missing_session() {
        let state = AppState::default();
        assert!(terminate_impl(
            &state,
            TerminalBridgeTerminateArgs {
                session_id: "missing-session".to_owned(),
            }
        )
        .is_ok());
    }

    fn app_state_with_locked_project(path: PathBuf) -> AppState {
        let state = AppState::default();
        *state.locked_project.lock().unwrap() = Some(path);
        state
    }

    #[test]
    fn resolve_start_session_context_errors_when_project_unlocked() {
        let state = AppState::default();
        let err = resolve_start_session_context(&state, Some("interactive")).unwrap_err();
        assert!(err.contains("No project locked"));
    }

    #[test]
    fn resolve_start_task_session_context_errors_when_project_unlocked() {
        let state = AppState::default();
        let err = resolve_start_task_session_context(&state, 7).unwrap_err();
        assert!(err.contains("No project locked"));
    }

    #[test]
    fn resolve_start_session_context_allows_no_mcp_config_without_db() {
        let dir = tempdir().unwrap();
        let state = app_state_with_locked_project(dir.path().to_path_buf());

        let (project_path, mcp_config) = resolve_start_session_context(&state, None).unwrap();
        assert_eq!(project_path, dir.path().to_path_buf());
        assert!(mcp_config.is_none());
    }

    #[test]
    fn resolve_start_session_context_requires_open_db_when_generating_mcp() {
        let dir = tempdir().unwrap();
        let state = app_state_with_locked_project(dir.path().to_path_buf());

        let err = resolve_start_session_context(&state, Some("interactive")).unwrap_err();
        assert!(err.contains("database not open"));
    }

    #[test]
    fn resolve_start_task_session_context_requires_open_db() {
        let dir = tempdir().unwrap();
        let state = app_state_with_locked_project(dir.path().to_path_buf());

        let err = resolve_start_task_session_context(&state, 42).unwrap_err();
        assert!(err.contains("database not open"));
    }
}
