use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use core_contracts::events::{BackendDiagnosticEvent, BackendDiagnosticLevel};
use core_contracts::transport::EventSink;
use core_errors::{codes, err_string, RalphResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
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
struct ApiServerState {
    sink: Arc<dyn EventSink>,
    db_path: Arc<RwLock<Option<String>>>,
}

pub async fn start_api_server(sink: Arc<dyn EventSink>) -> RalphResult<u16> {
    let state = ApiServerState {
        sink,
        db_path: Arc::new(RwLock::new(None)),
    };
    let sink = Arc::clone(&state.sink);

    let app = Router::new()
        .route("/api/task-signal", post(handle_signal))
        .route("/api/set-db-path", post(set_db_path))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| err_string(codes::INTERNAL, format!("Failed to bind api-server: {e}")))?;

    let port = listener
        .local_addr()
        .map_err(|e| {
            err_string(
                codes::INTERNAL,
                format!("Failed to get api-server addr: {e}"),
            )
        })?
        .port();

    tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, app).await {
            let message = format!("API server crashed: {error}");
            tracing::error!("{message}");
            if let Err(event_error) = sink.emit_backend_diagnostic(BackendDiagnosticEvent {
                level: BackendDiagnosticLevel::Error,
                source: "api-server".to_owned(),
                code: "crash".to_owned(),
                message,
            }) {
                tracing::warn!(
                    "Failed to emit backend-diagnostic for api-server crash: {event_error}"
                );
            }
        }
    });

    Ok(port)
}

async fn set_db_path(
    State(state): State<ApiServerState>,
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
    State(state): State<ApiServerState>,
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

    let db = match data_sqlite::SqliteDb::open(std::path::Path::new(&db_path), None) {
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

    if let Err(e) = state.sink.emit_backend_diagnostic(BackendDiagnosticEvent {
        level: BackendDiagnosticLevel::Warning,
        source: "api-server".to_owned(),
        code: "task-signal-inserted".to_owned(),
        message: format!("task_id={}, verb={}", event.task_id, event.verb),
    }) {
        tracing::warn!("Failed to emit backend-diagnostic for task-signal-inserted: {e}");
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "verb": request.verb
        })),
    )
}

fn insert_signal(db: &data_sqlite::SqliteDb, request: &SignalRequest) -> RalphResult<()> {
    match request.verb.as_str() {
        "done" => {
            let summary = request
                .payload
                .get("summary")
                .and_then(|v| v.as_str())
                .ok_or_else(|| err_string(codes::TASK_VALIDATION, "Missing summary"))?;

            db.insert_done_signal(
                None,
                data_sqlite::DoneSignalInput {
                    task_id: request.task_id,
                    session_id: request.session_id.clone(),
                    summary: summary.to_owned(),
                },
            )?;
        }
        "partial" => {
            let summary = request
                .payload
                .get("summary")
                .and_then(|v| v.as_str())
                .ok_or_else(|| err_string(codes::TASK_VALIDATION, "Missing summary"))?;
            let remaining = request
                .payload
                .get("remaining")
                .and_then(|v| v.as_str())
                .ok_or_else(|| err_string(codes::TASK_VALIDATION, "Missing remaining"))?;

            db.insert_partial_signal(
                None,
                data_sqlite::PartialSignalInput {
                    task_id: request.task_id,
                    session_id: request.session_id.clone(),
                    summary: summary.to_owned(),
                    remaining: remaining.to_owned(),
                },
            )?;
        }
        "stuck" => {
            let reason = request
                .payload
                .get("reason")
                .and_then(|v| v.as_str())
                .ok_or_else(|| err_string(codes::TASK_VALIDATION, "Missing reason"))?;

            db.insert_stuck_signal(
                None,
                data_sqlite::StuckSignalInput {
                    task_id: request.task_id,
                    session_id: request.session_id.clone(),
                    reason: reason.to_owned(),
                },
            )?;
        }
        "ask" => {
            let question = request
                .payload
                .get("question")
                .and_then(|v| v.as_str())
                .ok_or_else(|| err_string(codes::TASK_VALIDATION, "Missing question"))?;
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
                        .map(str::to_owned)
                        .collect::<Vec<_>>()
                });
            let preferred = request
                .payload
                .get("preferred")
                .and_then(|v| v.as_str())
                .map(str::to_owned);

            db.insert_ask_signal(
                None,
                data_sqlite::AskSignalInput {
                    task_id: request.task_id,
                    session_id: request.session_id.clone(),
                    question: question.to_owned(),
                    blocking,
                    options,
                    preferred,
                },
            )?;
        }
        "flag" => {
            let what = request
                .payload
                .get("what")
                .and_then(|v| v.as_str())
                .ok_or_else(|| err_string(codes::TASK_VALIDATION, "Missing what"))?;
            let severity = request
                .payload
                .get("severity")
                .and_then(|v| v.as_str())
                .ok_or_else(|| err_string(codes::TASK_VALIDATION, "Missing severity"))?;
            let category = request
                .payload
                .get("category")
                .and_then(|v| v.as_str())
                .ok_or_else(|| err_string(codes::TASK_VALIDATION, "Missing category"))?;

            db.insert_flag_signal(
                None,
                data_sqlite::FlagSignalInput {
                    task_id: request.task_id,
                    session_id: request.session_id.clone(),
                    what: what.to_owned(),
                    severity: severity.to_owned(),
                    category: category.to_owned(),
                },
            )?;
        }
        "learned" => {
            let text = request
                .payload
                .get("text")
                .and_then(|v| v.as_str())
                .ok_or_else(|| err_string(codes::TASK_VALIDATION, "Missing text"))?;
            let kind = request
                .payload
                .get("kind")
                .and_then(|v| v.as_str())
                .ok_or_else(|| err_string(codes::TASK_VALIDATION, "Missing kind"))?;
            let scope = request
                .payload
                .get("scope")
                .and_then(|v| v.as_str())
                .unwrap_or("feature");
            let rationale = request
                .payload
                .get("rationale")
                .and_then(|v| v.as_str())
                .map(str::to_owned);

            db.insert_learned_signal(
                None,
                data_sqlite::LearnedSignalInput {
                    task_id: request.task_id,
                    session_id: request.session_id.clone(),
                    text: text.to_owned(),
                    kind: kind.to_owned(),
                    scope: scope.to_owned(),
                    rationale,
                },
            )?;
        }
        "suggest" => {
            let what = request
                .payload
                .get("what")
                .and_then(|v| v.as_str())
                .ok_or_else(|| err_string(codes::TASK_VALIDATION, "Missing what"))?;
            let kind = request
                .payload
                .get("kind")
                .and_then(|v| v.as_str())
                .ok_or_else(|| err_string(codes::TASK_VALIDATION, "Missing kind"))?;
            let why = request
                .payload
                .get("why")
                .and_then(|v| v.as_str())
                .ok_or_else(|| err_string(codes::TASK_VALIDATION, "Missing why"))?;

            db.insert_suggest_signal(
                None,
                data_sqlite::SuggestSignalInput {
                    task_id: request.task_id,
                    session_id: request.session_id.clone(),
                    what: what.to_owned(),
                    kind: kind.to_owned(),
                    why: why.to_owned(),
                },
            )?;
        }
        "blocked" => {
            let on = request
                .payload
                .get("on")
                .and_then(|v| v.as_str())
                .ok_or_else(|| err_string(codes::TASK_VALIDATION, "Missing on"))?;
            let kind = request
                .payload
                .get("kind")
                .and_then(|v| v.as_str())
                .ok_or_else(|| err_string(codes::TASK_VALIDATION, "Missing kind"))?;
            let detail = request
                .payload
                .get("detail")
                .and_then(|v| v.as_str())
                .map(str::to_owned);

            db.insert_blocked_signal(
                None,
                data_sqlite::BlockedSignalInput {
                    task_id: request.task_id,
                    session_id: request.session_id.clone(),
                    on: on.to_owned(),
                    kind: kind.to_owned(),
                    detail,
                },
            )?;
        }
        _ => {
            return Err(err_string(
                codes::TASK_VALIDATION,
                format!("Unknown verb: {}", request.verb),
            ))
        }
    }

    Ok(())
}
