use futures_util::{SinkExt, StreamExt};
use ralph_contracts::protocol::ProtocolVersionInfo;
use ralph_contracts::transport::RemoteWireFrame;
use ralph_errors::{codes, err_string};
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

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    init_tracing();

    let bind = parse_bind_addr().map_err(|e| {
        eprintln!("{e}");
        std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
    })?;

    let listener = TcpListener::bind(bind).await?;
    tracing::info!(%bind, "ralphd listening");

    loop {
        let (stream, peer) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(error) = handle_connection(stream).await {
                tracing::warn!(%peer, error = %error, "ralphd connection closed with error");
            }
        });
    }
}

async fn handle_connection(stream: tokio::net::TcpStream) -> Result<(), String> {
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
                        tokio::spawn(async move {
                            let response = match handle_command(&command, payload) {
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

fn handle_command(command: &str, payload: serde_json::Value) -> Result<serde_json::Value, String> {
    match command {
        "protocol_version_get" => {
            if !payload.is_null() {
                return Err(err_string(
                    codes::INTERNAL,
                    format!("protocol_version_get expects null payload, got: {payload}"),
                ));
            }
            serde_json::to_value(ProtocolVersionInfo::current()).map_err(|e| {
                err_string(
                    codes::INTERNAL,
                    format!("Failed to encode protocol_version_get result: {e}"),
                )
            })
        }
        other => Err(err_string(
            codes::INTERNAL,
            format!("Unknown command: {other}"),
        )),
    }
}
