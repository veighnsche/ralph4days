use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoopState {
    Idle,
    Running,
    Paused,
    RateLimited,
    Complete,
    Aborted,
}

impl Default for LoopState {
    fn default() -> Self {
        Self::Idle
    }
}

// TODO: DEPRECATED ITERATION LOGIC
// - Remove max_iterations field, replace with loop_enabled: bool
// - Infinite loops (loop_enabled = true) run until stopped
// - Single run (loop_enabled = false) runs once
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopConfig {
    pub project_path: PathBuf,
    pub max_iterations: u32, // TODO: Replace with loop_enabled: bool
    pub haiku_iterations_before_opus: u32,
    pub max_stagnant_iterations: u32,
    pub iteration_timeout_secs: u64,
    pub rate_limit_retry_secs: u64,
    pub max_rate_limit_retries: u32,
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            project_path: PathBuf::new(),
            max_iterations: 100, // TODO: Replace with loop_enabled: false
            haiku_iterations_before_opus: 3,
            max_stagnant_iterations: 3,
            iteration_timeout_secs: 900,
            rate_limit_retry_secs: 300,
            max_rate_limit_retries: 5,
        }
    }
}

// TODO: DEPRECATED ITERATION LOGIC
// - Remove current_iteration and max_iterations fields
// - Can keep iteration count for display purposes but not for loop control
// - Loop should not complete based on iteration count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopStatus {
    pub state: LoopState,
    pub current_iteration: u32, // TODO: Keep for display only, not loop control
    pub max_iterations: u32,    // TODO: Remove, replace with loop_enabled in config
    pub stagnant_count: u32,
    pub rate_limit_retries: u32,
    pub last_progress_hash: Option<String>,
    pub project_path: Option<PathBuf>,
}

impl Default for LoopStatus {
    fn default() -> Self {
        Self {
            state: LoopState::Idle,
            current_iteration: 0, // TODO: Keep for display only
            max_iterations: 0,    // TODO: Remove
            stagnant_count: 0,
            rate_limit_retries: 0,
            last_progress_hash: None,
            project_path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum RalphEvent {
    StateChanged {
        state: LoopState,
        iteration: u32,
    },
    OutputChunk {
        text: String,
    },
    IterationComplete {
        iteration: u32,
        success: bool,
        message: Option<String>,
    },
    RateLimited {
        retry_in_secs: u64,
        attempt: u32,
        max_attempts: u32,
    },
    Error {
        message: String,
    },
}

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
        RalphError::IoError(e.to_string())
    }
}
