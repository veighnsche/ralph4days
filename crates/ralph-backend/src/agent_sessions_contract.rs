use ralph_macros::ipc_type;

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentSessionsByIdArgs {
    pub id: String,
}

