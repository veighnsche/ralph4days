use serde::Serialize;

pub const TERMINAL_BRIDGE_OUTPUT_EVENT: &str = "terminal_bridge.output";
pub const TERMINAL_BRIDGE_CLOSED_EVENT: &str = "terminal_bridge.closed";

#[derive(Clone, Serialize)]
pub struct PtyOutputEvent {
    pub session_id: String,
    /// Base64-encoded PTY output (avoids JSON number[] serialization overhead)
    pub data: String,
}

#[derive(Clone, Serialize)]
pub struct PtyClosedEvent {
    pub session_id: String,
    pub exit_code: u32,
}
