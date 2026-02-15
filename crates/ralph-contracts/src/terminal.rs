use ralph_macros::ipc_type;

pub const TERMINAL_OUTPUT_EVENT: &str = "terminal:output";
pub const TERMINAL_CLOSED_EVENT: &str = "terminal:closed";

#[ipc_type]
pub struct PtyOutputEvent {
    pub session_id: String,
    #[serde(
        serialize_with = "crate::json_safe::serialize_u64",
        deserialize_with = "crate::json_safe::deserialize_u64"
    )]
    #[ts(type = "number")]
    pub seq: u64,
    /// Base64-encoded PTY output (avoids JSON number[] serialization overhead)
    pub data: String,
}

#[ipc_type]
pub struct PtyClosedEvent {
    pub session_id: String,
    pub exit_code: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json_safe::MAX_JSON_SAFE_INTEGER_U64;

    #[test]
    fn terminal_event_names_are_stable() {
        assert_eq!(TERMINAL_OUTPUT_EVENT, "terminal:output");
        assert_eq!(TERMINAL_CLOSED_EVENT, "terminal:closed");
    }

    #[test]
    fn pty_output_event_serializes_expected_shape() {
        let payload = PtyOutputEvent {
            session_id: "session-42".to_owned(),
            seq: 7,
            data: "SGVsbG8=".to_owned(),
        };
        let json = serde_json::to_value(payload).unwrap();
        assert_eq!(json["sessionId"], "session-42");
        assert_eq!(json["seq"], 7);
        assert_eq!(json["data"], "SGVsbG8=");
    }

    #[test]
    fn pty_closed_event_serializes_expected_shape() {
        let payload = PtyClosedEvent {
            session_id: "session-42".to_owned(),
            exit_code: 0,
        };
        let json = serde_json::to_value(payload).unwrap();
        assert_eq!(json["sessionId"], "session-42");
        assert_eq!(json["exitCode"], 0);
    }

    #[test]
    fn pty_output_event_rejects_seq_out_of_json_safe_range() {
        let payload = PtyOutputEvent {
            session_id: "session-42".to_owned(),
            seq: MAX_JSON_SAFE_INTEGER_U64 + 1,
            data: "SGVsbG8=".to_owned(),
        };
        let err = serde_json::to_value(payload).unwrap_err();
        assert!(err.to_string().contains("JSON-safe"));
    }
}
