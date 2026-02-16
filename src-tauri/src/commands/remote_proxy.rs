use core_contracts::transport::RpcClient;
use core_errors::{codes, err_string, RalphResult};
use serde::de::DeserializeOwned;
use serde::Serialize;

fn decode_remote_result<TResult: DeserializeOwned>(
    command: &str,
    value: serde_json::Value,
) -> RalphResult<TResult> {
    serde_json::from_value::<TResult>(value).map_err(|e| {
        err_string(
            codes::INTERNAL,
            format!("Remote RPC result decode failed for '{command}': {e}"),
        )
    })
}

async fn remote_invoke_payload<TResult: DeserializeOwned>(
    rpc: &dyn RpcClient,
    command: &str,
    payload: serde_json::Value,
) -> RalphResult<TResult> {
    let value = rpc.invoke(command.to_owned(), payload).await?;
    decode_remote_result(command, value)
}

pub(crate) async fn remote_invoke_no_args<TResult: DeserializeOwned>(
    rpc: &dyn RpcClient,
    command: &str,
) -> RalphResult<TResult> {
    remote_invoke_payload(rpc, command, serde_json::Value::Null).await
}

pub(crate) async fn remote_invoke_args<TArgs: Serialize, TResult: DeserializeOwned>(
    rpc: &dyn RpcClient,
    command: &str,
    args: TArgs,
) -> RalphResult<TResult> {
    let payload = serde_json::json!({ "args": args });
    remote_invoke_payload(rpc, command, payload).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_contracts::protocol::{ProtocolVersionInfo, PROTOCOL_VERSION};
    use core_contracts::transport::RemoteWireFrame;
    use core_contracts::transport::{EventSink, RemoteEventFrame};
    use futures_util::{SinkExt, StreamExt};
    use std::net::SocketAddr;
    use std::sync::Arc;
    use tokio::net::TcpListener;
    use tokio_tungstenite::tungstenite::Message;

    struct NoopSink;

    impl EventSink for NoopSink {
        fn emit_backend_diagnostic(
            &self,
            _payload: core_contracts::events::BackendDiagnosticEvent,
        ) -> Result<(), String> {
            Ok(())
        }

        fn emit_terminal_output(
            &self,
            _payload: core_contracts::terminal::PtyOutputEvent,
        ) -> Result<(), String> {
            Ok(())
        }

        fn emit_terminal_closed(
            &self,
            _payload: core_contracts::terminal::PtyClosedEvent,
        ) -> Result<(), String> {
            Ok(())
        }
    }

    #[derive(Debug, Clone, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct TestArgs {
        foo: u32,
    }

    #[derive(Debug, Clone, serde::Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct TestResult {
        ok: bool,
    }

    async fn spawn_test_server() -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut ws = tokio_tungstenite::accept_async(stream).await.unwrap();

            // protocol_version_get handshake
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
                result: serde_json::to_value(ProtocolVersionInfo::current()).unwrap(),
            };
            ws.send(Message::Text(serde_json::to_string(&ok).unwrap().into()))
                .await
                .unwrap();

            // actual call
            let msg = ws.next().await.unwrap().unwrap();
            let Message::Text(text) = msg else {
                panic!("Expected text ws frame, got: {msg:?}");
            };
            let frame: RemoteWireFrame = serde_json::from_str(&text).unwrap();
            let RemoteWireFrame::RpcRequest {
                id,
                command,
                payload,
            } = frame
            else {
                panic!("Expected rpc-request, got: {frame:?}");
            };

            assert_eq!(command, "test_command");
            assert_eq!(payload, serde_json::json!({ "args": { "foo": 7 } }));

            // Server push event should be ignored by the invoke helper and handled by the pump.
            let event = RemoteWireFrame::Event {
                frame: RemoteEventFrame::TerminalClosed(core_contracts::terminal::PtyClosedEvent {
                    session_id: "s".to_owned(),
                    exit_code: 0,
                }),
            };
            ws.send(Message::Text(serde_json::to_string(&event).unwrap().into()))
                .await
                .unwrap();

            let ok = RemoteWireFrame::RpcOk {
                id,
                result: serde_json::json!({ "ok": true }),
            };
            ws.send(Message::Text(serde_json::to_string(&ok).unwrap().into()))
                .await
                .unwrap();

            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        });

        addr
    }

    #[tokio::test]
    async fn remote_invoke_args_wraps_payload_in_args_key_and_decodes_result() {
        assert_eq!(PROTOCOL_VERSION, 1);
        let addr = spawn_test_server().await;
        let ws_url = format!("ws://{addr}");

        let conn = crate::remote::RemoteWireFrameConnection::connect(ws_url, Arc::new(NoopSink))
            .await
            .unwrap();

        let rpc = conn.rpc_client();
        let result: TestResult = remote_invoke_args(&rpc, "test_command", TestArgs { foo: 7 })
            .await
            .unwrap();

        assert!(result.ok);
        conn.shutdown().await.unwrap();
    }
}
