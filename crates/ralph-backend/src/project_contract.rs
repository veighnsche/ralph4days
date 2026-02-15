use ralph_macros::ipc_type;

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecentProject {
    pub path: String,
    pub name: String,
    pub last_opened: String,
}
