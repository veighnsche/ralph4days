use core_contracts::transport::RemoteWireFrame;
use futures_util::{SinkExt, StreamExt};
use std::path::Path;
use std::process::Stdio;
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use tokio_tungstenite::tungstenite::Message;

async fn spawn_fake_ollama() -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind fake ollama");
    let addr = listener.local_addr().expect("local addr");

    let handle = tokio::spawn(async move {
        loop {
            let (mut socket, _) = match listener.accept().await {
                Ok(v) => v,
                Err(_) => break,
            };

            tokio::spawn(async move {
                let mut buf = [0u8; 4096];
                let mut req = Vec::new();
                loop {
                    let n = socket.read(&mut buf).await.unwrap_or(0);
                    if n == 0 {
                        break;
                    }
                    req.extend_from_slice(&buf[..n]);
                    if req.windows(4).any(|w| w == b"\r\n\r\n") {
                        break;
                    }
                }

                let first_line = req
                    .split(|b| *b == b'\n')
                    .next()
                    .map(|l| String::from_utf8_lossy(l).trim().to_owned())
                    .unwrap_or_default();
                let mut parts = first_line.split_whitespace();
                let method = parts.next().unwrap_or("");
                let path = parts.next().unwrap_or("");

                let (status, body) = match (method, path) {
                    ("POST", "/api/embed") => (
                        "200 OK",
                        // embedding_dims = 1 in the test config below.
                        r#"{"embeddings":[[0.0]]}"#.to_owned(),
                    ),
                    ("GET", "/api/tags") => ("200 OK", r#"{"models":[]}"#.to_owned()),
                    _ => ("404 Not Found", r#"{"error":"not found"}"#.to_owned()),
                };

                let resp = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = socket.write_all(resp.as_bytes()).await;
                let _ = socket.shutdown().await;
            });
        }
    });

    (addr, handle)
}

fn write_external_services_config(config_home: &Path, ollama_addr: std::net::SocketAddr) {
    let ralph_dir = config_home.join("ralph");
    std::fs::create_dir_all(&ralph_dir).expect("create $XDG_CONFIG_HOME/ralph");

    let path = ralph_dir.join("external_services.json");
    let contents = serde_json::json!({
        "version": 1,
        "ollama": {
            "api_url": format!("http://{ollama_addr}"),
            "embedding_model": "test-embed",
            "embedding_dims": 1,
            "llm_model": "test-llm",
            "llm_temperature": 0.7
        },
        "comfy": {
            "api_url": "http://127.0.0.1:1",
            "default_workflow": "discipline_character.json",
            "timeout_secs": 1
        }
    });
    std::fs::write(&path, serde_json::to_string_pretty(&contents).unwrap())
        .expect("write external_services.json");
}

async fn spawn_ralphd(config_home: &Path, data_home: &Path) -> (tokio::process::Child, String) {
    let exe = env!("CARGO_BIN_EXE_ralphd");
    let mut child = Command::new(exe)
        .arg("--bind")
        .arg("127.0.0.1:0")
        .env("RALPHD_PRINT_LISTEN_ADDR", "1")
        .env("XDG_CONFIG_HOME", config_home)
        .env("XDG_DATA_HOME", data_home)
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
            RemoteWireFrame::Event { .. } => {}
            RemoteWireFrame::RpcRequest { .. } => {}
        }
    }
}

fn unwrap_ok(frame: RemoteWireFrame) -> serde_json::Value {
    match frame {
        RemoteWireFrame::RpcOk { result, .. } => result,
        other => panic!("expected rpc-ok, got: {other:?}"),
    }
}

#[tokio::test]
async fn v1_must_rpc_subset_roundtrips_over_ws() {
    let (ollama_addr, ollama_handle) = spawn_fake_ollama().await;
    let config_home = TempDir::new().expect("config tmpdir");
    let data_home = TempDir::new().expect("data tmpdir");
    write_external_services_config(config_home.path(), ollama_addr);

    let (mut child, ws_url) = spawn_ralphd(config_home.path(), data_home.path()).await;

    let (ws, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("connect ws");
    let (mut write, mut read) = ws.split();

    // protocol_version_get
    let result = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            1,
            "protocol_version_get",
            serde_json::Value::Null,
        )
        .await,
    );
    assert!(result.get("protocolVersion").is_some());

    // Prepare a project under a deterministic scan root.
    let scan_root = TempDir::new().expect("scan root");
    let project_path = scan_root.path().join("parity-project");
    std::fs::create_dir_all(&project_path).expect("create project dir");

    // project_initialize
    let _ = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            2,
            "project_initialize",
            serde_json::json!({
                "args": {
                    "path": project_path.to_string_lossy().to_string(),
                    "projectTitle": "parity smoke".to_owned(),
                    "stack": 1
                }
            }),
        )
        .await,
    );

    // project_lock_set + get
    let _ = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            3,
            "project_lock_set",
            serde_json::json!({ "args": { "path": project_path.to_string_lossy().to_string() } }),
        )
        .await,
    );
    let locked = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            4,
            "project_lock_get",
            serde_json::Value::Null,
        )
        .await,
    );
    assert!(
        locked.as_str().is_some() || locked.is_null(),
        "project_lock_get should return Option<String>"
    );

    // project_info_get
    let info = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            5,
            "project_info_get",
            serde_json::Value::Null,
        )
        .await,
    );
    assert_eq!(info["title"], "parity smoke");

    // project_scan (scoped root)
    let scan = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            6,
            "project_scan",
            serde_json::json!({ "args": { "rootDir": scan_root.path().to_string_lossy().to_string() } }),
        )
        .await,
    );
    assert!(scan.as_array().is_some());

    // project_recent_list (best-effort: should be an array)
    let recents = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            7,
            "project_recent_list",
            serde_json::Value::Null,
        )
        .await,
    );
    assert!(recents.as_array().is_some());

    // Seed one subsystem + task via direct DB write (setup only; not part of MUST RPC subset).
    let db_path = project_path.join(".ralph").join("db").join("ralph.db");
    let db = data_sqlite::SqliteDb::open(&db_path, None).expect("open db");
    db.create_subsystem(data_sqlite::SubsystemInput {
        name: "demo".to_owned(),
        display_name: "Demo".to_owned(),
        acronym: "DEMO".to_owned(),
        description: Some("Demo subsystem".to_owned()),
    })
    .expect("create subsystem");
    let task_id = db
        .create_task(data_sqlite::TaskInput {
            subsystem: "demo".to_owned(),
            discipline: "implementation".to_owned(),
            title: "Seed task".to_owned(),
            description: None,
            status: None,
            priority: None,
            tags: vec![],
            depends_on: vec![],
            acceptance_criteria: None,
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
            agent: None,
            model: None,
            effort: None,
            thinking: None,
        })
        .expect("create task");

    // tasks_list_items + tasks_get + tasks_update + tasks_set_status
    let items = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            8,
            "tasks_list_items",
            serde_json::Value::Null,
        )
        .await,
    );
    assert!(items.as_array().is_some());

    let task = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            9,
            "tasks_get",
            serde_json::json!({ "args": { "id": task_id } }),
        )
        .await,
    );
    assert_eq!(task["id"], task_id);

    let updated = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            10,
            "tasks_update",
            serde_json::json!({
                "args": {
                    "id": task_id,
                    "subsystem": "demo",
                    "discipline": "implementation",
                    "title": "Updated title",
                    "description": null,
                    "priority": null,
                    "tags": [],
                    "dependsOn": [],
                    "acceptanceCriteria": [],
                    "contextFiles": [],
                    "outputArtifacts": [],
                    "hints": null,
                    "estimatedTurns": null,
                    "provenance": null,
                    "agent": null,
                    "model": null,
                    "effort": null,
                    "thinking": null
                }
            }),
        )
        .await,
    );
    assert_eq!(updated["title"], "Updated title");

    let status = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            11,
            "tasks_set_status",
            serde_json::json!({ "args": { "id": task_id, "status": "in_progress" } }),
        )
        .await,
    );
    assert_eq!(status["status"], "in_progress");

    // tasks_signal_add + update + delete + summaries + comment_reply_add
    let signaled = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            12,
            "tasks_signal_add",
            serde_json::json!({
                "args": {
                    "taskId": task_id,
                    "discipline": null,
                    "agentTaskId": null,
                    "priority": null,
                    "body": "signal body"
                }
            }),
        )
        .await,
    );
    let signal_id = signaled["signals"]
        .as_array()
        .and_then(|a| a.last())
        .and_then(|v| v.get("id"))
        .and_then(|v| v.as_u64())
        .expect("signal id") as u32;

    let _ = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            13,
            "tasks_signal_update",
            serde_json::json!({ "args": { "taskId": task_id, "signalId": signal_id, "body": "updated signal" } }),
        )
        .await,
    );

    let _ = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            14,
            "tasks_signal_summaries_get",
            serde_json::json!({ "args": { "taskIds": [task_id] } }),
        )
        .await,
    );

    let _ = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            15,
            "tasks_comment_reply_add",
            serde_json::json!({
                "args": {
                    "taskId": task_id,
                    "parentCommentId": signal_id,
                    "priority": null,
                    "body": "reply"
                }
            }),
        )
        .await,
    );

    let _ = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            16,
            "tasks_signal_delete",
            serde_json::json!({ "args": { "taskId": task_id, "signalId": signal_id } }),
        )
        .await,
    );

    // subsystems_list + comment add/update/delete (requires fake ollama)
    let subsystems = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            17,
            "subsystems_list",
            serde_json::Value::Null,
        )
        .await,
    );
    assert!(subsystems.as_array().is_some());

    let subsystem = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            18,
            "subsystems_comment_add",
            serde_json::json!({
                "args": {
                    "subsystemName": "demo",
                    "category": "learning",
                    "discipline": null,
                    "agentTaskId": null,
                    "body": "hello",
                    "summary": null,
                    "reason": null,
                    "sourceIteration": null
                }
            }),
        )
        .await,
    );
    let comment_id = subsystem["comments"]
        .as_array()
        .and_then(|a| a.last())
        .and_then(|v| v.get("id"))
        .and_then(|v| v.as_u64())
        .expect("comment id") as u32;

    let _ = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            19,
            "subsystems_comment_update",
            serde_json::json!({
                "args": {
                    "subsystemName": "demo",
                    "commentId": comment_id,
                    "body": "updated",
                    "summary": null,
                    "reason": null
                }
            }),
        )
        .await,
    );

    let _ = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            20,
            "subsystems_comment_delete",
            serde_json::json!({ "args": { "subsystemName": "demo", "commentId": comment_id } }),
        )
        .await,
    );

    // disciplines_list + cropped image + create/update/delete
    let disciplines = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            21,
            "disciplines_list",
            serde_json::Value::Null,
        )
        .await,
    );
    let crops = disciplines
        .as_array()
        .and_then(|a| {
            a.iter().find(|d| {
                d.get("name") == Some(&serde_json::Value::String("implementation".to_owned()))
            })
        })
        .and_then(|d| d.get("crops"))
        .and_then(|c| c.as_object())
        .and_then(|c| c.get("face"))
        .cloned()
        .expect("implementation face crop");

    let cropped = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            22,
            "disciplines_cropped_image_get",
            serde_json::json!({
                "args": {
                    "disciplineName": "implementation",
                    "crop": crops,
                    "label": "face"
                }
            }),
        )
        .await,
    );
    assert!(
        cropped.is_null() || cropped.as_str().is_some(),
        "cropped image should be Option<String>"
    );

    let created = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            23,
            "disciplines_create",
            serde_json::json!({
                "args": {
                    "name": "parity_test",
                    "displayName": "Parity Test",
                    "acronym": "PART",
                    "icon": "Test",
                    "color": "#000000",
                    "systemPrompt": null,
                    "agent": null,
                    "model": null,
                    "effort": null,
                    "thinking": null,
                    "skills": [],
                    "conventions": null,
                    "mcpServers": []
                }
            }),
        )
        .await,
    );
    assert_eq!(created["name"], "parity_test");

    let updated = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            24,
            "disciplines_update",
            serde_json::json!({
                "args": {
                    "name": "parity_test",
                    "displayName": "Parity Test Updated",
                    "acronym": "PART",
                    "icon": "Test",
                    "color": "#000000",
                    "systemPrompt": null,
                    "agent": null,
                    "model": null,
                    "effort": null,
                    "thinking": null,
                    "skills": [],
                    "conventions": null,
                    "mcpServers": []
                }
            }),
        )
        .await,
    );
    assert_eq!(updated["displayName"], "Parity Test Updated");

    let _ = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            25,
            "disciplines_delete",
            serde_json::json!({ "args": { "name": "parity_test" } }),
        )
        .await,
    );

    // prompt builder config list/save/get/delete + preview
    let _ = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            26,
            "prompt_builder_config_list",
            serde_json::Value::Null,
        )
        .await,
    );

    let _ = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            27,
            "prompt_builder_config_save",
            serde_json::json!({
                "args": {
                    "config": {
                        "name": "parity_cfg",
                        "basePrompt": "yap",
                        "sectionOrder": ["project_metadata"],
                        "sections": { "project_metadata": { "enabled": true } }
                    }
                }
            }),
        )
        .await,
    );

    let got = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            28,
            "prompt_builder_config_get",
            serde_json::json!({ "args": { "name": "parity_cfg" } }),
        )
        .await,
    );
    assert_eq!(got["basePrompt"], "yap");

    let _ = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            29,
            "prompt_builder_config_delete",
            serde_json::json!({ "args": { "name": "parity_cfg" } }),
        )
        .await,
    );

    let preview = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            30,
            "prompt_builder_preview",
            serde_json::json!({
                "args": {
                    "sections": [
                        { "name": "project_metadata", "enabled": true, "instructionOverride": null },
                        { "name": "user_input", "enabled": true, "instructionOverride": null }
                    ],
                    "userInput": "hello"
                }
            }),
        )
        .await,
    );
    assert!(preview.get("fullPrompt").is_some());
    assert!(preview.get("sections").is_some());

    // terminal bridge: list model form tree + start/resize/set_stream_mode/replay/terminate
    let tree = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            31,
            "terminal_list_model_form_tree",
            serde_json::Value::Null,
        )
        .await,
    );
    assert!(tree.get("providers").is_some());

    let session_id = "parity-shell-1";
    let _ = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            32,
            "terminal_start_session",
            serde_json::json!({ "args": { "sessionId": session_id, "agent": "shell" } }),
        )
        .await,
    );
    let _ = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            33,
            "terminal_resize",
            serde_json::json!({ "args": { "sessionId": session_id, "cols": 80, "rows": 24 } }),
        )
        .await,
    );
    let _ = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            34,
            "terminal_set_stream_mode",
            serde_json::json!({ "args": { "sessionId": session_id, "mode": "buffered" } }),
        )
        .await,
    );
    let _ = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            35,
            "terminal_send_input",
            serde_json::json!({ "args": { "sessionId": session_id, "data": [101, 99, 104, 111, 32, 111, 107, 10] } }),
        )
        .await,
    );

    let replay = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            36,
            "terminal_replay_output",
            serde_json::json!({ "args": { "sessionId": session_id, "afterSeq": 0, "limit": 5 } }),
        )
        .await,
    );
    assert!(replay.get("chunks").is_some());

    let _ = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            37,
            "terminal_terminate",
            serde_json::json!({ "args": { "sessionId": session_id } }),
        )
        .await,
    );

    // terminal_start_task_session and terminal_start_human_session (shell agent defaults)
    let task_session_id = "parity-shell-task-1";
    let _ = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            38,
            "terminal_start_task_session",
            serde_json::json!({
                "args": {
                    "sessionId": task_session_id,
                    "taskId": task_id,
                    "agent": "shell",
                    "model": null,
                    "effort": null,
                    "permissionLevel": null,
                    "thinking": null,
                    "postStartPreamble": null
                }
            }),
        )
        .await,
    );

    let human = unwrap_ok(
        rpc(
            &mut write,
            &mut read,
            39,
            "terminal_start_human_session",
            serde_json::json!({
                "args": {
                    "terminalSessionId": "parity-human-1",
                    "kind": "manual",
                    "taskId": task_id,
                    "agent": "shell",
                    "model": null,
                    "effort": null,
                    "permissionLevel": null,
                    "postStartPreamble": null,
                    "initPrompt": null,
                    "mcpMode": null,
                    "thinking": null
                }
            }),
        )
        .await,
    );
    assert!(human.get("agentSessionId").is_some());

    let _ = child.kill().await;
    ollama_handle.abort();
    let _ = ollama_handle.await;
}
