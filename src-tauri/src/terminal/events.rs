use serde::Serialize;

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
