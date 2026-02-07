use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Matches the actual `claude --output-format stream-json` output.
///
/// Examples:
/// ```json
/// {"type":"system","subtype":"init","session_id":"...","tools":[...]}
/// {"type":"assistant","message":{"content":[{"type":"text","text":"hello"}]}}
/// {"type":"result","subtype":"success","duration_ms":2686}
/// ```
///
/// We use `serde_json::Value` for the message field since the structure varies
/// by event type, and we only need to extract text content from assistant events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeStreamEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(default)]
    pub subtype: Option<String>,
    #[serde(default)]
    pub message: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<ClaudeError>,
}

impl ClaudeStreamEvent {
    /// Extract text content from an assistant event's nested message.content array.
    pub fn extract_text(&self) -> Option<String> {
        let message = self.message.as_ref()?;
        let content_arr = message.get("content")?.as_array()?;
        let mut text = String::new();
        for block in content_arr {
            if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                if let Some(t) = block.get("text").and_then(|t| t.as_str()) {
                    text.push_str(t);
                }
            }
        }
        if text.is_empty() {
            None
        } else {
            Some(text)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeError {
    #[serde(rename = "type")]
    pub error_type: String,
    #[serde(default)]
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum RalphError {
    #[error("Project path does not exist: {0}")]
    ProjectNotFound(PathBuf),

    #[error("Missing .ralph directory in project")]
    MissingRalphDir,

    #[error("Missing required file: {0}")]
    MissingFile(String),

    #[error("Claude process failed: {0}")]
    ClaudeProcessError(String),

    #[error("Loop is not running")]
    NotRunning,

    #[error("Loop is already running")]
    AlreadyRunning,

    #[error("IO error: {0}")]
    IoError(String),
}

impl Serialize for RalphError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<std::io::Error> for RalphError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e.to_string())
    }
}
