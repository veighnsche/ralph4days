use core_contracts::transport::RemoteWireFrame;
use core_errors::{codes, err_string, RalphError, RalphResult};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::tungstenite::Message;

use crate::state::AppState;

pub async fn handle_connection(
    stream: tokio::net::TcpStream,
    state: Arc<AppState>,
) -> RalphResult<()> {
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

    let mut protocol_error: Option<RalphError> = None;
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
                            let response = match crate::commands::handle_command(
                                state.as_ref(),
                                &command,
                                payload,
                            )
                            .await
                            {
                                Ok(result) => RemoteWireFrame::RpcOk { id, result },
                                Err(error) => RemoteWireFrame::RpcErr { id, error },
                            };
                            let text = serde_json::to_string(&response).unwrap_or_else(|e| {
                                serde_json::to_string(&RemoteWireFrame::RpcErr {
                                    id,
                                    error: core_errors::err_string(
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
