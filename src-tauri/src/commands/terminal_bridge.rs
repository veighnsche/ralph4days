use super::state::{AppState, CommandContext, ProjectSessionService};
use crate::terminal::providers::{
    list_model_entries_for_agent, resolve_agent_provider, resolve_post_start_preamble,
    resolve_session_effort_for_agent, resolve_session_model_for_agent, shell_agent_enabled,
    AGENT_CLAUDE, AGENT_CODEX, AGENT_SHELL,
};
use crate::terminal::{
    PtyOutputEvent, SessionConfig, SessionInitSettings, SessionStreamMode,
    TerminalBridgeEmitSystemMessageArgs, TerminalBridgeListModelFormTreeResult,
    TerminalBridgeListModelsResult, TerminalBridgeModelOption, TerminalBridgeReplayOutputArgs,
    TerminalBridgeReplayOutputResult, TerminalBridgeResizeArgs, TerminalBridgeSendInputArgs,
    TerminalBridgeSetStreamModeArgs, TerminalBridgeStartHumanSessionArgs,
    TerminalBridgeStartHumanSessionResult, TerminalBridgeStartSessionArgs,
    TerminalBridgeStartTaskSessionArgs, TerminalBridgeTerminateArgs, TERMINAL_OUTPUT_EVENT,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, State};

static AGENT_SESSION_COUNTER: AtomicU64 = AtomicU64::new(0);

fn locked_project_path(state: &AppState) -> Result<PathBuf, String> {
    ProjectSessionService::new(state).locked_project_path()
}

fn build_system_message_event(session_id: String, text: String) -> PtyOutputEvent {
    PtyOutputEvent {
        session_id,
        seq: 0,
        data: STANDARD.encode(text.as_bytes()),
    }
}

fn emit_system_message<R: tauri::Runtime>(
    app: &AppHandle<R>,
    session_id: String,
    text: String,
) -> Result<(), String> {
    let event = build_system_message_event(session_id, text);
    app.emit(TERMINAL_OUTPUT_EVENT, event)
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

fn build_session_config(
    agent: Option<String>,
    selected_model: Option<String>,
    effort: Option<String>,
    thinking: Option<bool>,
    permission_level: Option<String>,
    post_start_preamble: Option<String>,
) -> Result<SessionConfig, String> {
    let provider_id = resolve_agent_provider(agent.as_deref()).id();
    if provider_id == AGENT_SHELL {
        if !shell_agent_enabled() {
            return Err("Shell terminal sessions are disabled in production builds".to_owned());
        }
        if selected_model
            .as_deref()
            .map(str::trim)
            .is_some_and(|value| !value.is_empty())
        {
            return Err("Shell terminal sessions do not support model selection".to_owned());
        }
        if effort
            .as_deref()
            .map(str::trim)
            .is_some_and(|value| !value.is_empty())
        {
            return Err("Shell terminal sessions do not support effort selection".to_owned());
        }
        let resolved_preamble = resolve_session_post_start_preamble(
            agent.as_deref(),
            None,
            None,
            thinking,
            post_start_preamble,
        );
        return Ok(SessionConfig {
            agent,
            model: None,
            effort: None,
            thinking,
            permission_level,
            init_settings: SessionInitSettings::default(),
            post_start_preamble: resolved_preamble,
        });
    }

    let runtime_model = resolve_session_model_for_agent(agent.as_deref(), selected_model.clone())?;
    let runtime_effort =
        resolve_session_effort_for_agent(agent.as_deref(), selected_model.as_deref(), effort)?;
    let resolved_preamble = resolve_session_post_start_preamble(
        agent.as_deref(),
        runtime_model.clone(),
        runtime_effort.clone(),
        thinking,
        post_start_preamble,
    );
    Ok(SessionConfig {
        agent,
        model: runtime_model,
        effort: runtime_effort,
        thinking,
        permission_level,
        init_settings: SessionInitSettings::default(),
        post_start_preamble: resolved_preamble,
    })
}

fn start_session_impl(
    app: AppHandle,
    state: &AppState,
    args: TerminalBridgeStartSessionArgs,
) -> Result<(), String> {
    tracing::debug!(
        session_id = %args.session_id,
        agent = ?args.agent,
        mcp_mode = ?args.mcp_mode,
        model = ?args.model,
        thinking = ?args.thinking,
        "terminal_start_session"
    );
    let (project_path, mcp_config) =
        resolve_start_session_context(state, args.mcp_mode.as_deref())?;
    let config = build_session_config(
        args.agent,
        args.model,
        args.effort,
        args.thinking,
        args.permission_level,
        args.post_start_preamble,
    )?;

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
        agent = ?args.agent,
        model = ?args.model,
        thinking = ?args.thinking,
        "terminal_start_task_session"
    );
    let (project_path, mcp_config) = resolve_start_task_session_context(state, args.task_id)?;
    let config = build_session_config(
        args.agent,
        args.model,
        args.effort,
        args.thinking,
        args.permission_level,
        args.post_start_preamble,
    )?;

    state.pty_manager.create_session(
        app,
        args.session_id,
        &project_path,
        Some(mcp_config),
        config,
    )
}

fn send_input_impl(state: &AppState, args: TerminalBridgeSendInputArgs) -> Result<(), String> {
    tracing::debug!(
        session_id = %args.session_id,
        byte_count = args.data.len(),
        text = %String::from_utf8_lossy(&args.data).escape_debug().to_string(),
        "terminal_send_input"
    );
    state.pty_manager.send_input(&args.session_id, &args.data)
}

fn resize_impl(state: &AppState, args: TerminalBridgeResizeArgs) -> Result<(), String> {
    tracing::trace!(
        session_id = %args.session_id,
        cols = args.cols,
        rows = args.rows,
        "terminal_resize"
    );
    state
        .pty_manager
        .resize(&args.session_id, args.cols, args.rows)
}

fn terminate_impl(state: &AppState, args: TerminalBridgeTerminateArgs) -> Result<(), String> {
    tracing::debug!(session_id = %args.session_id, "terminal_terminate");
    state.pty_manager.terminate(&args.session_id)
}

fn set_stream_mode_impl(
    state: &AppState,
    args: TerminalBridgeSetStreamModeArgs,
) -> Result<(), String> {
    let mode = SessionStreamMode::parse(&args.mode)?;
    tracing::debug!(
        session_id = %args.session_id,
        mode = %args.mode,
        "terminal_set_stream_mode"
    );
    state.pty_manager.set_stream_mode(&args.session_id, mode)
}

fn replay_output_impl(
    state: &AppState,
    args: TerminalBridgeReplayOutputArgs,
) -> Result<TerminalBridgeReplayOutputResult, String> {
    tracing::trace!(
        session_id = %args.session_id,
        after_seq = args.after_seq,
        limit = args.limit,
        "terminal_replay_output"
    );
    state
        .pty_manager
        .replay_output(&args.session_id, args.after_seq, args.limit as usize)
}

fn emit_system_message_impl<R: tauri::Runtime>(
    app: &AppHandle<R>,
    args: TerminalBridgeEmitSystemMessageArgs,
) -> Result<(), String> {
    tracing::debug!(
        session_id = %args.session_id,
        text = %args.text.escape_debug().to_string(),
        "terminal_emit_system_message"
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

fn resolve_session_post_start_preamble(
    agent: Option<&str>,
    model: Option<String>,
    effort: Option<String>,
    thinking: Option<bool>,
    user_preamble: Option<String>,
) -> Option<String> {
    let config = SessionConfig {
        agent: agent.map(str::to_owned),
        model,
        effort,
        thinking,
        permission_level: None,
        init_settings: SessionInitSettings::default(),
        post_start_preamble: None,
    };
    resolve_post_start_preamble(agent, &config, user_preamble)
}

fn build_launch_command(config: &SessionConfig) -> String {
    let agent = resolve_agent_provider(config.agent.as_deref()).id();
    if agent == AGENT_SHELL {
        return "shell -i".to_owned();
    }

    let mut parts = vec![agent.to_owned()];

    if let Some(model) = config.model.as_deref() {
        parts.push("--model".to_owned());
        parts.push(model.to_owned());
    }
    if let Some(effort) = config.effort.as_deref() {
        if agent == AGENT_CODEX {
            parts.push("--config".to_owned());
            parts.push(format!("model_reasoning_effort={effort}"));
        } else {
            parts.push("--effort".to_owned());
            parts.push(effort.to_owned());
        }
    }

    if agent == AGENT_CODEX {
        match config.permission_level.as_deref().map(str::trim) {
            Some("safe") => {
                parts.push("--sandbox".to_owned());
                parts.push("workspace-write".to_owned());
                parts.push("--ask-for-approval".to_owned());
                parts.push("untrusted".to_owned());
            }
            Some("auto") => {
                parts.push("--full-auto".to_owned());
            }
            Some("full_auto") => {
                parts.push("--dangerously-bypass-approvals-and-sandbox".to_owned());
            }
            _ => {
                parts.push("--sandbox".to_owned());
                parts.push("workspace-write".to_owned());
                parts.push("--ask-for-approval".to_owned());
                parts.push("on-request".to_owned());
            }
        }
    } else {
        parts.push("--permission-mode".to_owned());
        parts.push(
            match config.permission_level.as_deref().map(str::trim) {
                Some("safe") => "default",
                Some("auto") => "dontAsk",
                Some("full_auto") => "bypassPermissions",
                _ => "delegate",
            }
            .to_owned(),
        );
    }

    parts.join(" ")
}

#[tauri::command]
pub fn terminal_start_session(
    app: AppHandle,
    state: State<'_, AppState>,
    args: TerminalBridgeStartSessionArgs,
) -> Result<(), String> {
    start_session_impl(app, state.inner(), args)
}

#[tauri::command]
pub fn terminal_send_input(
    state: State<'_, AppState>,
    args: TerminalBridgeSendInputArgs,
) -> Result<(), String> {
    send_input_impl(state.inner(), args)
}

#[tauri::command]
pub fn terminal_resize(
    state: State<'_, AppState>,
    args: TerminalBridgeResizeArgs,
) -> Result<(), String> {
    resize_impl(state.inner(), args)
}

#[tauri::command]
pub fn terminal_terminate(
    state: State<'_, AppState>,
    args: TerminalBridgeTerminateArgs,
) -> Result<(), String> {
    terminate_impl(state.inner(), args)
}

#[tauri::command]
pub fn terminal_set_stream_mode(
    state: State<'_, AppState>,
    args: TerminalBridgeSetStreamModeArgs,
) -> Result<(), String> {
    set_stream_mode_impl(state.inner(), args)
}

#[tauri::command]
pub fn terminal_replay_output(
    state: State<'_, AppState>,
    args: TerminalBridgeReplayOutputArgs,
) -> Result<TerminalBridgeReplayOutputResult, String> {
    replay_output_impl(state.inner(), args)
}

#[tauri::command]
pub fn terminal_start_task_session(
    app: AppHandle,
    state: State<'_, AppState>,
    args: TerminalBridgeStartTaskSessionArgs,
) -> Result<(), String> {
    start_task_session_impl(app, state.inner(), args)
}

#[tauri::command]
pub fn terminal_emit_system_message(
    app: AppHandle,
    args: TerminalBridgeEmitSystemMessageArgs,
) -> Result<(), String> {
    emit_system_message_impl(&app, args)
}

#[tauri::command]
pub fn terminal_start_human_session(
    app: AppHandle,
    state: State<'_, AppState>,
    args: TerminalBridgeStartHumanSessionArgs,
) -> Result<TerminalBridgeStartHumanSessionResult, String> {
    tracing::debug!(
        terminal_session_id = %args.terminal_session_id,
        kind = %args.kind,
        task_id = ?args.task_id,
        agent = ?args.agent,
        model = ?args.model,
        mcp_mode = ?args.mcp_mode,
        thinking = ?args.thinking,
        "terminal_start_human_session"
    );

    let session_config = build_session_config(
        args.agent.clone(),
        args.model.clone(),
        args.effort.clone(),
        args.thinking,
        args.permission_level.clone(),
        args.post_start_preamble.clone(),
    )?;
    let launch_command = build_launch_command(&session_config);

    let resolved_post_start_preamble = session_config.post_start_preamble;

    let agent_session_id = generate_agent_session_id();
    tracing::debug!(
        terminal_session_id = %args.terminal_session_id,
        agent_session_id = %agent_session_id,
        "terminal_start_human_session.created_agent_session_id"
    );

    let command_ctx = CommandContext::from_tauri_state(&state);

    let agent_session_number = command_ctx.db_tx(|db| {
        db.create_human_agent_session(sqlite_db::AgentSessionCreateInput {
            id: agent_session_id.clone(),
            kind: args.kind.clone(),
            task_id: args.task_id,
            agent: args.agent.clone(),
            model: args.model.clone(),
            launch_command: Some(launch_command),
            post_start_preamble: resolved_post_start_preamble,
            init_prompt: args.init_prompt.clone(),
        })?;

        db.get_agent_session_by_id(&agent_session_id)
            .map(|s| s.session_number)
            .ok_or_else(|| {
                format!("Failed to load newly created agent session '{agent_session_id}'")
            })
    })?;

    let start_result = if let Some(task_id) = args.task_id {
        start_task_session_impl(
            app.clone(),
            state.inner(),
            TerminalBridgeStartTaskSessionArgs {
                session_id: args.terminal_session_id.clone(),
                task_id,
                agent: args.agent.clone(),
                model: args.model.clone(),
                effort: args.effort.clone(),
                permission_level: args.permission_level.clone(),
                thinking: args.thinking,
                post_start_preamble: args.post_start_preamble.clone(),
            },
        )
    } else {
        start_session_impl(
            app.clone(),
            state.inner(),
            TerminalBridgeStartSessionArgs {
                session_id: args.terminal_session_id.clone(),
                agent: args.agent.clone(),
                mcp_mode: args.mcp_mode.clone(),
                model: args.model.clone(),
                effort: args.effort.clone(),
                permission_level: args.permission_level.clone(),
                thinking: args.thinking,
                post_start_preamble: args.post_start_preamble.clone(),
            },
        )
    };

    if let Err(err) = start_result {
        let _ = command_ctx.db(|db| {
            db.update_human_agent_session(sqlite_db::AgentSessionUpdateInput {
                id: agent_session_id.clone(),
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
            })
        });
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

fn list_models_for_agent(agent: &str) -> TerminalBridgeListModelsResult {
    let provider = resolve_agent_provider(Some(agent));
    let models = list_model_entries_for_agent(Some(agent))
        .into_iter()
        .map(TerminalBridgeModelOption::from)
        .collect();
    TerminalBridgeListModelsResult {
        agent: provider.id().to_owned(),
        models,
    }
}

#[tauri::command]
pub fn terminal_list_model_form_tree() -> TerminalBridgeListModelFormTreeResult {
    let mut providers = vec![
        list_models_for_agent(AGENT_CODEX),
        list_models_for_agent(AGENT_CLAUDE),
    ];
    if shell_agent_enabled() {
        providers.push(list_models_for_agent(AGENT_SHELL));
    }

    TerminalBridgeListModelFormTreeResult { providers }
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
        assert_eq!(event.seq, 0);
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
    fn build_launch_command_for_codex_uses_reasoning_effort_config_override() {
        let config = SessionConfig {
            agent: Some(AGENT_CODEX.to_owned()),
            model: Some("gpt-5.3-codex".to_owned()),
            effort: Some("high".to_owned()),
            thinking: None,
            permission_level: None,
            init_settings: SessionInitSettings::default(),
            post_start_preamble: None,
        };

        let launch = build_launch_command(&config);
        assert!(launch.contains("--config model_reasoning_effort=high"));
        assert!(!launch.contains("--effort "));
    }

    #[test]
    fn terminal_emit_system_message_emits_output_event() {
        let app = tauri::test::mock_app();
        let (tx, rx) = mpsc::channel::<String>();
        let listener_id = app.listen_any(TERMINAL_OUTPUT_EVENT, move |event: tauri::Event| {
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
        assert_eq!(payload["sessionId"], "session-emission");
        assert_eq!(payload["seq"], 0);
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

    #[test]
    fn set_stream_mode_impl_returns_missing_session_error() {
        let state = AppState::default();
        let err = set_stream_mode_impl(
            &state,
            TerminalBridgeSetStreamModeArgs {
                session_id: "missing-session".to_owned(),
                mode: "buffered".to_owned(),
            },
        )
        .unwrap_err();
        assert!(err.contains("No terminal bridge session: missing-session"));
    }

    #[test]
    fn set_stream_mode_impl_rejects_invalid_mode() {
        let state = AppState::default();
        let err = set_stream_mode_impl(
            &state,
            TerminalBridgeSetStreamModeArgs {
                session_id: "missing-session".to_owned(),
                mode: "invalid".to_owned(),
            },
        )
        .unwrap_err();
        assert!(err.contains("Invalid stream mode"));
    }

    #[test]
    fn replay_output_impl_returns_missing_session_error() {
        let state = AppState::default();
        let err = replay_output_impl(
            &state,
            TerminalBridgeReplayOutputArgs {
                session_id: "missing-session".to_owned(),
                after_seq: 0,
                limit: 64,
            },
        )
        .unwrap_err();
        assert!(err.contains("No terminal bridge session: missing-session"));
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
