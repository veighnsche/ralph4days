use ralph_macros::ipc_type;

#[ipc_type]
pub struct RalphProject {
    pub name: String,
    pub path: String,
}

#[ipc_type]
pub struct ProjectInfo {
    pub title: String,
    pub description: Option<String>,
    pub created: Option<String>,
}

#[ipc_type]
pub struct ProjectScanArgs {
    pub root_dir: Option<String>,
}

#[ipc_type]
pub struct RecentProject {
    pub path: String,
    pub name: String,
    pub last_opened: String,
}
