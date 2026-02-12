use ralph_macros::ipc_type;
use serde::Serialize;

pub const TERMINAL_BRIDGE_OUTPUT_EVENT: &str = "terminal_bridge:output";
pub const TERMINAL_BRIDGE_CLOSED_EVENT: &str = "terminal_bridge:closed";

#[ipc_type]
#[derive(Clone, Serialize)]
pub struct PtyOutputEvent {
    pub session_id: String,
    /// Base64-encoded PTY output (avoids JSON number[] serialization overhead)
    pub data: String,
}

#[ipc_type]
#[derive(Clone, Serialize)]
pub struct PtyClosedEvent {
    pub session_id: String,
    pub exit_code: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_bridge_event_names_are_stable() {
        assert_eq!(TERMINAL_BRIDGE_OUTPUT_EVENT, "terminal_bridge:output");
        assert_eq!(TERMINAL_BRIDGE_CLOSED_EVENT, "terminal_bridge:closed");
    }

    #[test]
    fn pty_output_event_serializes_expected_shape() {
        let payload = PtyOutputEvent {
            session_id: "session-42".to_owned(),
            data: "SGVsbG8=".to_owned(),
        };
        let json = serde_json::to_value(payload).unwrap();
        assert_eq!(json["session_id"], "session-42");
        assert_eq!(json["data"], "SGVsbG8=");
    }

    #[test]
    fn pty_closed_event_serializes_expected_shape() {
        let payload = PtyClosedEvent {
            session_id: "session-42".to_owned(),
            exit_code: 0,
        };
        let json = serde_json::to_value(payload).unwrap();
        assert_eq!(json["session_id"], "session-42");
        assert_eq!(json["exit_code"], 0);
    }
}
