use futures_util::{SinkExt, StreamExt};
use ralph_contracts::events::{BackendDiagnosticEvent, BackendDiagnosticLevel};
use ralph_contracts::transport::{RemoteEventFrame, RemoteWireFrame};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::{timeout, Duration};
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
async fn client_sent_non_request_frames_hard_fail() {
    let (mut child, ws_url) = spawn_ralphd().await;

    let (ws, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("connect ws");
    let (mut write, mut read) = ws.split();

    let illegal = RemoteWireFrame::Event {
        frame: RemoteEventFrame::BackendDiagnostic(BackendDiagnosticEvent {
            level: BackendDiagnosticLevel::Warning,
            source: "test".to_owned(),
            code: "illegal".to_owned(),
            message: "illegal".to_owned(),
        }),
    };

    write
        .send(Message::Text(
            serde_json::to_string(&illegal).unwrap().into(),
        ))
        .await
        .expect("send illegal frame");

    // Server should close the connection on protocol violations.
    let outcome = timeout(Duration::from_secs(5), read.next()).await;
    assert!(outcome.is_ok(), "expected connection to close promptly");

    let _ = child.kill().await;
}
