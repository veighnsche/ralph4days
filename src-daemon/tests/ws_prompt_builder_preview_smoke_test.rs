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
            RemoteWireFrame::Event { .. } | RemoteWireFrame::RpcRequest { .. } => {}
        }
    }
}

#[tokio::test]
async fn prompt_builder_preview_smoke() {
    let (mut child, ws_url) = spawn_ralphd().await;

    let (ws, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("connect ws");
    let (mut write, mut read) = ws.split();

    let project_dir = unique_tmp_dir("ralphd-ws-prompt-preview");
    std::fs::create_dir_all(&project_dir).expect("create project dir");

    let init_resp = rpc(
        &mut write,
        &mut read,
        1,
        "project_initialize",
        serde_json::json!({
            "args": {
                "path": project_dir.to_string_lossy().to_string(),
                "projectTitle": "ws prompt preview".to_owned(),
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
        2,
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

    let user_input = "hello from ws prompt preview";
    let preview_resp = rpc(
        &mut write,
        &mut read,
        3,
        "prompt_builder_preview",
        serde_json::json!({
            "args": {
                "sections": [
                    { "name": "user_input", "enabled": true, "instructionOverride": null }
                ],
                "userInput": user_input
            }
        }),
    )
    .await;

    let RemoteWireFrame::RpcOk { id, result } = preview_resp else {
        panic!("expected RpcOk for prompt_builder_preview");
    };
    assert_eq!(id, 3);

    let full_prompt = result
        .get("fullPrompt")
        .and_then(|v| v.as_str())
        .expect("fullPrompt should be a string");
    assert!(
        full_prompt.contains(user_input),
        "fullPrompt should include user input"
    );

    let _ = child.kill().await;
    let _ = std::fs::remove_dir_all(&project_dir);
}
