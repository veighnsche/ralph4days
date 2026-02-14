use ralph_contracts::events::BACKEND_DIAGNOSTIC_EVENT;
use ralph_contracts::terminal::{TERMINAL_CLOSED_EVENT, TERMINAL_OUTPUT_EVENT};
use ralph_contracts::transport::EventSink;
use tauri::{AppHandle, Emitter};

pub struct TauriEventSink {
    app: AppHandle,
}

impl TauriEventSink {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl EventSink for TauriEventSink {
    fn emit_backend_diagnostic(
        &self,
        payload: ralph_contracts::events::BackendDiagnosticEvent,
    ) -> Result<(), String> {
        self.app
            .emit(BACKEND_DIAGNOSTIC_EVENT, payload)
            .map_err(|e| e.to_string())
    }

    fn emit_terminal_output(
        &self,
        payload: ralph_contracts::terminal::PtyOutputEvent,
    ) -> Result<(), String> {
        self.app
            .emit(TERMINAL_OUTPUT_EVENT, payload)
            .map_err(|e| e.to_string())
    }

    fn emit_terminal_closed(
        &self,
        payload: ralph_contracts::terminal::PtyClosedEvent,
    ) -> Result<(), String> {
        self.app
            .emit(TERMINAL_CLOSED_EVENT, payload)
            .map_err(|e| e.to_string())
    }
}
