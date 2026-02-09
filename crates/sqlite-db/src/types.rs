use ralph_macros::ipc_type;
use ralph_rag::FeatureLearning;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[ipc_type]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Done,
    Blocked,
    Skipped,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Done => "done",
            Self::Blocked => "blocked",
            Self::Skipped => "skipped",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "in_progress" => Some(Self::InProgress),
            "done" => Some(Self::Done),
            "blocked" => Some(Self::Blocked),
            "skipped" => Some(Self::Skipped),
            _ => None,
        }
    }
}

#[ipc_type]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InferredTaskStatus {
    Ready,
    WaitingOnDeps,
    ExternallyBlocked,
    InProgress,
    Done,
    Skipped,
}

#[ipc_type]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            "critical" => Some(Self::Critical),
            _ => None,
        }
    }
}

#[ipc_type]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskProvenance {
    Agent,
    Human,
    System,
}

impl TaskProvenance {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Agent => "agent",
            Self::Human => "human",
            Self::System => "system",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "agent" => Some(Self::Agent),
            "human" => Some(Self::Human),
            "system" => Some(Self::System),
            _ => None,
        }
    }
}

#[ipc_type]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommentAuthor {
    Human,
    Agent,
}

impl CommentAuthor {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Agent => "agent",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "human" => Some(Self::Human),
            "agent" => Some(Self::Agent),
            _ => None,
        }
    }
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskComment {
    pub id: u32,
    pub author: CommentAuthor,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_task_id: Option<u32>,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: u32,
    pub feature: String,
    pub discipline: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: TaskStatus,
    pub inferred_status: InferredTaskStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<Priority>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub acceptance_criteria: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_files: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub output_artifacts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hints: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_turns: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<TaskProvenance>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub comments: Vec<TaskComment>,
    pub feature_display_name: String,
    pub feature_acronym: String,
    pub discipline_display_name: String,
    pub discipline_acronym: String,
    pub discipline_icon: String,
    pub discipline_color: String,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupStats {
    pub name: String,
    pub display_name: String,
    pub total: u32,
    pub done: u32,
    pub pending: u32,
    pub in_progress: u32,
    pub blocked: u32,
    pub skipped: u32,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectProgress {
    pub total_tasks: u32,
    pub done_tasks: u32,
    pub progress_percent: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub name: String,
    pub display_name: String,
    #[serde(default)]
    pub acronym: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub knowledge_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boundaries: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub learnings: Vec<FeatureLearning>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
}

/// Learnings excluded — append-only via dedicated API.
#[derive(Default)]
pub struct FeatureInput {
    pub name: String,
    pub display_name: String,
    pub acronym: String,
    pub description: Option<String>,
    pub architecture: Option<String>,
    pub boundaries: Option<String>,
    pub knowledge_paths: Vec<String>,
    pub context_files: Vec<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DisciplineInput {
    pub name: String,
    pub display_name: String,
    pub acronym: String,
    pub icon: String,
    pub color: String,
    pub description: Option<String>,
    pub system_prompt: Option<String>,
    pub skills: String,
    pub conventions: Option<String>,
    pub mcp_servers: String,
    pub image_path: Option<String>,
    pub crops: Option<String>,
    pub image_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discipline {
    pub name: String,
    pub display_name: String,
    pub icon: String,
    pub color: String,
    #[serde(default)]
    pub acronym: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conventions: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_servers: Vec<McpServerConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stack_id: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crops: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
}

#[derive(Default)]
pub struct TaskInput {
    pub feature: String,
    pub discipline: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<Priority>,
    pub tags: Vec<String>,
    pub depends_on: Vec<u32>,
    pub acceptance_criteria: Option<Vec<String>>,
    pub context_files: Vec<String>,
    pub output_artifacts: Vec<String>,
    pub hints: Option<String>,
    pub estimated_turns: Option<u32>,
    pub provenance: Option<TaskProvenance>,
}
