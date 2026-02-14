use crate::events::BackendDiagnosticEvent;
use crate::terminal::{PtyClosedEvent, PtyOutputEvent};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

pub trait EventSink: Send + Sync {
    fn emit_backend_diagnostic(&self, payload: BackendDiagnosticEvent) -> Result<(), String>;

    fn emit_terminal_output(&self, payload: PtyOutputEvent) -> Result<(), String>;

    fn emit_terminal_closed(&self, payload: PtyClosedEvent) -> Result<(), String>;
}

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Transport-agnostic invoke-style RPC.
///
/// Intended uses:
/// - Tauri backend "remote mode" can forward `#[tauri::command]` calls to a remote `ralphd`.
/// - `ralphd` unit tests can swap in an in-memory fake transport.
pub trait RpcClient: Send + Sync {
    fn invoke(
        &self,
        command: String,
        args: serde_json::Value,
    ) -> BoxFuture<'_, Result<serde_json::Value, String>>;
}

/// Remote event envelope for the `ralphd` event stream.
///
/// Serialization shape:
/// - `{ "event": "<event-name>", "payload": { ... } }`
///
/// Policy:
/// - Unknown event names are protocol errors (must fail loudly, not be silently ignored).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload", deny_unknown_fields)]
pub enum RemoteEventFrame {
    #[serde(rename = "backend-diagnostic")]
    BackendDiagnostic(BackendDiagnosticEvent),
    #[serde(rename = "terminal:output")]
    TerminalOutput(PtyOutputEvent),
    #[serde(rename = "terminal:closed")]
    TerminalClosed(PtyClosedEvent),
}

impl RemoteEventFrame {
    pub fn emit_to(self, sink: &dyn EventSink) -> Result<(), String> {
        match self {
            Self::BackendDiagnostic(payload) => sink.emit_backend_diagnostic(payload),
            Self::TerminalOutput(payload) => sink.emit_terminal_output(payload),
            Self::TerminalClosed(payload) => sink.emit_terminal_closed(payload),
        }
    }
}

pub trait RemoteEventStream: Send {
    fn next(&mut self) -> BoxFuture<'_, Option<Result<RemoteEventFrame, String>>>;
}

pub trait RemoteEventSource: Send + Sync {
    fn subscribe(&self) -> BoxFuture<'_, Result<Box<dyn RemoteEventStream>, String>>;
}

/// One-channel remote wire protocol frame (RPC + events).
///
/// Intended use: a single WebSocket that carries:
/// - invoke-style requests/responses (proxy-friendly),
/// - and server-push events (`RemoteEventFrame`).
///
/// Strict decode is mandatory: unknown frame types/fields are protocol errors.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case", deny_unknown_fields)]
pub enum RemoteWireFrame {
    RpcRequest {
        id: u64,
        command: String,
        /// Intended to preserve the existing invoke payload shape (including `{ "args": ... }`).
        payload: serde_json::Value,
    },
    RpcOk {
        id: u64,
        result: serde_json::Value,
    },
    RpcErr {
        id: u64,
        error: String,
    },
    Event {
        #[serde(flatten)]
        frame: RemoteEventFrame,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{BackendDiagnosticLevel, BACKEND_DIAGNOSTIC_EVENT};
    use crate::terminal::{TERMINAL_CLOSED_EVENT, TERMINAL_OUTPUT_EVENT};
    use std::sync::Mutex;

    #[test]
    fn remote_event_frame_serializes_backend_diagnostic_with_expected_tag() {
        let payload = BackendDiagnosticEvent {
            level: BackendDiagnosticLevel::Warning,
            source: "test".to_owned(),
            code: "code".to_owned(),
            message: "message".to_owned(),
        };
        let json = serde_json::to_value(RemoteEventFrame::BackendDiagnostic(payload)).unwrap();
        assert_eq!(json["event"], BACKEND_DIAGNOSTIC_EVENT);
        assert!(json.get("payload").is_some());
    }

    #[test]
    fn remote_event_frame_serializes_terminal_output_with_expected_tag() {
        let payload = PtyOutputEvent {
            session_id: "session-1".to_owned(),
            seq: 7,
            data: "SGVsbG8=".to_owned(),
        };
        let json = serde_json::to_value(RemoteEventFrame::TerminalOutput(payload)).unwrap();
        assert_eq!(json["event"], TERMINAL_OUTPUT_EVENT);
        assert!(json.get("payload").is_some());
    }

    #[test]
    fn remote_event_frame_serializes_terminal_closed_with_expected_tag() {
        let payload = PtyClosedEvent {
            session_id: "session-1".to_owned(),
            exit_code: 0,
        };
        let json = serde_json::to_value(RemoteEventFrame::TerminalClosed(payload)).unwrap();
        assert_eq!(json["event"], TERMINAL_CLOSED_EVENT);
        assert!(json.get("payload").is_some());
    }

    #[test]
    fn remote_event_frame_deserializes_backend_diagnostic() {
        let json = serde_json::json!({
            "event": BACKEND_DIAGNOSTIC_EVENT,
            "payload": {
                "level": "warning",
                "source": "test",
                "code": "code",
                "message": "message"
            }
        });
        let frame: RemoteEventFrame = serde_json::from_value(json).unwrap();
        match frame {
            RemoteEventFrame::BackendDiagnostic(payload) => {
                assert!(matches!(payload.level, BackendDiagnosticLevel::Warning));
                assert_eq!(payload.source, "test");
                assert_eq!(payload.code, "code");
                assert_eq!(payload.message, "message");
            }
            other => panic!("Expected BackendDiagnostic frame, got {other:?}"),
        }
    }

    #[test]
    fn remote_event_frame_deserializes_terminal_output() {
        let json = serde_json::json!({
            "event": TERMINAL_OUTPUT_EVENT,
            "payload": {
                "sessionId": "session-1",
                "seq": 7,
                "data": "SGVsbG8="
            }
        });
        let frame: RemoteEventFrame = serde_json::from_value(json).unwrap();
        match frame {
            RemoteEventFrame::TerminalOutput(payload) => {
                assert_eq!(payload.session_id, "session-1");
                assert_eq!(payload.seq, 7);
                assert_eq!(payload.data, "SGVsbG8=");
            }
            other => panic!("Expected TerminalOutput frame, got {other:?}"),
        }
    }

    #[test]
    fn remote_event_frame_deserializes_terminal_closed() {
        let json = serde_json::json!({
            "event": TERMINAL_CLOSED_EVENT,
            "payload": {
                "sessionId": "session-1",
                "exitCode": 0
            }
        });
        let frame: RemoteEventFrame = serde_json::from_value(json).unwrap();
        match frame {
            RemoteEventFrame::TerminalClosed(payload) => {
                assert_eq!(payload.session_id, "session-1");
                assert_eq!(payload.exit_code, 0);
            }
            other => panic!("Expected TerminalClosed frame, got {other:?}"),
        }
    }

    #[test]
    fn remote_event_frame_rejects_unknown_event_tag() {
        let json = serde_json::json!({ "event": "unknown", "payload": {} });
        let err = serde_json::from_value::<RemoteEventFrame>(json).unwrap_err();
        assert!(err.to_string().contains("unknown"));
    }

    #[test]
    fn remote_event_frame_rejects_unknown_top_level_fields() {
        let json = serde_json::json!({
            "event": BACKEND_DIAGNOSTIC_EVENT,
            "payload": {
                "level": "warning",
                "source": "test",
                "code": "code",
                "message": "message"
            },
            "extra": 1
        });
        let err = serde_json::from_value::<RemoteEventFrame>(json).unwrap_err();
        let message = err.to_string();
        assert!(
            message.contains("expected \"event\" or \"payload\""),
            "Expected top-level key validation error, got: {message}"
        );
    }

    #[test]
    fn remote_event_frame_rejects_unknown_payload_fields() {
        let json = serde_json::json!({
            "event": TERMINAL_OUTPUT_EVENT,
            "payload": {
                "sessionId": "session-1",
                "seq": 7,
                "data": "SGVsbG8=",
                "extraField": true
            }
        });
        let err = serde_json::from_value::<RemoteEventFrame>(json).unwrap_err();
        assert!(err.to_string().contains("unknown field"));
    }

    #[derive(Default)]
    struct RecordingSink {
        diags: Mutex<Vec<BackendDiagnosticEvent>>,
        output: Mutex<Vec<PtyOutputEvent>>,
        closed: Mutex<Vec<PtyClosedEvent>>,
    }

    impl EventSink for RecordingSink {
        fn emit_backend_diagnostic(&self, payload: BackendDiagnosticEvent) -> Result<(), String> {
            self.diags.lock().unwrap().push(payload);
            Ok(())
        }

        fn emit_terminal_output(&self, payload: PtyOutputEvent) -> Result<(), String> {
            self.output.lock().unwrap().push(payload);
            Ok(())
        }

        fn emit_terminal_closed(&self, payload: PtyClosedEvent) -> Result<(), String> {
            self.closed.lock().unwrap().push(payload);
            Ok(())
        }
    }

    #[test]
    fn remote_event_frame_emits_to_event_sink() {
        let sink = RecordingSink::default();

        RemoteEventFrame::TerminalOutput(PtyOutputEvent {
            session_id: "session-1".to_owned(),
            seq: 7,
            data: "SGVsbG8=".to_owned(),
        })
        .emit_to(&sink)
        .unwrap();

        RemoteEventFrame::TerminalClosed(PtyClosedEvent {
            session_id: "session-1".to_owned(),
            exit_code: 0,
        })
        .emit_to(&sink)
        .unwrap();

        RemoteEventFrame::BackendDiagnostic(BackendDiagnosticEvent {
            level: BackendDiagnosticLevel::Warning,
            source: "test".to_owned(),
            code: "code".to_owned(),
            message: "message".to_owned(),
        })
        .emit_to(&sink)
        .unwrap();

        assert_eq!(sink.output.lock().unwrap().len(), 1);
        assert_eq!(sink.closed.lock().unwrap().len(), 1);
        assert_eq!(sink.diags.lock().unwrap().len(), 1);
    }

    #[test]
    fn remote_wire_frame_serializes_event_flattened() {
        let json = serde_json::to_value(RemoteWireFrame::Event {
            frame: RemoteEventFrame::TerminalOutput(PtyOutputEvent {
                session_id: "session-1".to_owned(),
                seq: 7,
                data: "SGVsbG8=".to_owned(),
            }),
        })
        .unwrap();

        assert_eq!(json["type"], "event");
        assert_eq!(json["event"], TERMINAL_OUTPUT_EVENT);
        assert!(json.get("payload").is_some());
    }

    #[test]
    fn remote_wire_frame_deserializes_rpc_request() {
        let json = serde_json::json!({
            "type": "rpc-request",
            "id": 1,
            "command": "terminal_start_session",
            "payload": { "args": { "sessionId": "s1" } }
        });
        let frame: RemoteWireFrame = serde_json::from_value(json).unwrap();
        match frame {
            RemoteWireFrame::RpcRequest {
                id,
                command,
                payload,
            } => {
                assert_eq!(id, 1);
                assert_eq!(command, "terminal_start_session");
                assert!(payload.get("args").is_some());
            }
            other => panic!("Expected RpcRequest frame, got {other:?}"),
        }
    }

    #[test]
    fn remote_wire_frame_deserializes_rpc_ok() {
        let json = serde_json::json!({
            "type": "rpc-ok",
            "id": 1,
            "result": { "ok": true }
        });
        let frame: RemoteWireFrame = serde_json::from_value(json).unwrap();
        match frame {
            RemoteWireFrame::RpcOk { id, result } => {
                assert_eq!(id, 1);
                assert_eq!(result["ok"], true);
            }
            other => panic!("Expected RpcOk frame, got {other:?}"),
        }
    }

    #[test]
    fn remote_wire_frame_deserializes_rpc_err() {
        let json = serde_json::json!({
            "type": "rpc-err",
            "id": 1,
            "error": "[R-0000] boom"
        });
        let frame: RemoteWireFrame = serde_json::from_value(json).unwrap();
        match frame {
            RemoteWireFrame::RpcErr { id, error } => {
                assert_eq!(id, 1);
                assert!(error.contains("boom"));
            }
            other => panic!("Expected RpcErr frame, got {other:?}"),
        }
    }

    #[test]
    fn remote_wire_frame_rejects_unknown_type() {
        let json = serde_json::json!({ "type": "nope" });
        let err = serde_json::from_value::<RemoteWireFrame>(json).unwrap_err();
        assert!(err.to_string().contains("nope"));
    }

    #[test]
    fn remote_wire_frame_rejects_unknown_fields() {
        let json = serde_json::json!({
            "type": "rpc-ok",
            "id": 1,
            "result": {},
            "extra": 1
        });
        let err = serde_json::from_value::<RemoteWireFrame>(json).unwrap_err();
        assert!(
            err.to_string().contains("expected"),
            "Expected strict decode error, got: {err}"
        );
    }
}
