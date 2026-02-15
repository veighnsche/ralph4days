use ralph_macros::ipc_type;

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SubsystemsCreateArgs {
    pub name: String,
    pub display_name: String,
    pub acronym: String,
    pub description: Option<String>,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SubsystemsUpdateArgs {
    pub name: String,
    pub display_name: String,
    pub acronym: String,
    pub description: Option<String>,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SubsystemsDeleteArgs {
    pub name: String,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SubsystemsCommentUpdateArgs {
    pub subsystem_name: String,
    pub comment_id: u32,
    pub body: String,
    pub summary: Option<String>,
    pub reason: Option<String>,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SubsystemsCommentDeleteArgs {
    pub subsystem_name: String,
    pub comment_id: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subsystem_data_serializes_required_comments_array_even_when_empty() {
        let subsystem = SubsystemData {
            id: 1,
            name: "core".to_owned(),
            display_name: "Core".to_owned(),
            acronym: "CORE".to_owned(),
            description: None,
            created: None,
            status: "active".to_owned(),
            comments: vec![],
        };

        let value = serde_json::to_value(subsystem).expect("SubsystemData should serialize");
        let obj = value
            .as_object()
            .expect("SubsystemData should serialize to an object");

        let comments = obj
            .get("comments")
            .expect("comments should be present")
            .as_array()
            .expect("comments should be an array");
        assert!(
            comments.is_empty(),
            "comments should serialize as an empty array"
        );
    }
}
