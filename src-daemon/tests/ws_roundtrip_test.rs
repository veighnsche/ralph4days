use futures_util::{SinkExt, StreamExt};
use ralph_contracts::transport::RemoteWireFrame;
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

#[tokio::test]
async fn ws_roundtrip_protocol_version_get() {
    let (mut child, ws_url) = spawn_ralphd().await;

    let (ws, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("connect ws");
    let (mut write, mut read) = ws.split();

    let req = RemoteWireFrame::RpcRequest {
        id: 1,
        command: "protocol_version_get".to_owned(),
        payload: serde_json::Value::Null,
    };
    write
        .send(Message::Text(serde_json::to_string(&req).unwrap().into()))
        .await
        .expect("send request");

    let msg = read.next().await.expect("read response").expect("ws ok");
    let Message::Text(text) = msg else {
        panic!("expected text frame response");
    };
    let frame: RemoteWireFrame = serde_json::from_str(&text).expect("decode RemoteWireFrame");
    match frame {
        RemoteWireFrame::RpcOk { id, result } => {
            assert_eq!(id, 1);
            assert!(result.get("protocolVersion").is_some());
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let _ = child.kill().await;
}

