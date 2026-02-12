use super::state::{get_db, AppState};
use crate::terminal::{
    PtyOutputEvent, SessionConfig, TerminalBridgeEmitSystemMessageArgs, TerminalBridgeResizeArgs,
    TerminalBridgeSendInputArgs, TerminalBridgeStartHumanSessionArgs,
    TerminalBridgeStartHumanSessionResult, TerminalBridgeStartSessionArgs,
    TerminalBridgeStartTaskSessionArgs, TerminalBridgeTerminateArgs, TERMINAL_BRIDGE_OUTPUT_EVENT,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use ralph_errors::{codes, ToStringErr};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, State};

static AGENT_SESSION_COUNTER: AtomicU64 = AtomicU64::new(0);

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
    tracing::debug!(
        session_id = %args.session_id,
        mcp_mode = ?args.mcp_mode,
        model = ?args.model,
        thinking = ?args.thinking,
        "terminal_bridge_start_session"
    );
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
    tracing::debug!(
        session_id = %args.session_id,
        task_id = args.task_id,
        model = ?args.model,
        thinking = ?args.thinking,
        "terminal_bridge_start_task_session"
    );
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
    tracing::trace!(
        session_id = %args.session_id,
        byte_count = args.data.len(),
        text = %String::from_utf8_lossy(&args.data).escape_debug().to_string(),
        "terminal_bridge_send_input"
    );
    state.pty_manager.send_input(&args.session_id, &args.data)
}

fn resize_impl(state: &AppState, args: TerminalBridgeResizeArgs) -> Result<(), String> {
    tracing::trace!(
        session_id = %args.session_id,
        cols = args.cols,
        rows = args.rows,
        "terminal_bridge_resize"
    );
    state
        .pty_manager
        .resize(&args.session_id, args.cols, args.rows)
}

fn terminate_impl(state: &AppState, args: TerminalBridgeTerminateArgs) -> Result<(), String> {
    tracing::debug!(session_id = %args.session_id, "terminal_bridge_terminate");
    state.pty_manager.terminate(&args.session_id)
}

fn emit_system_message_impl<R: tauri::Runtime>(
    app: &AppHandle<R>,
    args: TerminalBridgeEmitSystemMessageArgs,
) -> Result<(), String> {
    tracing::debug!(
        session_id = %args.session_id,
        text = %args.text.escape_debug().to_string(),
        "terminal_bridge_emit_system_message"
    );
    emit_system_message(app, args.session_id, args.text)
}

fn generate_agent_session_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0u128, |d| d.as_millis());
    let counter = AGENT_SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("agent-session-{millis}-{counter}")
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

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn terminal_bridge_start_human_session(
    app: AppHandle,
    state: State<'_, AppState>,
    terminal_session_id: String,
    kind: String,
    task_id: Option<u32>,
    agent: Option<String>,
    model: Option<String>,
    launch_command: Option<String>,
    post_start_preamble: Option<String>,
    init_prompt: Option<String>,
    mcp_mode: Option<String>,
    thinking: Option<bool>,
) -> Result<TerminalBridgeStartHumanSessionResult, String> {
    tracing::debug!(
        terminal_session_id = %terminal_session_id,
        kind = %kind,
        task_id = ?task_id,
        agent = ?agent,
        model = ?model,
        mcp_mode = ?mcp_mode,
        thinking = ?thinking,
        "terminal_bridge_start_human_session"
    );
    let args = TerminalBridgeStartHumanSessionArgs {
        terminal_session_id,
        kind,
        task_id,
        agent,
        model,
        launch_command,
        post_start_preamble,
        init_prompt,
        mcp_mode,
        thinking,
    };

    let agent_session_id = generate_agent_session_id();
    tracing::debug!(
        terminal_session_id = %args.terminal_session_id,
        agent_session_id = %agent_session_id,
        "terminal_bridge_start_human_session.created_agent_session_id"
    );

    let db = get_db(&state)?;
    db.create_human_agent_session(sqlite_db::AgentSessionCreateInput {
        id: agent_session_id.clone(),
        kind: args.kind.clone(),
        task_id: args.task_id,
        agent: args.agent.clone(),
        model: args.model.clone(),
        launch_command: args.launch_command.clone(),
        post_start_preamble: args.post_start_preamble.clone(),
        init_prompt: args.init_prompt.clone(),
    })?;

    let agent_session_number = {
        let db = get_db(&state)?;
        db.get_agent_session_by_id(&agent_session_id)
            .map(|s| s.session_number)
            .ok_or_else(|| {
                format!("Failed to load newly created agent session '{agent_session_id}'")
            })?
    };

    let start_result = if let Some(task_id) = args.task_id {
        start_task_session_impl(
            app.clone(),
            state.inner(),
            TerminalBridgeStartTaskSessionArgs {
                session_id: args.terminal_session_id.clone(),
                task_id,
                model: args.model.clone(),
                thinking: args.thinking,
            },
        )
    } else {
        start_session_impl(
            app.clone(),
            state.inner(),
            TerminalBridgeStartSessionArgs {
                session_id: args.terminal_session_id.clone(),
                mcp_mode: args.mcp_mode.clone(),
                model: args.model.clone(),
                thinking: args.thinking,
            },
        )
    };

    if let Err(err) = start_result {
        if let Ok(db) = get_db(&state) {
            let _ = db.update_human_agent_session(sqlite_db::AgentSessionUpdateInput {
                id: agent_session_id,
                kind: None,
                task_id: None,
                agent: None,
                model: None,
                launch_command: None,
                post_start_preamble: None,
                init_prompt: None,
                ended: Some(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()),
                exit_code: Some(1),
                closing_verb: None,
                status: Some("crashed".to_owned()),
                prompt_hash: None,
                output_bytes: None,
                error_text: Some(err.clone()),
            });
        }
        return Err(err);
    }

    let connected_line =
        format!("\x1b[2m[connected to agent_session #{agent_session_number:03}]\x1b[0m\r\n");
    emit_system_message(&app, args.terminal_session_id, connected_line)?;

    Ok(TerminalBridgeStartHumanSessionResult {
        agent_session_id,
        agent_session_number,
    })
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
