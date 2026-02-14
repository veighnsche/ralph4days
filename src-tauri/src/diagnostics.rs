use ralph_contracts::events::{BackendDiagnosticEvent, BackendDiagnosticLevel};
use ralph_contracts::transport::EventSink;
use std::sync::{Arc, OnceLock};

static EVENT_SINK: OnceLock<Arc<dyn EventSink>> = OnceLock::new();

pub fn register_sink(sink: Arc<dyn EventSink>) {
    assert!(
        EVENT_SINK.set(sink).is_ok(),
        "backend-diagnostic sink already registered"
    );
}

pub fn emit(level: BackendDiagnosticLevel, source: &str, code: &str, message: &str) {
    let event = BackendDiagnosticEvent {
        level,
        source: source.to_owned(),
        code: code.to_owned(),
        message: message.to_owned(),
    };

    if let Some(sink) = EVENT_SINK.get() {
        if let Err(error) = sink.emit_backend_diagnostic(event) {
            tracing::warn!("Failed to emit backend-diagnostic event: {error}");
        }
    } else {
        tracing::warn!("backend-diagnostic sink not registered: {}", event.message);
    }
}

pub fn emit_warning(source: &str, code: &str, message: &str) {
    emit(BackendDiagnosticLevel::Warning, source, code, message);
}
