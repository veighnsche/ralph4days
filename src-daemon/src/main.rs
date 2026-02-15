use futures_util::{SinkExt, StreamExt};
use ralph_contracts::protocol::ProtocolVersionInfo;
use ralph_contracts::transport::RemoteWireFrame;
use ralph_errors::{codes, err_string};
use serde::de::DeserializeOwned;
use sqlite_db::SqliteDb;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

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

#[derive(Default)]
struct RalphdState {
    locked_project: Mutex<Option<PathBuf>>,
    db: Mutex<Option<SqliteDb>>,
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    init_tracing();

    let bind = parse_bind_addr().map_err(|e| {
        eprintln!("{e}");
        std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
    })?;

    let listener = TcpListener::bind(bind).await?;
    tracing::info!(%bind, "ralphd listening");

    let state = Arc::new(RalphdState::default());

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
                            let response = match handle_command(&state, &command, payload) {
                                Ok(result) => RemoteWireFrame::RpcOk { id, result },
                                Err(error) => RemoteWireFrame::RpcErr { id, error },
                            };
                            let text = serde_json::to_string(&response).unwrap_or_else(|e| {
                                serde_json::to_string(&RemoteWireFrame::RpcErr {
                                    id,
                                    error: err_string(
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
                        return Err(err_string(
                            codes::INTERNAL,
                            "Remote protocol error: client sent a non-request frame",
                        ));
                    }
                }
            }
            Message::Ping(payload) => {
                let _ = tx.send(Message::Pong(payload));
            }
            Message::Pong(_) | Message::Frame(_) => {}
            Message::Close(_) => break,
            Message::Binary(_) => {
                return Err(err_string(
                    codes::INTERNAL,
                    "Remote protocol error: binary WS frames are not supported",
                ));
            }
        }
    }

    writer.abort();
    let _ = writer.await;
    Ok(())
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

fn handle_command(
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
            let _ =
                ralph_backend::session::project_lock_set(&state.locked_project, &state.db, args)?;
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
        other => Err(err_string(
            codes::INTERNAL,
            format!("Unknown command: {other}"),
        )),
    }
}
