use futures_util::{SinkExt, StreamExt};
use prompt_builder::CodebaseSnapshot;
use ralph_backend::disciplines_contract::{
    DisciplinesCreateArgs, DisciplinesCroppedImageGetArgs, DisciplinesDeleteArgs,
    DisciplinesImageDataGetArgs, DisciplinesUpdateArgs,
};
use ralph_backend::project_contract::ProjectScanArgs;
use ralph_backend::project_scan;
use ralph_backend::prompt_builder_configs_contract::{
    PromptBuilderConfigDeleteArgs, PromptBuilderConfigGetArgs, PromptBuilderConfigSaveArgs,
};
use ralph_backend::prompt_builder_preview::{PromptBuilderPreviewArgs, PromptBuilderPreviewDeps};
use ralph_backend::subsystems_contract::{
    SubsystemsCommentAddArgs, SubsystemsCommentDeleteArgs, SubsystemsCommentUpdateArgs,
    SubsystemsCreateArgs, SubsystemsDeleteArgs, SubsystemsUpdateArgs,
};
use ralph_backend::terminal::{
    PTYManager, TerminalBridgeListModelFormTreeResult, TerminalBridgeReplayOutputArgs,
    TerminalBridgeReplayOutputResult, TerminalBridgeResizeArgs,
    TerminalBridgeResolveTaskLaunchArgs, TerminalBridgeResolvedLaunchConfig,
    TerminalBridgeSendInputArgs, TerminalBridgeSetStreamModeArgs,
    TerminalBridgeStartHumanSessionArgs, TerminalBridgeStartHumanSessionResult,
    TerminalBridgeStartSessionArgs, TerminalBridgeStartTaskSessionArgs,
    TerminalBridgeTerminateArgs,
};
use ralph_backend::{
    disciplines_service, prompt_builder_configs_service, subsystems_service, terminal_bridge,
};
use ralph_contracts::protocol::ProtocolVersionInfo;
use ralph_contracts::transport::{EventSink, RemoteWireFrame};
use ralph_errors::{codes, err_string, ToStringErr};
use serde::de::DeserializeOwned;
use sqlite_db::SqliteDb;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::tungstenite::Message;

mod event_sink;

fn init_tracing() {
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("ralphd=info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .init();
}

fn parse_bind_addr() -> Result<std::net::SocketAddr, String> {
    let mut args = std::env::args().skip(1);
    let mut bind = "127.0.0.1:9944".to_owned();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bind" => {
                bind = args.next().ok_or_else(|| {
                    err_string(
                        codes::INTERNAL,
                        "Missing value for --bind (expected HOST:PORT)",
                    )
                })?;
            }
            "--help" | "-h" => {
                eprintln!("Usage: ralphd [--bind HOST:PORT]");
                std::process::exit(0);
            }
            other => {
                return Err(err_string(
                    codes::INTERNAL,
                    format!("Unknown CLI argument: {other}"),
                ));
            }
        }
    }

    bind.parse().map_err(|e| {
        err_string(
            codes::INTERNAL,
            format!("Invalid --bind value '{bind}': {e}"),
        )
    })
}

fn ralph4days_data_dir() -> Result<PathBuf, String> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("share"))
        })
        .ok_or_else(|| {
            err_string(
                codes::FILESYSTEM,
                "No XDG data directory (missing HOME/XDG_DATA_HOME)",
            )
        })?;
    Ok(base.join("ralph4days"))
}

struct RalphdState {
    locked_project: Mutex<Option<PathBuf>>,
    db: Mutex<Option<SqliteDb>>,
    codebase_snapshot: Mutex<Option<CodebaseSnapshot>>,
    pty_manager: PTYManager,
    mcp_dir: PathBuf,
    api_server_port: Mutex<Option<u16>>,
    event_tx: broadcast::Sender<String>,
    event_sink: Arc<dyn EventSink>,
}

impl RalphdState {
    fn new() -> Self {
        let (event_tx, _) = broadcast::channel::<String>(1024);
        let sink: Arc<dyn EventSink> = Arc::new(event_sink::RalphdEventSink::new(event_tx.clone()));
        Self {
            locked_project: Mutex::new(None),
            db: Mutex::new(None),
            codebase_snapshot: Mutex::new(None),
            pty_manager: PTYManager::new(),
            mcp_dir: std::env::temp_dir().join(format!("ralphd-mcp-{}", std::process::id())),
            api_server_port: Mutex::new(None),
            event_tx,
            event_sink: sink,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    init_tracing();

    let bind = parse_bind_addr().map_err(|e| {
        eprintln!("{e}");
        std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
    })?;

    let listener = TcpListener::bind(bind).await?;
    let local = listener.local_addr()?;
    tracing::info!(%local, "ralphd listening");
    if std::env::var_os("RALPHD_PRINT_LISTEN_ADDR").is_some() {
        println!("RALPHD_LISTEN_ADDR={local}");
    }

    let state = Arc::new(RalphdState::new());
    ralph_backend::diagnostics::register_sink(Arc::clone(&state.event_sink));

    loop {
        let (stream, peer) = listener.accept().await?;
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            if let Err(error) = handle_connection(stream, state).await {
                tracing::warn!(%peer, error = %error, "ralphd connection closed with error");
            }
        });
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    state: Arc<RalphdState>,
) -> Result<(), String> {
    let ws = tokio_tungstenite::accept_async(stream)
        .await
        .map_err(|e| err_string(codes::INTERNAL, format!("WS accept failed: {e}")))?;

    let (mut write, mut read) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    let writer = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if write.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut events = state.event_tx.subscribe();
    let events_tx = tx.clone();
    let events_forwarder = tokio::spawn(async move {
        loop {
            match events.recv().await {
                Ok(text) => {
                    if events_tx.send(Message::Text(text.into())).is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    tracing::warn!(
                        skipped,
                        "WS client lagged on event stream; closing connection"
                    );
                    let _ = events_tx.send(Message::Close(None));
                    break;
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    let mut protocol_error: Option<String> = None;
    while let Some(item) = read.next().await {
        let msg = item.map_err(|e| err_string(codes::INTERNAL, format!("WS read failed: {e}")))?;

        match msg {
            Message::Text(text) => {
                let frame: RemoteWireFrame = serde_json::from_str(&text).map_err(|e| {
                    err_string(
                        codes::INTERNAL,
                        format!("Remote wire frame decode error: {e}"),
                    )
                })?;

                match frame {
                    RemoteWireFrame::RpcRequest {
                        id,
                        command,
                        payload,
                    } => {
                        let tx_clone = tx.clone();
                        let state = Arc::clone(&state);
                        tokio::spawn(async move {
                            let response = match handle_command(&state, &command, payload).await {
                                Ok(result) => RemoteWireFrame::RpcOk { id, result },
                                Err(error) => {
                                    // Contract: all errors should be coded. If something violates that invariant,
                                    // surface it loudly as an INTERNAL error (instead of silently passing raw strings).
                                    let structured = ralph_errors::parse_ralph_error(&error)
                                        .unwrap_or_else(|| {
                                            ralph_errors::RalphError::new(
                                                codes::INTERNAL,
                                                format!(
                                                    "ralphd invariant violated: uncoded rpc error: {error}"
                                                ),
                                            )
                                        });
                                    RemoteWireFrame::RpcErr {
                                        id,
                                        error: structured,
                                    }
                                }
                            };
                            let text = serde_json::to_string(&response).unwrap_or_else(|e| {
                                serde_json::to_string(&RemoteWireFrame::RpcErr {
                                    id,
                                    error: ralph_errors::RalphError::new(
                                        codes::INTERNAL,
                                        format!("Failed to encode rpc response: {e}"),
                                    ),
                                })
                                .expect("rpc-err frame must serialize")
                            });
                            let _ = tx_clone.send(Message::Text(text.into()));
                        });
                    }
                    RemoteWireFrame::Event { .. }
                    | RemoteWireFrame::RpcOk { .. }
                    | RemoteWireFrame::RpcErr { .. } => {
                        protocol_error = Some(err_string(
                            codes::INTERNAL,
                            "Remote protocol error: client sent a non-request frame",
                        ));
                        break;
                    }
                }
            }
            Message::Ping(payload) => {
                let _ = tx.send(Message::Pong(payload));
            }
            Message::Pong(_) | Message::Frame(_) => {}
            Message::Close(_) => break,
            Message::Binary(_) => {
                protocol_error = Some(err_string(
                    codes::INTERNAL,
                    "Remote protocol error: binary WS frames are not supported",
                ));
                break;
            }
        }
    }

    writer.abort();
    events_forwarder.abort();
    let _ = writer.await;
    let _ = events_forwarder.await;

    if let Some(error) = protocol_error {
        Err(error)
    } else {
        Ok(())
    }
}

fn require_null_payload(command: &str, payload: serde_json::Value) -> Result<(), String> {
    if payload.is_null() {
        Ok(())
    } else {
        Err(err_string(
            codes::INTERNAL,
            format!("{command} expects null payload, got: {payload}"),
        ))
    }
}

fn decode_args<TArgs: DeserializeOwned>(
    command: &str,
    payload: serde_json::Value,
) -> Result<TArgs, String> {
    let serde_json::Value::Object(mut map) = payload else {
        return Err(err_string(
            codes::INTERNAL,
            format!("{command} expects payload {{ args: ... }}, got: {payload}"),
        ));
    };

    let args_value = map.remove("args").ok_or_else(|| {
        err_string(
            codes::INTERNAL,
            format!("{command} expects payload {{ args: ... }} (missing 'args' key)"),
        )
    })?;
    if !map.is_empty() {
        let keys = map.keys().cloned().collect::<Vec<_>>().join(", ");
        return Err(err_string(
            codes::INTERNAL,
            format!("{command} payload has unexpected keys: {keys}"),
        ));
    }

    serde_json::from_value(args_value).map_err(|e| {
        err_string(
            codes::INTERNAL,
            format!("{command} args decode failed: {e}"),
        )
    })
}

fn encode_result<T: serde::Serialize>(
    command: &str,
    value: T,
) -> Result<serde_json::Value, String> {
    serde_json::to_value(value).map_err(|e| {
        err_string(
            codes::INTERNAL,
            format!("Failed to encode '{command}' result: {e}"),
        )
    })
}

async fn handle_command(
    state: &RalphdState,
    command: &str,
    payload: serde_json::Value,
) -> Result<serde_json::Value, String> {
    match command {
        "protocol_version_get" => {
            require_null_payload(command, payload)?;
            encode_result(command, ProtocolVersionInfo::current())
        }
        "project_validate_path" => {
            let args: ralph_backend::project::ProjectValidatePathArgs =
                decode_args(command, payload)?;
            let path = PathBuf::from(args.path);
            ralph_backend::project::validate_project_path(&path)?;
            Ok(serde_json::Value::Null)
        }
        "project_lock_set" => {
            let args: ralph_backend::session::ProjectLockSetArgs = decode_args(command, payload)?;
            let canonical =
                ralph_backend::session::project_lock_set(&state.locked_project, &state.db, args)?;
            let project_name = canonical
                .file_name()
                .map_or_else(|| "Unknown".to_owned(), |n| n.to_string_lossy().to_string());
            if let Err(error) = project_scan::recents_add(
                &ralph4days_data_dir()?,
                canonical.to_string_lossy().to_string(),
                project_name,
            ) {
                tracing::warn!("Failed to persist recent projects: {error}");
            }
            Ok(serde_json::Value::Null)
        }
        "project_lock_get" => {
            require_null_payload(command, payload)?;
            let locked = ralph_backend::session::project_lock_get(&state.locked_project)?;
            encode_result(command, locked)
        }
        "project_initialize" => {
            let args: ralph_backend::project::ProjectInitializeArgs =
                decode_args(command, payload)?;
            ralph_backend::project::project_initialize(args)?;
            Ok(serde_json::Value::Null)
        }
        "project_recent_list" => {
            require_null_payload(command, payload)?;
            let projects = project_scan::recents_load(&ralph4days_data_dir()?)?;
            encode_result(command, projects)
        }
        "project_scan" => {
            let args: ProjectScanArgs = decode_args(command, payload)?;
            let projects = project_scan::project_scan(args)?;
            encode_result(command, projects)
        }
        "project_info_get" => {
            require_null_payload(command, payload)?;
            let info = ralph_backend::session::with_db(&state.db, |db| {
                project_scan::project_info_get(db)
            })?;
            encode_result(command, info)
        }
        "subsystems_list" => {
            require_null_payload(command, payload)?;
            let subsystems = ralph_backend::session::with_db(&state.db, |db| {
                subsystems_service::subsystems_list(db)
            })?;
            encode_result(command, subsystems)
        }
        "subsystems_create" => {
            let args: SubsystemsCreateArgs = decode_args(command, payload)?;
            let created = ralph_backend::session::with_db(&state.db, |db| {
                subsystems_service::subsystems_create(db, args)
            })?;
            encode_result(command, created)
        }
        "subsystems_update" => {
            let args: SubsystemsUpdateArgs = decode_args(command, payload)?;
            let updated = ralph_backend::session::with_db(&state.db, |db| {
                subsystems_service::subsystems_update(db, args)
            })?;
            encode_result(command, updated)
        }
        "subsystems_delete" => {
            let args: SubsystemsDeleteArgs = decode_args(command, payload)?;
            ralph_backend::session::with_db(&state.db, |db| {
                subsystems_service::subsystems_delete(db, args)
            })?;
            Ok(serde_json::Value::Null)
        }
        "subsystems_comment_add" => {
            let args: SubsystemsCommentAddArgs = decode_args(command, payload)?;
            let project_path = ralph_backend::session::locked_project_path(&state.locked_project)?;

            let (subsystem, embed_work) = ralph_backend::session::with_db(&state.db, |db| {
                subsystems_service::subsystems_comment_add_prepare(db, args)
            })?;
            subsystems_service::subsystems_comment_apply_embedding(&project_path, embed_work)
                .await?;

            encode_result(command, subsystem)
        }
        "subsystems_comment_update" => {
            let args: SubsystemsCommentUpdateArgs = decode_args(command, payload)?;
            let project_path = ralph_backend::session::locked_project_path(&state.locked_project)?;

            let (subsystem, embed_work) = ralph_backend::session::with_db(&state.db, |db| {
                subsystems_service::subsystems_comment_update_prepare(db, args)
            })?;

            if let Some(work) = embed_work {
                subsystems_service::subsystems_comment_apply_embedding(&project_path, work).await?;
            }

            encode_result(command, subsystem)
        }
        "subsystems_comment_delete" => {
            let args: SubsystemsCommentDeleteArgs = decode_args(command, payload)?;
            let updated = ralph_backend::session::with_db(&state.db, |db| {
                subsystems_service::subsystems_comment_delete(db, args)
            })?;
            encode_result(command, updated)
        }
        "disciplines_list" => {
            require_null_payload(command, payload)?;
            let disciplines = ralph_backend::session::with_db(&state.db, |db| {
                disciplines_service::disciplines_list(db)
            })?;
            encode_result(command, disciplines)
        }
        "disciplines_create" => {
            let args: DisciplinesCreateArgs = decode_args(command, payload)?;
            let created = ralph_backend::session::with_db(&state.db, |db| {
                disciplines_service::disciplines_create(db, args)
            })?;
            encode_result(command, created)
        }
        "disciplines_update" => {
            let args: DisciplinesUpdateArgs = decode_args(command, payload)?;
            let updated = ralph_backend::session::with_db(&state.db, |db| {
                disciplines_service::disciplines_update(db, args)
            })?;
            encode_result(command, updated)
        }
        "disciplines_delete" => {
            let args: DisciplinesDeleteArgs = decode_args(command, payload)?;
            let deleted = ralph_backend::session::with_db(&state.db, |db| {
                disciplines_service::disciplines_delete(db, args)
            })?;
            encode_result(command, deleted)
        }
        "disciplines_image_data_get" => {
            let args: DisciplinesImageDataGetArgs = decode_args(command, payload)?;
            let project_path = ralph_backend::session::locked_project_path(&state.locked_project)?;
            let result = ralph_backend::session::with_db(&state.db, |db| {
                disciplines_service::disciplines_image_data_get(&project_path, db, args)
            })?;
            encode_result(command, result)
        }
        "disciplines_cropped_image_get" => {
            let args: DisciplinesCroppedImageGetArgs = decode_args(command, payload)?;
            let project_path = ralph_backend::session::locked_project_path(&state.locked_project)?;
            let result = ralph_backend::session::with_db(&state.db, |db| {
                disciplines_service::disciplines_cropped_image_get(&project_path, db, args)
            })?;
            encode_result(command, result)
        }
        "prompt_builder_config_list" => {
            require_null_payload(command, payload)?;
            let names = ralph_backend::session::with_db(&state.db, |db| {
                prompt_builder_configs_service::prompt_builder_config_list(db)
            })?;
            encode_result(command, names)
        }
        "prompt_builder_config_get" => {
            let args: PromptBuilderConfigGetArgs = decode_args(command, payload)?;
            let config = ralph_backend::session::with_db(&state.db, |db| {
                prompt_builder_configs_service::prompt_builder_config_get(db, args)
            })?;
            encode_result(command, config)
        }
        "prompt_builder_config_save" => {
            let args: PromptBuilderConfigSaveArgs = decode_args(command, payload)?;
            ralph_backend::session::with_db(&state.db, |db| {
                prompt_builder_configs_service::prompt_builder_config_save(db, args)
            })?;
            Ok(serde_json::Value::Null)
        }
        "prompt_builder_config_delete" => {
            let args: PromptBuilderConfigDeleteArgs = decode_args(command, payload)?;
            ralph_backend::session::with_db(&state.db, |db| {
                prompt_builder_configs_service::prompt_builder_config_delete(db, args)
            })?;
            Ok(serde_json::Value::Null)
        }
        "prompt_builder_preview" => {
            let args: PromptBuilderPreviewArgs = decode_args(command, payload)?;
            let project_path = ralph_backend::session::locked_project_path(&state.locked_project)?;
            let api_server_port = *state.api_server_port.lock().err_str(codes::INTERNAL)?;
            let preview = ralph_backend::session::with_db(&state.db, |db| {
                ralph_backend::prompt_builder_preview::prompt_builder_preview(
                    PromptBuilderPreviewDeps {
                        db,
                        project_path: project_path.as_path(),
                        mcp_dir: state.mcp_dir.as_path(),
                        codebase_snapshot: &state.codebase_snapshot,
                        api_server_port,
                    },
                    args,
                )
            })?;
            encode_result(command, preview)
        }
        "terminal_start_session" => {
            let args: TerminalBridgeStartSessionArgs = decode_args(command, payload)?;
            let project_path = ralph_backend::session::locked_project_path(&state.locked_project)?;
            let api_server_port = *state.api_server_port.lock().err_str(codes::INTERNAL)?;
            let ctx = terminal_bridge::TerminalBridgeCtx {
                pty_manager: &state.pty_manager,
                sink: Arc::clone(&state.event_sink),
                locked_project_path: project_path.as_path(),
                db: &state.db,
                codebase_snapshot: &state.codebase_snapshot,
                mcp_dir: state.mcp_dir.as_path(),
                api_server_port,
            };
            terminal_bridge::terminal_start_session(&ctx, args)?;
            Ok(serde_json::Value::Null)
        }
        "terminal_start_task_session" => {
            let args: TerminalBridgeStartTaskSessionArgs = decode_args(command, payload)?;
            let project_path = ralph_backend::session::locked_project_path(&state.locked_project)?;
            let api_server_port = *state.api_server_port.lock().err_str(codes::INTERNAL)?;
            let ctx = terminal_bridge::TerminalBridgeCtx {
                pty_manager: &state.pty_manager,
                sink: Arc::clone(&state.event_sink),
                locked_project_path: project_path.as_path(),
                db: &state.db,
                codebase_snapshot: &state.codebase_snapshot,
                mcp_dir: state.mcp_dir.as_path(),
                api_server_port,
            };
            terminal_bridge::terminal_start_task_session(&ctx, args)?;
            Ok(serde_json::Value::Null)
        }
        "terminal_resolve_task_launch_config" => {
            let args: TerminalBridgeResolveTaskLaunchArgs = decode_args(command, payload)?;
            let resolved: TerminalBridgeResolvedLaunchConfig =
                ralph_backend::session::with_db(&state.db, |db| {
                    ralph_backend::terminal::resolve_task_launch_config(
                        db,
                        args.task_id,
                        args.defaults,
                    )
                })?;
            encode_result(command, resolved)
        }
        "terminal_start_human_session" => {
            let args: TerminalBridgeStartHumanSessionArgs = decode_args(command, payload)?;
            let project_path = ralph_backend::session::locked_project_path(&state.locked_project)?;
            let api_server_port = *state.api_server_port.lock().err_str(codes::INTERNAL)?;
            let ctx = terminal_bridge::TerminalBridgeCtx {
                pty_manager: &state.pty_manager,
                sink: Arc::clone(&state.event_sink),
                locked_project_path: project_path.as_path(),
                db: &state.db,
                codebase_snapshot: &state.codebase_snapshot,
                mcp_dir: state.mcp_dir.as_path(),
                api_server_port,
            };
            let result: TerminalBridgeStartHumanSessionResult =
                terminal_bridge::terminal_start_human_session(&ctx, args)?;
            encode_result(command, result)
        }
        "terminal_list_model_form_tree" => {
            require_null_payload(command, payload)?;
            let tree: TerminalBridgeListModelFormTreeResult =
                terminal_bridge::terminal_list_model_form_tree();
            encode_result(command, tree)
        }
        "terminal_send_input" => {
            let args: TerminalBridgeSendInputArgs = decode_args(command, payload)?;
            terminal_bridge::terminal_send_input(&state.pty_manager, args)?;
            Ok(serde_json::Value::Null)
        }
        "terminal_resize" => {
            let args: TerminalBridgeResizeArgs = decode_args(command, payload)?;
            terminal_bridge::terminal_resize(&state.pty_manager, args)?;
            Ok(serde_json::Value::Null)
        }
        "terminal_set_stream_mode" => {
            let args: TerminalBridgeSetStreamModeArgs = decode_args(command, payload)?;
            terminal_bridge::terminal_set_stream_mode(&state.pty_manager, args)?;
            Ok(serde_json::Value::Null)
        }
        "terminal_replay_output" => {
            let args: TerminalBridgeReplayOutputArgs = decode_args(command, payload)?;
            let result: TerminalBridgeReplayOutputResult =
                terminal_bridge::terminal_replay_output(&state.pty_manager, args)?;
            encode_result(command, result)
        }
        "terminal_terminate" => {
            let args: TerminalBridgeTerminateArgs = decode_args(command, payload)?;
            terminal_bridge::terminal_terminate(&state.pty_manager, args)?;
            Ok(serde_json::Value::Null)
        }
        "tasks_create" => {
            let args: ralph_backend::tasks::TasksCreateArgs = decode_args(command, payload)?;
            let created = ralph_backend::session::with_db(&state.db, |db| {
                ralph_backend::tasks::tasks_create(db, args)
            })?;
            encode_result(command, created)
        }
        "tasks_update" => {
            let args: ralph_backend::tasks::TasksUpdateArgs = decode_args(command, payload)?;
            let updated = ralph_backend::session::with_db(&state.db, |db| {
                ralph_backend::tasks::tasks_update(db, args)
            })?;
            encode_result(command, updated)
        }
        "tasks_set_status" => {
            let args: ralph_backend::tasks::TasksSetStatusArgs = decode_args(command, payload)?;
            let updated = ralph_backend::session::with_db(&state.db, |db| {
                ralph_backend::tasks::tasks_set_status(db, args)
            })?;
            encode_result(command, updated)
        }
        "tasks_delete" => {
            let args: ralph_backend::tasks::TasksDeleteArgs = decode_args(command, payload)?;
            ralph_backend::session::with_db(&state.db, |db| {
                ralph_backend::tasks::tasks_delete(db, args)
            })?;
            Ok(serde_json::Value::Null)
        }
        "tasks_signal_add" => {
            let args: ralph_backend::tasks::TasksSignalAddArgs = decode_args(command, payload)?;
            let updated = ralph_backend::session::with_db(&state.db, |db| {
                ralph_backend::tasks::tasks_signal_add(db, args)
            })?;
            encode_result(command, updated)
        }
        "tasks_signal_update" => {
            let args: ralph_backend::tasks::TasksSignalUpdateArgs = decode_args(command, payload)?;
            let updated = ralph_backend::session::with_db(&state.db, |db| {
                ralph_backend::tasks::tasks_signal_update(db, args)
            })?;
            encode_result(command, updated)
        }
        "tasks_signal_delete" => {
            let args: ralph_backend::tasks::TasksSignalDeleteArgs = decode_args(command, payload)?;
            let updated = ralph_backend::session::with_db(&state.db, |db| {
                ralph_backend::tasks::tasks_signal_delete(db, args)
            })?;
            encode_result(command, updated)
        }
        "tasks_list" => {
            require_null_payload(command, payload)?;
            let tasks =
                ralph_backend::session::with_db(&state.db, ralph_backend::tasks::tasks_list)?;
            encode_result(command, tasks)
        }
        "tasks_get" => {
            let args: ralph_backend::tasks::TasksGetArgs = decode_args(command, payload)?;
            let task = ralph_backend::session::with_db(&state.db, |db| {
                ralph_backend::tasks::tasks_get(db, args)
            })?;
            encode_result(command, task)
        }
        "tasks_list_items" => {
            require_null_payload(command, payload)?;
            let items =
                ralph_backend::session::with_db(&state.db, ralph_backend::tasks::tasks_list_items)?;
            encode_result(command, items)
        }
        "tasks_signal_summaries_get" => {
            let args: ralph_backend::tasks::TasksSignalSummariesGetArgs =
                decode_args(command, payload)?;
            let summaries = ralph_backend::session::with_db(&state.db, |db| {
                ralph_backend::tasks::tasks_signal_summaries_get(db, args)
            })?;
            encode_result(command, summaries)
        }
        "tasks_ask_answer" => {
            let args: ralph_backend::tasks::TasksAskAnswerArgs = decode_args(command, payload)?;
            ralph_backend::session::with_db(&state.db, |db| {
                ralph_backend::tasks::tasks_ask_answer(db, args)
            })?;
            Ok(serde_json::Value::Null)
        }
        "tasks_comment_reply_add" => {
            let args: ralph_backend::tasks::TasksCommentReplyAddArgs =
                decode_args(command, payload)?;
            let updated = ralph_backend::session::with_db(&state.db, |db| {
                ralph_backend::tasks::tasks_comment_reply_add(db, args)
            })?;
            encode_result(command, updated)
        }
        "tasks_signal_comment_add" => {
            let args: sqlite_db::TaskSignalCommentCreateInput = decode_args(command, payload)?;
            let id = ralph_backend::session::with_db(&state.db, |db| {
                ralph_backend::tasks::tasks_signal_comment_add(db, args)
            })?;
            encode_result(command, id)
        }
        "tasks_signal_comment_update" => {
            let args: ralph_backend::tasks::TasksSignalCommentUpdateArgs =
                decode_args(command, payload)?;
            ralph_backend::session::with_db(&state.db, |db| {
                ralph_backend::tasks::tasks_signal_comment_update(db, args)
            })?;
            Ok(serde_json::Value::Null)
        }
        "tasks_signal_comment_delete" => {
            let args: ralph_backend::tasks::TasksSignalCommentDeleteArgs =
                decode_args(command, payload)?;
            ralph_backend::session::with_db(&state.db, |db| {
                ralph_backend::tasks::tasks_signal_comment_delete(db, args)
            })?;
            Ok(serde_json::Value::Null)
        }
        "tasks_signal_comments_list" => {
            let args: ralph_backend::tasks::TasksSignalCommentsListArgs =
                decode_args(command, payload)?;
            let comments = ralph_backend::session::with_db(&state.db, |db| {
                ralph_backend::tasks::tasks_signal_comments_list(db, args)
            })?;
            encode_result(command, comments)
        }
        other => {
            ralph_backend::diagnostics::emit_warning(
                "ralphd",
                "unknown-command",
                &format!("Unknown command: {other}"),
            );
            Err(err_string(
                codes::INTERNAL,
                format!("Unknown command: {other}"),
            ))
        }
    }
}
