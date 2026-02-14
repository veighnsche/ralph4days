use ralph_macros::ipc_type;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[ipc_type]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Draft,
    Pending,
    InProgress,
    Done,
    Blocked,
    Skipped,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Done => "done",
            Self::Blocked => "blocked",
            Self::Skipped => "skipped",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "draft" => Some(Self::Draft),
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSignal {
    pub id: u32,
    pub author: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_verb: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_signal_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub question: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub what: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocking: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub why: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answer: Option<String>,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: u32,
    pub subsystem: String,
    pub discipline: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: TaskStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<Priority>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<String>,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    #[serde(default)]
    pub context_files: Vec<String>,
    #[serde(default)]
    pub output_artifacts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hints: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_turns: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<TaskProvenance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pseudocode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enriched_at: Option<String>,
    #[serde(default)]
    pub signals: Vec<TaskSignal>,
    pub subsystem_display_name: String,
    pub subsystem_acronym: String,
    pub discipline_display_name: String,
    pub discipline_acronym: String,
    pub discipline_icon: String,
    pub discipline_color: String,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskListItem {
    pub id: u32,
    pub subsystem: String,
    pub discipline: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: TaskStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<Priority>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<u32>,
    pub acceptance_criteria_count: u32,
    pub signal_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<TaskProvenance>,
    pub subsystem_display_name: String,
    pub subsystem_acronym: String,
    pub discipline_display_name: String,
    pub discipline_acronym: String,
    pub discipline_icon: String,
    pub discipline_color: String,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskTemplate {
    pub id: u32,
    pub discipline_id: u32,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<Priority>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hints: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_turns: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pseudocode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
    pub pulled_count: u32,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupStats {
    pub name: String,
    pub display_name: String,
    pub total: u32,
    pub draft: u32,
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

#[ipc_type]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubsystemStatus {
    Active,
    Archived,
}

impl SubsystemStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "active" => Some(Self::Active),
            "archived" => Some(Self::Archived),
            _ => None,
        }
    }
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsystemComment {
    pub id: u32,
    pub category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discipline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_task_id: Option<u32>,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_iteration: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subsystem {
    pub id: u32,
    pub name: String,
    pub display_name: String,
    #[serde(default)]
    pub acronym: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    pub status: SubsystemStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub comments: Vec<SubsystemComment>,
}

#[derive(Default)]
pub struct SubsystemInput {
    pub name: String,
    pub display_name: String,
    pub acronym: String,
    pub description: Option<String>,
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
    pub agent: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub thinking: Option<bool>,
    pub skills: String,
    pub conventions: Option<String>,
    pub mcp_servers: String,
    pub image_path: Option<String>,
    pub crops: Option<String>,
    pub image_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discipline {
    pub id: u32,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking: Option<bool>,
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

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSignalSummary {
    pub pending_asks: u32,
    pub flag_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_flag_severity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_closing_verb: Option<String>,
    pub session_count: u32,
    pub learned_count: u32,
}

#[derive(Default)]
pub struct TaskInput {
    pub subsystem: String,
    pub discipline: String,
    pub title: String,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<Priority>,
    pub tags: Vec<String>,
    pub depends_on: Vec<u32>,
    pub acceptance_criteria: Option<Vec<String>>,
    pub context_files: Vec<String>,
    pub output_artifacts: Vec<String>,
    pub hints: Option<String>,
    pub estimated_turns: Option<u32>,
    pub provenance: Option<TaskProvenance>,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub thinking: Option<bool>,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSession {
    pub id: String,
    pub session_number: u32,
    pub kind: String,
    pub started_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_start_preamble: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub init_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closing_verb: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_bytes: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_text: Option<String>,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSessionCreateInput {
    pub id: String,
    pub kind: String,
    pub task_id: Option<u32>,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub launch_command: Option<String>,
    pub post_start_preamble: Option<String>,
    pub init_prompt: Option<String>,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSessionUpdateInput {
    pub id: String,
    pub kind: Option<String>,
    pub task_id: Option<u32>,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub launch_command: Option<String>,
    pub post_start_preamble: Option<String>,
    pub init_prompt: Option<String>,
    pub ended: Option<String>,
    pub exit_code: Option<i32>,
    pub closing_verb: Option<String>,
    pub status: Option<String>,
    pub prompt_hash: Option<String>,
    pub output_bytes: Option<u32>,
    pub error_text: Option<String>,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSignalComment {
    pub id: u32,
    pub signal_id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub author_type: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSignalCommentCreateInput {
    pub signal_id: u32,
    pub session_id: Option<String>,
    pub author_type: String,
    pub body: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_serializes_required_arrays_even_when_empty() {
        let task = Task {
            id: 1,
            subsystem: "core".to_owned(),
            discipline: "frontend".to_owned(),
            title: "Test task".to_owned(),
            description: None,
            status: TaskStatus::Draft,
            priority: None,
            tags: vec![],
            depends_on: vec![],
            created: None,
            updated: None,
            completed: None,
            acceptance_criteria: vec![],
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
            agent: None,
            model: None,
            effort: None,
            thinking: None,
            pseudocode: None,
            enriched_at: None,
            signals: vec![],
            subsystem_display_name: "Core".to_owned(),
            subsystem_acronym: "CORE".to_owned(),
            discipline_display_name: "Frontend".to_owned(),
            discipline_acronym: "FE".to_owned(),
            discipline_icon: "code".to_owned(),
            discipline_color: "#000000".to_owned(),
        };

        let value = serde_json::to_value(task).expect("Task should serialize");
        let obj = value
            .as_object()
            .expect("Task should serialize to a JSON object");

        for key in [
            "tags",
            "dependsOn",
            "acceptanceCriteria",
            "contextFiles",
            "outputArtifacts",
            "signals",
        ] {
            let Some(v) = obj.get(key) else {
                panic!("Task JSON missing required key: {key}");
            };
            assert!(
                v.is_array(),
                "Task JSON key {key} should be an array; got {v:?}"
            );
        }
    }

    #[test]
    fn mcp_server_config_serializes_args_and_env_even_when_empty() {
        let cfg = McpServerConfig {
            name: "test".to_owned(),
            command: "echo".to_owned(),
            args: vec![],
            env: HashMap::new(),
        };

        let value = serde_json::to_value(cfg).expect("McpServerConfig should serialize");
        let obj = value
            .as_object()
            .expect("McpServerConfig should serialize to a JSON object");

        assert!(
            obj.contains_key("args"),
            "McpServerConfig JSON missing args"
        );
        assert!(
            obj.get("args").is_some_and(serde_json::Value::is_array),
            "McpServerConfig args should be an array"
        );

        assert!(obj.contains_key("env"), "McpServerConfig JSON missing env");
        assert!(
            obj.get("env").is_some_and(serde_json::Value::is_object),
            "McpServerConfig env should be an object"
        );
    }
}
