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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopConfig {
    pub project_path: PathBuf,
    pub max_iterations: u32,
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
            max_iterations: 100,
            haiku_iterations_before_opus: 3,
            max_stagnant_iterations: 3,
            iteration_timeout_secs: 900,
            rate_limit_retry_secs: 300,
            max_rate_limit_retries: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopStatus {
    pub state: LoopState,
    pub current_iteration: u32,
    pub max_iterations: u32,
    pub stagnant_count: u32,
    pub rate_limit_retries: u32,
    pub last_progress_hash: Option<String>,
    pub project_path: Option<PathBuf>,
}

impl Default for LoopStatus {
    fn default() -> Self {
        Self {
            state: LoopState::Idle,
            current_iteration: 0,
            max_iterations: 0,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeStreamEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(default)]
    pub subtype: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub error: Option<ClaudeError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeError {
    #[serde(rename = "type")]
    pub error_type: String,
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
