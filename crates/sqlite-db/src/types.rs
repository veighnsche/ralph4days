use ralph_rag::FeatureLearning;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Task status enum (stored in DB as snake_case text)
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
            TaskStatus::Pending => "pending",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Done => "done",
            TaskStatus::Blocked => "blocked",
            TaskStatus::Skipped => "skipped",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(TaskStatus::Pending),
            "in_progress" => Some(TaskStatus::InProgress),
            "done" => Some(TaskStatus::Done),
            "blocked" => Some(TaskStatus::Blocked),
            "skipped" => Some(TaskStatus::Skipped),
            _ => None,
        }
    }
}

/// Inferred task status (computed from TaskStatus + dependency graph)
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

/// Task priority enum
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
            Priority::Low => "low",
            Priority::Medium => "medium",
            Priority::High => "high",
            Priority::Critical => "critical",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "low" => Some(Priority::Low),
            "medium" => Some(Priority::Medium),
            "high" => Some(Priority::High),
            "critical" => Some(Priority::Critical),
            _ => None,
        }
    }
}

/// Task provenance — who created this task
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
            TaskProvenance::Agent => "agent",
            TaskProvenance::Human => "human",
            TaskProvenance::System => "system",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "agent" => Some(TaskProvenance::Agent),
            "human" => Some(TaskProvenance::Human),
            "system" => Some(TaskProvenance::System),
            _ => None,
        }
    }
}

/// Comment author — who wrote a task comment
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommentAuthor {
    Human,
    Agent,
}

impl CommentAuthor {
    pub fn as_str(&self) -> &'static str {
        match self {
            CommentAuthor::Human => "human",
            CommentAuthor::Agent => "agent",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "human" => Some(CommentAuthor::Human),
            "agent" => Some(CommentAuthor::Agent),
            _ => None,
        }
    }
}

/// A structured comment on a task (now with stable ID from DB)
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

/// MCP server configuration for discipline-specific tooling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
}

/// Task record with pre-joined feature/discipline display data.
/// Uses camelCase for JSON serialization (frontend-ready).
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
    // Pre-joined display fields
    pub feature_display_name: String,
    pub feature_acronym: String,
    pub discipline_display_name: String,
    pub discipline_acronym: String,
    pub discipline_icon: String,
    pub discipline_color: String,
}

/// Stats for a group of tasks (feature or discipline)
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

/// Overall project progress
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectProgress {
    pub total_tasks: u32,
    pub done_tasks: u32,
    pub progress_percent: u32,
}

/// Feature definition
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

/// Input for creating or updating a feature (excludes learnings — those are append-only).
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

/// Discipline definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discipline {
    pub name: String,
    pub display_name: String,
    pub icon: String,
    pub color: String,
    #[serde(default)]
    pub acronym: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conventions: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_servers: Vec<McpServerConfig>,
}

/// Project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
}

/// Input for creating a new task (before ID assignment)
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
