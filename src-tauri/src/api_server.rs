use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalRequest {
    task_id: u32,
    session_id: String,
    verb: String,
    payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct SignalEvent {
    task_id: u32,
    verb: String,
}

#[derive(Clone)]
struct AppState {
    app_handle: AppHandle,
    db_path: Arc<RwLock<Option<String>>>,
}

pub async fn start_api_server(app_handle: AppHandle) -> Result<u16, String> {
    let state = AppState {
        app_handle,
        db_path: Arc::new(RwLock::new(None)),
    };

    let app = Router::new()
        .route("/api/task-signal", post(handle_signal))
        .route("/api/set-db-path", post(set_db_path))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("Failed to bind to port: {e}"))?;

    let port = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local address: {e}"))?
        .port();

    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("API server crashed");
    });

    Ok(port)
}

async fn set_db_path(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some(path) = payload.get("db_path").and_then(|v| v.as_str()) {
        *state.db_path.write().await = Some(path.to_owned());
        (StatusCode::OK, "Database path set")
    } else {
        (StatusCode::BAD_REQUEST, "Missing db_path")
    }
}

async fn handle_signal(
    State(state): State<AppState>,
    Json(request): Json<SignalRequest>,
) -> impl IntoResponse {
    let db_path = state.db_path.read().await.clone();
    let Some(db_path) = db_path else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "Database path not set"
            })),
        );
    };

    let db = match sqlite_db::SqliteDb::open(std::path::Path::new(&db_path), None) {
        Ok(db) => db,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to open database: {e}")
                })),
            );
        }
    };

    let result = insert_signal(&db, &request);

    if let Err(e) = result {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to insert signal: {e}")
            })),
        );
    }

    let event = SignalEvent {
        task_id: request.task_id,
        verb: request.verb.clone(),
    };

    if let Err(e) = state.app_handle.emit("signal-added", &event) {
        eprintln!("Failed to emit signal-added event: {e}");
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "verb": request.verb
        })),
    )
}

fn insert_signal(db: &sqlite_db::SqliteDb, request: &SignalRequest) -> Result<(), String> {
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    match request.verb.as_str() {
        "done" => {
            let summary = request
                .payload
                .get("summary")
                .and_then(|v| v.as_str())
                .ok_or("Missing summary")?;

            db.execute_raw(&format!(
                "INSERT INTO task_comments (task_id, session_id, verb, summary, created) \
                 VALUES ({}, '{}', 'done', '{}', '{}')",
                request.task_id,
                request.session_id,
                summary.replace('\'', "''"),
                now
            ))?;
        }
        "partial" => {
            let summary = request
                .payload
                .get("summary")
                .and_then(|v| v.as_str())
                .ok_or("Missing summary")?;
            let remaining = request
                .payload
                .get("remaining")
                .and_then(|v| v.as_str())
                .ok_or("Missing remaining")?;

            db.execute_raw(&format!(
                "INSERT INTO task_comments (task_id, session_id, verb, summary, remaining, created) \
                 VALUES ({}, '{}', 'partial', '{}', '{}', '{}')",
                request.task_id,
                request.session_id,
                summary.replace('\'', "''"),
                remaining.replace('\'', "''"),
                now
            ))?;
        }
        "stuck" => {
            let reason = request
                .payload
                .get("reason")
                .and_then(|v| v.as_str())
                .ok_or("Missing reason")?;

            db.execute_raw(&format!(
                "INSERT INTO task_comments (task_id, session_id, verb, reason, created) \
                 VALUES ({}, '{}', 'stuck', '{}', '{}')",
                request.task_id,
                request.session_id,
                reason.replace('\'', "''"),
                now
            ))?;
        }
        "ask" => {
            let question = request
                .payload
                .get("question")
                .and_then(|v| v.as_str())
                .ok_or("Missing question")?;
            let blocking = request
                .payload
                .get("blocking")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            let options = request
                .payload
                .get("options")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join("\n")
                });
            let preferred = request.payload.get("preferred").and_then(|v| v.as_str());

            db.execute_raw(&format!(
                "INSERT INTO task_comments (task_id, session_id, verb, question, options, preferred, blocking, created) \
                 VALUES ({}, '{}', 'ask', '{}', {}, {}, {}, '{}')",
                request.task_id,
                request.session_id,
                question.replace('\'', "''"),
                options.map_or("NULL".to_owned(), |o| format!("'{}'", o.replace('\'', "''"))),
                preferred.map_or("NULL".to_owned(), |p| format!("'{}'", p.replace('\'', "''"))),
                i32::from(blocking),
                now
            ))?;
        }
        "flag" => {
            let what = request
                .payload
                .get("what")
                .and_then(|v| v.as_str())
                .ok_or("Missing what")?;
            let severity = request
                .payload
                .get("severity")
                .and_then(|v| v.as_str())
                .ok_or("Missing severity")?;
            let category = request
                .payload
                .get("category")
                .and_then(|v| v.as_str())
                .ok_or("Missing category")?;

            db.execute_raw(&format!(
                "INSERT INTO task_comments (task_id, session_id, verb, what, severity, category, created) \
                 VALUES ({}, '{}', 'flag', '{}', '{}', '{}', '{}')",
                request.task_id,
                request.session_id,
                what.replace('\'', "''"),
                severity,
                category,
                now
            ))?;
        }
        "learned" => {
            let text = request
                .payload
                .get("text")
                .and_then(|v| v.as_str())
                .ok_or("Missing text")?;
            let kind = request
                .payload
                .get("kind")
                .and_then(|v| v.as_str())
                .ok_or("Missing kind")?;
            let scope = request
                .payload
                .get("scope")
                .and_then(|v| v.as_str())
                .unwrap_or("feature");
            let rationale = request.payload.get("rationale").and_then(|v| v.as_str());

            db.execute_raw(&format!(
                "INSERT INTO task_comments (task_id, session_id, verb, text, kind, scope, rationale, created) \
                 VALUES ({}, '{}', 'learned', '{}', '{}', '{}', {}, '{}')",
                request.task_id,
                request.session_id,
                text.replace('\'', "''"),
                kind,
                scope,
                rationale.map_or("NULL".to_owned(), |r| format!("'{}'", r.replace('\'', "''"))),
                now
            ))?;
        }
        "suggest" => {
            let what = request
                .payload
                .get("what")
                .and_then(|v| v.as_str())
                .ok_or("Missing what")?;
            let kind = request
                .payload
                .get("kind")
                .and_then(|v| v.as_str())
                .ok_or("Missing kind")?;
            let why = request
                .payload
                .get("why")
                .and_then(|v| v.as_str())
                .ok_or("Missing why")?;

            db.execute_raw(&format!(
                "INSERT INTO task_comments (task_id, session_id, verb, what, kind, why, created) \
                 VALUES ({}, '{}', 'suggest', '{}', '{}', '{}', '{}')",
                request.task_id,
                request.session_id,
                what.replace('\'', "''"),
                kind,
                why.replace('\'', "''"),
                now
            ))?;
        }
        "blocked" => {
            let on = request
                .payload
                .get("on")
                .and_then(|v| v.as_str())
                .ok_or("Missing on")?;
            let kind = request
                .payload
                .get("kind")
                .and_then(|v| v.as_str())
                .ok_or("Missing kind")?;
            let detail = request.payload.get("detail").and_then(|v| v.as_str());

            db.execute_raw(&format!(
                "INSERT INTO task_comments (task_id, session_id, verb, \"on\", kind, detail, created) \
                 VALUES ({}, '{}', 'blocked', '{}', '{}', {}, '{}')",
                request.task_id,
                request.session_id,
                on.replace('\'', "''"),
                kind,
                detail.map_or("NULL".to_owned(), |d| format!("'{}'", d.replace('\'', "''"))),
                now
            ))?;
        }
        _ => return Err(format!("Unknown verb: {}", request.verb)),
    }

    Ok(())
}
