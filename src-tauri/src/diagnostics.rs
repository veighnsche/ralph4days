use serde::Serialize;
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter};

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BackendDiagnostic {
    level: String,
    source: String,
    code: String,
    message: String,
}

pub fn register_app_handle(app_handle: &AppHandle) {
    let _ = APP_HANDLE.set(app_handle.clone());
}

pub fn emit(level: &str, source: &str, code: &str, message: &str) {
    let event = BackendDiagnostic {
        level: level.to_owned(),
        source: source.to_owned(),
        code: code.to_owned(),
        message: message.to_owned(),
    };

    if let Some(handle) = APP_HANDLE.get() {
        if let Err(error) = handle.emit("backend-diagnostic", &event) {
            tracing::warn!("Failed to emit backend-diagnostic event: {error}");
        }
    } else {
        tracing::warn!("backend-diagnostic not registered: {}", event.message);
    }
}

pub fn emit_warning(source: &str, code: &str, message: &str) {
    emit("warning", source, code, message);
}
