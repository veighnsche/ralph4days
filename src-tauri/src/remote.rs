use futures_util::{SinkExt, StreamExt};
use ralph_contracts::protocol::{ProtocolVersionInfo, PROTOCOL_VERSION};
use ralph_contracts::transport::{BoxFuture, EventSink, RemoteWireFrame, RpcClient};
use ralph_errors::{codes, err_string};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio_tungstenite::tungstenite::Message;

fn remote_err(message: impl Into<String>) -> String {
    // Dedicated transport/remote error codes don't exist yet; use INTERNAL to keep the string
    // machine-parsable and to fail loudly.
    err_string(codes::INTERNAL, message)
}

#[derive(Debug)]
struct RemoteWireInner {
    write_tx: mpsc::UnboundedSender<Message>,
    pending: Mutex<HashMap<u64, oneshot::Sender<Result<serde_json::Value, String>>>>,
    next_id: AtomicU64,
    disconnected: AtomicBool,
}

impl RemoteWireInner {
    async fn fail_all_pending(&self, error: String) {
        let mut pending = self.pending.lock().await;
        for (_, tx) in pending.drain() {
            let _ = tx.send(Err(error.clone()));
        }
    }
}

#[derive(Clone, Debug)]
pub struct RemoteRpcClient {
    inner: Arc<RemoteWireInner>,
}

impl RemoteRpcClient {
    pub fn is_connected(&self) -> bool {
        !self.inner.disconnected.load(Ordering::SeqCst)
    }
}

impl RpcClient for RemoteRpcClient {
    fn invoke(
        &self,
        command: String,
        args: serde_json::Value,
    ) -> BoxFuture<'_, Result<serde_json::Value, String>> {
        let inner = Arc::clone(&self.inner);
        Box::pin(async move {
            if inner.disconnected.load(Ordering::SeqCst) {
                return Err(remote_err("Remote transport is disconnected"));
            }

            let id = inner.next_id.fetch_add(1, Ordering::Relaxed);
            let (tx, rx) = oneshot::channel::<Result<serde_json::Value, String>>();

            let mut pending = inner.pending.lock().await;
            // This should be impossible because ids are monotonic.
            assert!(
                pending.insert(id, tx).is_none(),
                "Duplicate remote RPC id allocated: {id}"
            );
            drop(pending);

            let frame = RemoteWireFrame::RpcRequest {
                id,
                command,
                payload: args,
            };
            let text = serde_json::to_string(&frame)
                .map_err(|e| remote_err(format!("Failed to encode remote RPC request: {e}")))?;

            if inner.write_tx.send(Message::Text(text.into())).is_err() {
                let error = remote_err("Remote transport write channel is closed");
                let mut pending = inner.pending.lock().await;
                pending.remove(&id);
                return Err(error);
            }

            rx.await
                .map_err(|_| remote_err("Remote RPC response channel closed"))?
        })
    }
}

pub struct RemoteWireFrameConnection {
    ws_url: String,
    remote_protocol: ProtocolVersionInfo,
    rpc: RemoteRpcClient,
    shutdown_tx: Option<oneshot::Sender<()>>,
    writer: Option<tokio::task::JoinHandle<()>>,
    reader: Option<tokio::task::JoinHandle<()>>,
}

impl std::fmt::Debug for RemoteWireFrameConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RemoteWireFrameConnection")
            .field("ws_url", &self.ws_url)
            .field("remote_protocol", &self.remote_protocol)
            .finish_non_exhaustive()
    }
}

impl RemoteWireFrameConnection {
    pub fn ws_url(&self) -> &str {
        &self.ws_url
    }

    pub fn remote_protocol(&self) -> ProtocolVersionInfo {
        self.remote_protocol.clone()
    }

    pub fn is_connected(&self) -> bool {
        self.rpc.is_connected()
    }

    pub async fn connect(ws_url: String, sink: Arc<dyn EventSink>) -> Result<Self, String> {
        let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url)
            .await
            .map_err(|e| {
                remote_err(format!(
                    "Failed to connect remote WebSocket '{ws_url}': {e}"
                ))
            })?;

        let (mut write, mut read) = ws_stream.split();
        let (write_tx, mut write_rx) = mpsc::unbounded_channel::<Message>();
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

        let inner = Arc::new(RemoteWireInner {
            write_tx: write_tx.clone(),
            pending: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(1),
            disconnected: AtomicBool::new(false),
        });
        let rpc = RemoteRpcClient {
            inner: Arc::clone(&inner),
        };

        let inner_writer = Arc::clone(&inner);
        let writer = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        let _ = write.send(Message::Close(None)).await;
                        break;
                    }
                    maybe = write_rx.recv() => {
                        let Some(msg) = maybe else { break };
                        if let Err(e) = write.send(msg).await {
                            let error = remote_err(format!("Remote WebSocket send failed: {e}"));
                            inner_writer.disconnected.store(true, Ordering::SeqCst);
                            inner_writer.fail_all_pending(error).await;
                            break;
                        }
                    }
                }
            }

            let _ = write.close().await;
        });

        let inner_reader = Arc::clone(&inner);
        let sink_reader = Arc::clone(&sink);
        let write_tx_reader = write_tx;
        let reader = tokio::spawn(async move {
            while let Some(item) = read.next().await {
                let msg = match item {
                    Ok(msg) => msg,
                    Err(e) => {
                        let error = remote_err(format!("Remote WebSocket read failed: {e}"));
                        inner_reader.disconnected.store(true, Ordering::SeqCst);
                        inner_reader.fail_all_pending(error).await;
                        break;
                    }
                };

                match msg {
                    Message::Text(text) => {
                        let frame: RemoteWireFrame = match serde_json::from_str(&text) {
                            Ok(frame) => frame,
                            Err(e) => {
                                let error =
                                    remote_err(format!("Remote wire frame decode error: {e}"));
                                inner_reader.disconnected.store(true, Ordering::SeqCst);
                                inner_reader.fail_all_pending(error).await;
                                break;
                            }
                        };

                        match frame {
                            RemoteWireFrame::RpcOk { id, result } => {
                                let tx = {
                                    let mut pending = inner_reader.pending.lock().await;
                                    pending.remove(&id)
                                };
                                let Some(tx) = tx else {
                                    let error = remote_err(format!(
                                        "Remote protocol error: rpc-ok for unknown id {id}"
                                    ));
                                    inner_reader.disconnected.store(true, Ordering::SeqCst);
                                    inner_reader.fail_all_pending(error).await;
                                    break;
                                };
                                let _ = tx.send(Ok(result));
                            }
                            RemoteWireFrame::RpcErr { id, error } => {
                                let tx = {
                                    let mut pending = inner_reader.pending.lock().await;
                                    pending.remove(&id)
                                };
                                let Some(tx) = tx else {
                                    let error = remote_err(format!(
                                        "Remote protocol error: rpc-err for unknown id {id}: {error}"
                                    ));
                                    inner_reader.disconnected.store(true, Ordering::SeqCst);
                                    inner_reader.fail_all_pending(error).await;
                                    break;
                                };
                                let _ = tx.send(Err(error));
                            }
                            RemoteWireFrame::Event { frame } => {
                                if let Err(error) = frame.emit_to(sink_reader.as_ref()) {
                                    tracing::warn!(
                                        error = %error,
                                        "Failed to re-emit remote event via sink"
                                    );
                                }
                            }
                            RemoteWireFrame::RpcRequest { id, command, .. } => {
                                let error = remote_err(format!(
                                    "Remote protocol error: unexpected rpc-request from server (id={id}, command={command})"
                                ));
                                inner_reader.disconnected.store(true, Ordering::SeqCst);
                                inner_reader.fail_all_pending(error).await;
                                break;
                            }
                        }
                    }
                    Message::Ping(payload) => {
                        let _ = write_tx_reader.send(Message::Pong(payload));
                    }
                    Message::Pong(_) | Message::Frame(_) => {}
                    Message::Close(_) => break,
                    Message::Binary(_) => {
                        let error =
                            remote_err("Remote protocol error: binary WS frames are not supported");
                        inner_reader.disconnected.store(true, Ordering::SeqCst);
                        inner_reader.fail_all_pending(error).await;
                        break;
                    }
                }
            }

            inner_reader.disconnected.store(true, Ordering::SeqCst);
            inner_reader
                .fail_all_pending(remote_err("Remote WebSocket disconnected"))
                .await;
        });

        let mut conn = Self {
            ws_url,
            remote_protocol: ProtocolVersionInfo {
                protocol_version: 0,
            },
            rpc,
            shutdown_tx: Some(shutdown_tx),
            writer: Some(writer),
            reader: Some(reader),
        };

        let remote_version_value = tokio::time::timeout(
            Duration::from_secs(5),
            conn.rpc
                .invoke("protocol_version_get".to_owned(), serde_json::Value::Null),
        )
        .await
        .map_err(|_| remote_err("Timed out waiting for protocol_version_get response"))??;

        let remote_protocol: ProtocolVersionInfo = serde_json::from_value(remote_version_value)
            .map_err(|e| {
                remote_err(format!("Failed to decode protocol_version_get result: {e}"))
            })?;

        if remote_protocol.protocol_version != PROTOCOL_VERSION {
            let remote = remote_protocol.protocol_version;
            let local = PROTOCOL_VERSION;
            let _ = conn.shutdown().await;
            return Err(remote_err(format!(
                "Protocol mismatch: local protocol_version={local}, remote protocol_version={remote}. Upgrade ralphd or this app to match."
            )));
        }

        conn.remote_protocol = remote_protocol;

        Ok(conn)
    }

    pub async fn shutdown(mut self) -> Result<(), String> {
        self.rpc.inner.disconnected.store(true, Ordering::SeqCst);
        self.rpc
            .inner
            .fail_all_pending(remote_err("Remote connection closed"))
            .await;

        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }

        if let Some(writer) = &self.writer {
            writer.abort();
        }
        if let Some(reader) = &self.reader {
            reader.abort();
        }

        if let Some(writer) = self.writer.take() {
            let _ = writer.await;
        }
        if let Some(reader) = self.reader.take() {
            let _ = reader.await;
        }

        Ok(())
    }
}

impl Drop for RemoteWireFrameConnection {
    fn drop(&mut self) {
        self.rpc.inner.disconnected.store(true, Ordering::SeqCst);
        if let Some(writer) = &self.writer {
            writer.abort();
        }
        if let Some(reader) = &self.reader {
            reader.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::{SinkExt, StreamExt};
    use ralph_contracts::terminal::{PtyOutputEvent, TERMINAL_OUTPUT_EVENT};
    use std::net::SocketAddr;
    use std::sync::Mutex as StdMutex;
    use tokio::net::TcpListener;

    #[derive(Default)]
    struct RecordingSink {
        output: StdMutex<Vec<PtyOutputEvent>>,
    }

    impl EventSink for RecordingSink {
        fn emit_backend_diagnostic(
            &self,
            _payload: ralph_contracts::events::BackendDiagnosticEvent,
        ) -> Result<(), String> {
            Ok(())
        }

        fn emit_terminal_output(&self, payload: PtyOutputEvent) -> Result<(), String> {
            self.output.lock().unwrap().push(payload);
            Ok(())
        }

        fn emit_terminal_closed(
            &self,
            _payload: ralph_contracts::terminal::PtyClosedEvent,
        ) -> Result<(), String> {
            Ok(())
        }
    }

    async fn spawn_test_server(protocol_version: u32, send_event: bool) -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut ws = tokio_tungstenite::accept_async(stream).await.unwrap();

            // Expect the initial protocol_version_get request.
            let msg = ws.next().await.unwrap().unwrap();
            let Message::Text(text) = msg else {
                panic!("Expected text ws frame, got: {msg:?}");
            };
            let frame: RemoteWireFrame = serde_json::from_str(&text).unwrap();
            let RemoteWireFrame::RpcRequest { id, command, .. } = frame else {
                panic!("Expected rpc-request, got: {frame:?}");
            };
            assert_eq!(command, "protocol_version_get");

            let ok = RemoteWireFrame::RpcOk {
                id,
                result: serde_json::to_value(ProtocolVersionInfo { protocol_version }).unwrap(),
            };
            ws.send(Message::Text(serde_json::to_string(&ok).unwrap().into()))
                .await
                .unwrap();

            if send_event {
                let ev = RemoteWireFrame::Event {
                    frame: ralph_contracts::transport::RemoteEventFrame::TerminalOutput(
                        PtyOutputEvent {
                            session_id: "s1".to_owned(),
                            seq: 1,
                            data: "SGVsbG8=".to_owned(),
                        },
                    ),
                };
                ws.send(Message::Text(serde_json::to_string(&ev).unwrap().into()))
                    .await
                    .unwrap();
            }

            // Keep the socket alive briefly to allow the client to pump.
            tokio::time::sleep(Duration::from_millis(150)).await;
        });

        addr
    }

    #[tokio::test]
    async fn connect_hard_fails_on_protocol_mismatch() {
        let addr = spawn_test_server(PROTOCOL_VERSION + 1, false).await;
        let ws_url = format!("ws://{addr}");
        let sink: Arc<dyn EventSink> = Arc::new(RecordingSink::default());

        let err = RemoteWireFrameConnection::connect(ws_url, sink)
            .await
            .unwrap_err();

        assert!(err.contains("Protocol mismatch"), "err={err}");
    }

    #[tokio::test]
    async fn connect_reemits_remote_events_via_sink() {
        let addr = spawn_test_server(PROTOCOL_VERSION, true).await;
        let ws_url = format!("ws://{addr}");
        let sink: Arc<RecordingSink> = Arc::new(RecordingSink::default());
        let sink_obj: Arc<dyn EventSink> = sink.clone();

        let conn = RemoteWireFrameConnection::connect(ws_url, sink_obj)
            .await
            .unwrap();

        let expected_event_name = TERMINAL_OUTPUT_EVENT;
        assert_eq!(expected_event_name, "terminal:output");

        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                if !sink.output.lock().unwrap().is_empty() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();

        conn.shutdown().await.unwrap();
        assert_eq!(sink.output.lock().unwrap().len(), 1);
    }
}
