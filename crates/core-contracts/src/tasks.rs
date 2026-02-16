use crate::domain::{Priority, TaskProvenance};
use core_macros::ipc_type;

#[ipc_type]
pub struct TasksCreateArgs {
    pub subsystem: String,
    pub discipline: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<Priority>,
    pub tags: Vec<String>,
    pub depends_on: Vec<u32>,
    pub acceptance_criteria: Vec<String>,
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
pub struct TasksUpdateArgs {
    pub id: u32,
    pub subsystem: String,
    pub discipline: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<Priority>,
    pub tags: Vec<String>,
    pub depends_on: Vec<u32>,
    pub acceptance_criteria: Vec<String>,
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
pub struct TasksSetStatusArgs {
    pub id: u32,
    pub status: String,
}

#[ipc_type]
pub struct TasksDeleteArgs {
    pub id: u32,
}

#[ipc_type]
pub struct TasksGetArgs {
    pub id: u32,
}

#[ipc_type]
pub struct TasksSignalAddArgs {
    pub task_id: u32,
    pub discipline: Option<String>,
    pub agent_task_id: Option<u32>,
    pub priority: Option<String>,
    pub body: String,
}

#[ipc_type]
pub struct TasksSignalUpdateArgs {
    pub task_id: u32,
    pub signal_id: u32,
    pub body: String,
}

#[ipc_type]
pub struct TasksSignalDeleteArgs {
    pub task_id: u32,
    pub signal_id: u32,
}

#[ipc_type]
pub struct TasksSignalSummariesGetArgs {
    pub task_ids: Vec<u32>,
}

#[ipc_type]
pub struct TasksAskAnswerArgs {
    pub signal_id: u32,
    pub answer: String,
}

#[ipc_type]
pub struct TasksCommentReplyAddArgs {
    pub task_id: u32,
    pub parent_comment_id: u32,
    pub priority: Option<String>,
    pub body: String,
}

#[ipc_type]
pub struct TasksSignalCommentUpdateArgs {
    pub comment_id: u32,
    pub body: String,
}

#[ipc_type]
pub struct TasksSignalCommentDeleteArgs {
    pub comment_id: u32,
}

#[ipc_type]
pub struct TasksSignalCommentsListArgs {
    pub signal_id: u32,
}
