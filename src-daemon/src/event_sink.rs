use ralph_contracts::events::BackendDiagnosticEvent;
use ralph_contracts::terminal::{PtyClosedEvent, PtyOutputEvent};
use ralph_contracts::transport::{EventSink, RemoteEventFrame, RemoteWireFrame};
use ralph_errors::{codes, err_string};
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct RalphdEventSink {
    tx: broadcast::Sender<String>,
}

impl RalphdEventSink {
    pub fn new(tx: broadcast::Sender<String>) -> Self {
        Self { tx }
    }

    fn emit_frame(&self, frame: RemoteEventFrame) -> Result<(), String> {
        let wire = RemoteWireFrame::Event { frame };
        let text = serde_json::to_string(&wire).map_err(|e| {
            err_string(
                codes::INTERNAL,
                format!("Failed to encode remote event frame: {e}"),
            )
        })?;

        // It's not an error if there are currently no connected clients.
        let _ = self.tx.send(text);
        Ok(())
    }
}

impl EventSink for RalphdEventSink {
    fn emit_backend_diagnostic(&self, payload: BackendDiagnosticEvent) -> Result<(), String> {
        self.emit_frame(RemoteEventFrame::BackendDiagnostic(payload))
    }

    fn emit_terminal_output(&self, payload: PtyOutputEvent) -> Result<(), String> {
        self.emit_frame(RemoteEventFrame::TerminalOutput(payload))
    }

    fn emit_terminal_closed(&self, payload: PtyClosedEvent) -> Result<(), String> {
        self.emit_frame(RemoteEventFrame::TerminalClosed(payload))
    }
}
