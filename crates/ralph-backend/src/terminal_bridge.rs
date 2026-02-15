use crate::mcp::{generate_mcp_config, generate_mcp_config_for_task};
use crate::session::{with_db, with_db_tx};
use crate::terminal::providers::{
    list_model_entries_for_agent, resolve_agent_provider, resolve_post_start_preamble,
    resolve_session_effort_for_agent, resolve_session_model_for_agent, shell_agent_enabled,
    AGENT_CLAUDE, AGENT_CODEX, AGENT_SHELL,
};
use crate::terminal::{
    resolve_task_launch_config, PTYManager, PtyOutputEvent, SessionConfig, SessionInitSettings,
    SessionStreamMode, TerminalBridgeLaunchDefaults, TerminalBridgeListModelFormTreeResult,
    TerminalBridgeListModelsResult, TerminalBridgeModelOption, TerminalBridgeReplayOutputArgs,
    TerminalBridgeReplayOutputResult, TerminalBridgeResizeArgs, TerminalBridgeResolvedLaunchConfig,
    TerminalBridgeSendInputArgs, TerminalBridgeSetStreamModeArgs,
    TerminalBridgeStartHumanSessionArgs, TerminalBridgeStartHumanSessionResult,
    TerminalBridgeStartSessionArgs, TerminalBridgeStartTaskSessionArgs,
    TerminalBridgeTerminateArgs,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use prompt_builder::CodebaseSnapshot;
use ralph_contracts::transport::EventSink;
use ralph_errors::{codes, err_string, ralph_err, RalphResult, RalphResultExt};
use sqlite_db::SqliteDb;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

static AGENT_SESSION_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct TerminalBridgeCtx<'a> {
    pub pty_manager: &'a PTYManager,
    pub sink: Arc<dyn EventSink>,
    pub locked_project_path: &'a Path,
    pub db: &'a Mutex<Option<SqliteDb>>,
    pub codebase_snapshot: &'a Mutex<Option<CodebaseSnapshot>>,
    pub mcp_dir: &'a Path,
    pub api_server_port: Option<u16>,
}

pub fn build_system_message_event(session_id: String, text: String) -> PtyOutputEvent {
    PtyOutputEvent {
        session_id,
        seq: 0,
        data: STANDARD.encode(text.as_bytes()),
    }
}

pub fn emit_system_message(
    sink: &dyn EventSink,
    session_id: String,
    text: String,
) -> RalphResult<()> {
    sink.emit_terminal_output(build_system_message_event(session_id, text))
        .ralph_err(codes::INTERNAL, "Failed to emit terminal output")
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

fn build_session_config(
    agent: Option<String>,
    selected_model: Option<String>,
    effort: Option<String>,
    thinking: Option<bool>,
    permission_level: Option<String>,
    post_start_preamble: Option<String>,
) -> RalphResult<SessionConfig> {
    let provider_id = resolve_agent_provider(agent.as_deref()).id();
    if provider_id == AGENT_SHELL {
        if !shell_agent_enabled() {
            return ralph_err!(
                codes::TERMINAL,
                "Shell terminal sessions are disabled in production builds"
            );
        }
        if selected_model
            .as_deref()
            .map(str::trim)
            .is_some_and(|value| !value.is_empty())
        {
            return ralph_err!(
                codes::TERMINAL,
                "Shell terminal sessions do not support model selection"
            );
        }
        if effort
            .as_deref()
            .map(str::trim)
            .is_some_and(|value| !value.is_empty())
        {
            return ralph_err!(
                codes::TERMINAL,
                "Shell terminal sessions do not support effort selection"
            );
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

pub fn build_launch_command(config: &SessionConfig) -> String {
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

pub fn generate_agent_session_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0u128, |d| d.as_millis());
    let counter = AGENT_SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("agent-session-{millis}-{counter}")
}

pub fn resolve_start_session_context(
    db: &Mutex<Option<SqliteDb>>,
    codebase_snapshot: &Mutex<Option<CodebaseSnapshot>>,
    mcp_dir: &Path,
    api_server_port: Option<u16>,
    project_path: &Path,
    mcp_mode: Option<&str>,
) -> RalphResult<(PathBuf, Option<PathBuf>)> {
    if !project_path.is_dir() {
        return ralph_err!(
            codes::PROJECT_LOCK,
            "Locked project path is not a directory: {}",
            project_path.display()
        );
    }
    let mcp_config = if let Some(mode) = mcp_mode {
        Some(generate_mcp_config(
            db,
            codebase_snapshot,
            mcp_dir,
            api_server_port,
            mode,
            project_path,
        )?)
    } else {
        None
    };
    Ok((project_path.to_path_buf(), mcp_config))
}

pub fn resolve_start_task_session_context(
    db: &Mutex<Option<SqliteDb>>,
    codebase_snapshot: &Mutex<Option<CodebaseSnapshot>>,
    mcp_dir: &Path,
    api_server_port: Option<u16>,
    project_path: &Path,
    task_id: u32,
) -> RalphResult<(PathBuf, PathBuf)> {
    if !project_path.is_dir() {
        return ralph_err!(
            codes::PROJECT_LOCK,
            "Locked project path is not a directory: {}",
            project_path.display()
        );
    }
    let mcp_config = generate_mcp_config_for_task(
        db,
        codebase_snapshot,
        mcp_dir,
        api_server_port,
        task_id,
        project_path,
    )?;
    Ok((project_path.to_path_buf(), mcp_config))
}

pub fn terminal_start_session(
    ctx: &TerminalBridgeCtx<'_>,
    args: TerminalBridgeStartSessionArgs,
) -> RalphResult<()> {
    let (project_path, mcp_config) = resolve_start_session_context(
        ctx.db,
        ctx.codebase_snapshot,
        ctx.mcp_dir,
        ctx.api_server_port,
        ctx.locked_project_path,
        args.mcp_mode.as_deref(),
    )?;

    let config = build_session_config(
        args.agent,
        args.model,
        args.effort,
        args.thinking,
        args.permission_level,
        args.post_start_preamble,
    )?;

    ctx.pty_manager.create_session(
        Arc::clone(&ctx.sink),
        args.session_id,
        &project_path,
        mcp_config,
        config,
    )
}

pub fn terminal_start_task_session(
    ctx: &TerminalBridgeCtx<'_>,
    args: TerminalBridgeStartTaskSessionArgs,
) -> RalphResult<()> {
    let TerminalBridgeStartTaskSessionArgs {
        session_id,
        task_id,
        agent,
        model,
        effort,
        permission_level,
        thinking,
        post_start_preamble,
    } = args;

    let (project_path, mcp_config) = resolve_start_task_session_context(
        ctx.db,
        ctx.codebase_snapshot,
        ctx.mcp_dir,
        ctx.api_server_port,
        ctx.locked_project_path,
        task_id,
    )?;

    let resolved = with_db(ctx.db, |db| {
        resolve_task_launch_config(
            db,
            task_id,
            TerminalBridgeLaunchDefaults {
                agent,
                model,
                effort,
                thinking,
                permission_level,
            },
        )
    })?;

    let config = build_session_config(
        resolved.agent,
        resolved.model,
        resolved.effort,
        resolved.thinking,
        resolved.permission_level,
        post_start_preamble,
    )?;

    ctx.pty_manager.create_session(
        Arc::clone(&ctx.sink),
        session_id,
        &project_path,
        Some(mcp_config),
        config,
    )
}

pub fn terminal_send_input(
    pty_manager: &PTYManager,
    args: TerminalBridgeSendInputArgs,
) -> RalphResult<()> {
    pty_manager.send_input(&args.session_id, &args.data)
}

pub fn terminal_resize(
    pty_manager: &PTYManager,
    args: TerminalBridgeResizeArgs,
) -> RalphResult<()> {
    pty_manager.resize(&args.session_id, args.cols, args.rows)
}

pub fn terminal_terminate(
    pty_manager: &PTYManager,
    args: TerminalBridgeTerminateArgs,
) -> RalphResult<()> {
    pty_manager.terminate(&args.session_id)
}

pub fn terminal_set_stream_mode(
    pty_manager: &PTYManager,
    args: TerminalBridgeSetStreamModeArgs,
) -> RalphResult<()> {
    let mode = SessionStreamMode::parse(&args.mode)?;
    pty_manager.set_stream_mode(&args.session_id, mode)
}

pub fn terminal_replay_output(
    pty_manager: &PTYManager,
    args: TerminalBridgeReplayOutputArgs,
) -> RalphResult<TerminalBridgeReplayOutputResult> {
    pty_manager.replay_output(&args.session_id, args.after_seq, args.limit as usize)
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

pub fn terminal_start_human_session(
    ctx: &TerminalBridgeCtx<'_>,
    args: TerminalBridgeStartHumanSessionArgs,
) -> RalphResult<TerminalBridgeStartHumanSessionResult> {
    let resolved = match args.task_id {
        Some(task_id) => with_db(ctx.db, |db| {
            resolve_task_launch_config(
                db,
                task_id,
                TerminalBridgeLaunchDefaults {
                    agent: args.agent.clone(),
                    model: args.model.clone(),
                    effort: args.effort.clone(),
                    thinking: args.thinking,
                    permission_level: args.permission_level.clone(),
                },
            )
        })?,
        None => TerminalBridgeResolvedLaunchConfig {
            agent: args.agent.clone(),
            model: args.model.clone(),
            effort: args.effort.clone(),
            thinking: args.thinking,
            permission_level: args.permission_level.clone(),
            agent_source: crate::terminal::TerminalBridgeLaunchSource::Default,
            model_source: crate::terminal::TerminalBridgeLaunchSource::Default,
            effort_source: crate::terminal::TerminalBridgeLaunchSource::Default,
            thinking_source: crate::terminal::TerminalBridgeLaunchSource::Default,
            permission_level_source: crate::terminal::TerminalBridgeLaunchSource::Default,
            model_supports_effort: false,
        },
    };

    let session_config = build_session_config(
        resolved.agent.clone(),
        resolved.model.clone(),
        resolved.effort.clone(),
        resolved.thinking,
        resolved.permission_level.clone(),
        args.post_start_preamble.clone(),
    )?;
    let launch_command = build_launch_command(&session_config);
    let resolved_post_start_preamble = session_config.post_start_preamble;

    let agent_session_id = generate_agent_session_id();

    let agent_session_number = with_db_tx(ctx.db, |db| {
        db.create_human_agent_session(sqlite_db::AgentSessionCreateInput {
            id: agent_session_id.clone(),
            kind: args.kind.clone(),
            task_id: args.task_id,
            agent: resolved.agent.clone(),
            model: resolved.model.clone(),
            launch_command: Some(launch_command),
            post_start_preamble: resolved_post_start_preamble,
            init_prompt: args.init_prompt.clone(),
        })?;

        let session = db
            .get_agent_session_by_id(&agent_session_id)?
            .ok_or_else(|| {
                err_string(
                    codes::INTERNAL,
                    format!("Failed to load newly created agent session '{agent_session_id}'"),
                )
            })?;
        Ok(session.session_number)
    })?;

    let start_result = if let Some(task_id) = args.task_id {
        terminal_start_task_session(
            ctx,
            TerminalBridgeStartTaskSessionArgs {
                session_id: args.terminal_session_id.clone(),
                task_id,
                agent: resolved.agent.clone(),
                model: resolved.model.clone(),
                effort: resolved.effort.clone(),
                permission_level: resolved.permission_level.clone(),
                thinking: resolved.thinking,
                post_start_preamble: args.post_start_preamble.clone(),
            },
        )
    } else {
        terminal_start_session(
            ctx,
            TerminalBridgeStartSessionArgs {
                session_id: args.terminal_session_id.clone(),
                agent: resolved.agent.clone(),
                mcp_mode: args.mcp_mode.clone(),
                model: resolved.model.clone(),
                effort: resolved.effort.clone(),
                permission_level: resolved.permission_level.clone(),
                thinking: resolved.thinking,
                post_start_preamble: args.post_start_preamble.clone(),
            },
        )
    };

    if let Err(err) = start_result {
        if let Err(update_err) = crate::session::with_db(ctx.db, |db| {
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
                error_text: Some(err.to_string()),
            })
        }) {
            tracing::warn!(
                error = %update_err,
                agent_session_id = %agent_session_id,
                "Failed to persist agent session crash details"
            );
        }
        return Err(err);
    }

    let connected_line =
        format!("\u{1b}[2m[connected to agent_session #{agent_session_number:03}]\u{1b}[0m\r\n");
    emit_system_message(&*ctx.sink, args.terminal_session_id, connected_line)?;

    Ok(TerminalBridgeStartHumanSessionResult {
        agent_session_id,
        agent_session_number,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use ralph_contracts::events::BackendDiagnosticEvent;
    use ralph_contracts::terminal::{PtyClosedEvent, PtyOutputEvent};
    use std::sync::Arc;
    use tempfile::tempdir;

    #[derive(Default)]
    struct CaptureSink {
        outputs: Mutex<Vec<PtyOutputEvent>>,
    }

    impl CaptureSink {
        fn last_output(&self) -> Option<PtyOutputEvent> {
            self.outputs.lock().ok().and_then(|g| g.last().cloned())
        }
    }

    impl EventSink for CaptureSink {
        fn emit_backend_diagnostic(&self, _payload: BackendDiagnosticEvent) -> Result<(), String> {
            Ok(())
        }

        fn emit_terminal_output(&self, payload: PtyOutputEvent) -> Result<(), String> {
            let mut guard = self
                .outputs
                .lock()
                .map_err(|_| "poisoned CaptureSink mutex".to_owned())?;
            guard.push(payload);
            Ok(())
        }

        fn emit_terminal_closed(&self, _payload: PtyClosedEvent) -> Result<(), String> {
            Ok(())
        }
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
    fn emit_system_message_emits_terminal_output_event() {
        let sink = CaptureSink::default();
        emit_system_message(
            &sink,
            "session-emission".to_owned(),
            "[session started]\r\n".to_owned(),
        )
        .unwrap();

        let payload = sink.last_output().expect("expected terminal output");
        assert_eq!(payload.session_id, "session-emission");
        assert_eq!(payload.seq, 0);
        let decoded = STANDARD.decode(payload.data).unwrap();
        assert_eq!(String::from_utf8(decoded).unwrap(), "[session started]\r\n");
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
    fn send_input_returns_missing_session_error() {
        let pty = PTYManager::new();
        let err = terminal_send_input(
            &pty,
            TerminalBridgeSendInputArgs {
                session_id: "missing-session".to_owned(),
                data: b"echo hi\n".to_vec(),
            },
        )
        .unwrap_err();
        assert!(err
            .to_string()
            .contains("No terminal bridge session: missing-session"));
    }

    #[test]
    fn resize_returns_missing_session_error() {
        let pty = PTYManager::new();
        let err = terminal_resize(
            &pty,
            TerminalBridgeResizeArgs {
                session_id: "missing-session".to_owned(),
                cols: 120,
                rows: 40,
            },
        )
        .unwrap_err();
        assert!(err
            .to_string()
            .contains("No terminal bridge session: missing-session"));
    }

    #[test]
    fn terminate_is_noop_for_missing_session() {
        let pty = PTYManager::new();
        assert!(terminal_terminate(
            &pty,
            TerminalBridgeTerminateArgs {
                session_id: "missing-session".to_owned(),
            },
        )
        .is_ok());
    }

    #[test]
    fn set_stream_mode_rejects_invalid_mode() {
        let pty = PTYManager::new();
        let err = terminal_set_stream_mode(
            &pty,
            TerminalBridgeSetStreamModeArgs {
                session_id: "missing-session".to_owned(),
                mode: "invalid".to_owned(),
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("Invalid stream mode"));
    }

    #[test]
    fn replay_output_returns_missing_session_error() {
        let pty = PTYManager::new();
        let err = terminal_replay_output(
            &pty,
            TerminalBridgeReplayOutputArgs {
                session_id: "missing-session".to_owned(),
                after_seq: 0,
                limit: 64,
            },
        )
        .unwrap_err();
        assert!(err
            .to_string()
            .contains("No terminal bridge session: missing-session"));
    }

    #[test]
    fn resolve_start_session_context_allows_no_mcp_config_without_db() {
        let project_dir = tempdir().unwrap();
        let mcp_dir = tempdir().unwrap();

        let db = Mutex::new(None);
        let snapshot = Mutex::new(None);

        let (project_path, mcp_config) = resolve_start_session_context(
            &db,
            &snapshot,
            mcp_dir.path(),
            None,
            project_dir.path(),
            None,
        )
        .unwrap();

        assert_eq!(project_path, project_dir.path().to_path_buf());
        assert!(mcp_config.is_none());
    }

    #[test]
    fn resolve_start_session_context_requires_open_db_when_generating_mcp() {
        let project_dir = tempdir().unwrap();
        let mcp_dir = tempdir().unwrap();

        let db = Mutex::new(None);
        let snapshot = Mutex::new(None);

        let err = resolve_start_session_context(
            &db,
            &snapshot,
            mcp_dir.path(),
            None,
            project_dir.path(),
            Some("interactive"),
        )
        .unwrap_err();
        assert!(err.to_string().contains("database not open"));
    }

    #[test]
    fn resolve_start_task_session_context_requires_open_db() {
        let project_dir = tempdir().unwrap();
        let mcp_dir = tempdir().unwrap();

        let db = Mutex::new(None);
        let snapshot = Mutex::new(None);

        let err = resolve_start_task_session_context(
            &db,
            &snapshot,
            mcp_dir.path(),
            None,
            project_dir.path(),
            42,
        )
        .unwrap_err();
        assert!(err.to_string().contains("database not open"));
    }

    #[test]
    fn terminal_list_model_form_tree_includes_codex_and_claude() {
        let tree = terminal_list_model_form_tree();
        let agents: Vec<String> = tree.providers.into_iter().map(|p| p.agent).collect();
        assert!(agents.contains(&AGENT_CODEX.to_owned()));
        assert!(agents.contains(&AGENT_CLAUDE.to_owned()));
    }

    #[test]
    fn terminal_start_session_wires_sink_and_fails_loud_when_project_unavailable() {
        // Sanity: ensures the function requires a valid path and doesn't silently succeed.
        let pty = PTYManager::new();
        let sink: Arc<dyn EventSink> = Arc::new(CaptureSink::default());
        let db = Mutex::new(None);
        let snapshot = Mutex::new(None);
        let mcp_dir = tempdir().unwrap();
        let ctx = TerminalBridgeCtx {
            pty_manager: &pty,
            sink,
            locked_project_path: std::path::Path::new("/does/not/exist"),
            db: &db,
            codebase_snapshot: &snapshot,
            mcp_dir: mcp_dir.path(),
            api_server_port: None,
        };

        let err = terminal_start_session(
            &ctx,
            TerminalBridgeStartSessionArgs {
                session_id: "s".to_owned(),
                agent: Some(AGENT_CODEX.to_owned()),
                mcp_mode: None,
                model: None,
                effort: None,
                permission_level: None,
                thinking: None,
                post_start_preamble: None,
            },
        )
        .unwrap_err();

        assert!(!err.to_string().trim().is_empty());
    }
}
