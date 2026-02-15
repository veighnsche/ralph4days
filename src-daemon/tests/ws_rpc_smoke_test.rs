use futures_util::{SinkExt, StreamExt};
use ralph_contracts::transport::RemoteWireFrame;
use ralph_errors::codes;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio_tungstenite::tungstenite::Message;

async fn spawn_ralphd() -> (tokio::process::Child, String) {
    let exe = env!("CARGO_BIN_EXE_ralphd");
    let mut child = Command::new(exe)
        .arg("--bind")
        .arg("127.0.0.1:0")
        .env("RALPHD_PRINT_LISTEN_ADDR", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn ralphd");

    let stdout = child.stdout.take().expect("ralphd stdout");
    let mut lines = BufReader::new(stdout).lines();

    while let Some(line) = lines.next_line().await.expect("read ralphd stdout") {
        if let Some(addr) = line.strip_prefix("RALPHD_LISTEN_ADDR=") {
            let ws_url = format!("ws://{addr}");
            return (child, ws_url);
        }
    }

    panic!("ralphd did not print listen addr");
}

async fn rpc(ws_url: &str, frame: RemoteWireFrame) -> RemoteWireFrame {
    let (ws, _) = tokio_tungstenite::connect_async(ws_url)
        .await
        .expect("connect ws");
    let (mut write, mut read) = ws.split();
    write
        .send(Message::Text(serde_json::to_string(&frame).unwrap().into()))
        .await
        .expect("send request");
    let msg = read.next().await.expect("read response").expect("ws ok");
    let Message::Text(text) = msg else {
        panic!("expected text response");
    };
    serde_json::from_str(&text).expect("decode RemoteWireFrame")
}

#[tokio::test]
async fn protocol_version_get_yields_rpc_ok() {
    let (mut child, ws_url) = spawn_ralphd().await;

    let resp = rpc(
        &ws_url,
        RemoteWireFrame::RpcRequest {
            id: 1,
            command: "protocol_version_get".to_owned(),
            payload: serde_json::Value::Null,
        },
    )
    .await;

    match resp {
        RemoteWireFrame::RpcOk { id, result } => {
            assert_eq!(id, 1);
            assert!(result.get("protocolVersion").is_some());
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let _ = child.kill().await;
}

#[tokio::test]
async fn unknown_command_yields_rpc_err() {
    let (mut child, ws_url) = spawn_ralphd().await;

    let resp = rpc(
        &ws_url,
        RemoteWireFrame::RpcRequest {
            id: 1,
            command: "definitely_not_a_command".to_owned(),
            payload: serde_json::Value::Null,
        },
    )
    .await;

    match resp {
        RemoteWireFrame::RpcErr { id, error } => {
            assert_eq!(id, 1);
            assert_eq!(error.code, codes::INTERNAL);
            assert!(!error.message.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let _ = child.kill().await;
}
