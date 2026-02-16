use futures_util::{SinkExt, StreamExt};
use core_contracts::transport::RemoteWireFrame;
use std::process::Stdio;
use std::time::{SystemTime, UNIX_EPOCH};
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

fn unique_tmp_dir(prefix: &str) -> std::path::PathBuf {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    std::env::temp_dir().join(format!("{prefix}-{millis}-{}", std::process::id()))
}

async fn rpc(
    write: &mut (impl SinkExt<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin),
    read: &mut (impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin),
    events: &mut Vec<core_contracts::transport::RemoteEventFrame>,
    id: u64,
    command: &str,
    payload: serde_json::Value,
) -> RemoteWireFrame {
    let req = RemoteWireFrame::RpcRequest {
        id,
        command: command.to_owned(),
        payload,
    };
    write
        .send(Message::Text(serde_json::to_string(&req).unwrap().into()))
        .await
        .expect("send request");

    loop {
        let msg = timeout(Duration::from_secs(10), read.next())
            .await
            .expect("timeout waiting for frame")
            .expect("ws stream ended")
            .expect("ws ok");
        let Message::Text(text) = msg else { continue };
        let frame: RemoteWireFrame = serde_json::from_str(&text).expect("decode RemoteWireFrame");
        match &frame {
            RemoteWireFrame::RpcOk { id: got, .. } | RemoteWireFrame::RpcErr { id: got, .. } => {
                if *got == id {
                    return frame;
                }
            }
            RemoteWireFrame::Event { frame } => {
                events.push(frame.clone());
            }
            RemoteWireFrame::RpcRequest { .. } => {}
        }
    }
}

#[tokio::test]
async fn emits_backend_diagnostic_and_terminal_events() {
    let (mut child, ws_url) = spawn_ralphd().await;

    let (ws, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("connect ws");
    let (mut write, mut read) = ws.split();
    let mut events: Vec<core_contracts::transport::RemoteEventFrame> = Vec::new();

    // Trigger an unknown command; ralphd is expected to emit a backend-diagnostic event.
    let _ = rpc(
        &mut write,
        &mut read,
        &mut events,
        1,
        "definitely_not_a_command",
        serde_json::Value::Null,
    )
    .await;

    let mut saw_diag = events.iter().any(|ev| {
        matches!(
            ev,
            core_contracts::transport::RemoteEventFrame::BackendDiagnostic(_)
        )
    });
    let mut saw_terminal_output = false;
    let mut saw_terminal_closed = false;

    if !saw_diag {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        while tokio::time::Instant::now() < deadline {
            let msg = timeout(Duration::from_millis(500), read.next()).await;
            let Ok(Some(Ok(Message::Text(text)))) = msg else {
                continue;
            };
            let frame: RemoteWireFrame =
                serde_json::from_str(&text).expect("decode RemoteWireFrame");
            if let RemoteWireFrame::Event { frame } = frame {
                if matches!(
                    frame,
                    core_contracts::transport::RemoteEventFrame::BackendDiagnostic(_)
                ) {
                    saw_diag = true;
                    break;
                }
            }
        }
    }

    // Prepare a minimal project so terminal sessions have a valid working dir.
    let project_dir = unique_tmp_dir("ralphd-ws-events");
    std::fs::create_dir_all(&project_dir).expect("create project dir");

    let init_resp = rpc(
        &mut write,
        &mut read,
        &mut events,
        2,
        "project_initialize",
        serde_json::json!({
            "args": {
                "path": project_dir.to_string_lossy().to_string(),
                "projectTitle": "ws event smoke".to_owned(),
                "stack": 1
            }
        }),
    )
    .await;
    assert!(
        matches!(init_resp, RemoteWireFrame::RpcOk { .. }),
        "project_initialize should succeed"
    );

    let lock_resp = rpc(
        &mut write,
        &mut read,
        &mut events,
        3,
        "project_lock_set",
        serde_json::json!({
            "args": { "path": project_dir.to_string_lossy().to_string() }
        }),
    )
    .await;
    assert!(
        matches!(lock_resp, RemoteWireFrame::RpcOk { .. }),
        "project_lock_set should succeed"
    );

    // Start a shell session (debug-only provider) and assert we see output + closed events.
    let session_id = "ws-event-session-1";
    let start_resp = rpc(
        &mut write,
        &mut read,
        &mut events,
        4,
        "terminal_start_session",
        serde_json::json!({
            "args": {
                "sessionId": session_id,
                "agent": "shell"
            }
        }),
    )
    .await;
    assert!(
        matches!(start_resp, RemoteWireFrame::RpcOk { .. }),
        "terminal_start_session should succeed"
    );

    // Read a bit of the event stream, looking for the three key events.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    while tokio::time::Instant::now() < deadline {
        let msg = timeout(Duration::from_millis(500), read.next()).await;
        let Ok(Some(Ok(Message::Text(text)))) = msg else {
            continue;
        };
        let frame: RemoteWireFrame = serde_json::from_str(&text).expect("decode RemoteWireFrame");
        match frame {
            RemoteWireFrame::Event { frame } => match frame {
                core_contracts::transport::RemoteEventFrame::BackendDiagnostic(_) => {
                    saw_diag = true;
                }
                core_contracts::transport::RemoteEventFrame::TerminalOutput(ev) => {
                    if ev.session_id == session_id {
                        saw_terminal_output = true;
                    }
                }
                core_contracts::transport::RemoteEventFrame::TerminalClosed(ev) => {
                    if ev.session_id == session_id {
                        saw_terminal_closed = true;
                    }
                }
            },
            _ => {}
        }

        if saw_diag && saw_terminal_output {
            break;
        }
    }

    assert!(saw_diag, "expected backend-diagnostic event");
    assert!(saw_terminal_output, "expected terminal:output event");

    let term_resp = rpc(
        &mut write,
        &mut read,
        &mut events,
        5,
        "terminal_terminate",
        serde_json::json!({
            "args": { "sessionId": session_id }
        }),
    )
    .await;
    assert!(
        matches!(term_resp, RemoteWireFrame::RpcOk { .. }),
        "terminal_terminate should succeed"
    );

    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    while tokio::time::Instant::now() < deadline {
        let msg = timeout(Duration::from_millis(500), read.next()).await;
        let Ok(Some(Ok(Message::Text(text)))) = msg else {
            continue;
        };
        let frame: RemoteWireFrame = serde_json::from_str(&text).expect("decode RemoteWireFrame");
        if let RemoteWireFrame::Event { frame } = frame {
            if let core_contracts::transport::RemoteEventFrame::TerminalClosed(ev) = frame {
                if ev.session_id == session_id {
                    saw_terminal_closed = true;
                    break;
                }
            }
        }
    }

    assert!(saw_terminal_closed, "expected terminal:closed event");

    let _ = child.kill().await;
    let _ = std::fs::remove_dir_all(&project_dir);
}
