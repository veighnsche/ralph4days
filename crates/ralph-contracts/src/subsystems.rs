use ralph_macros::ipc_type;

#[ipc_type]
pub struct SubsystemCommentData {
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

#[ipc_type]
pub struct SubsystemData {
    pub id: u32,
    pub name: String,
    pub display_name: String,
    pub acronym: String,
    pub description: Option<String>,
    pub created: Option<String>,
    pub status: String,
    pub comments: Vec<SubsystemCommentData>,
}

#[ipc_type]
pub struct SubsystemsCreateArgs {
    pub name: String,
    pub display_name: String,
    pub acronym: String,
    pub description: Option<String>,
}

#[ipc_type]
pub struct SubsystemsUpdateArgs {
    pub name: String,
    pub display_name: String,
    pub acronym: String,
    pub description: Option<String>,
}

#[ipc_type]
pub struct SubsystemsDeleteArgs {
    pub name: String,
}

#[ipc_type]
pub struct SubsystemsCommentAddArgs {
    pub subsystem_name: String,
    pub category: String,
    pub discipline: Option<String>,
    pub agent_task_id: Option<u32>,
    pub body: String,
    pub summary: Option<String>,
    pub reason: Option<String>,
    pub source_iteration: Option<u32>,
}

#[ipc_type]
pub struct SubsystemsCommentUpdateArgs {
    pub subsystem_name: String,
    pub comment_id: u32,
    pub body: String,
    pub summary: Option<String>,
    pub reason: Option<String>,
}

#[ipc_type]
pub struct SubsystemsCommentDeleteArgs {
    pub subsystem_name: String,
    pub comment_id: u32,
}
