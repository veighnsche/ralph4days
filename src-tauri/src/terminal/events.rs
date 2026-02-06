use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct PtyOutputEvent {
    pub session_id: String,
    pub data: Vec<u8>,
}

#[derive(Clone, Serialize)]
pub struct PtyClosedEvent {
    pub session_id: String,
    pub exit_code: u32,
}
