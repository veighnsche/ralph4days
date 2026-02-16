#[cfg_attr(mobile, path = "agent_sessions_mobile.rs")]
pub(crate) mod agent_sessions;
#[cfg_attr(mobile, path = "project_mobile.rs")]
pub(crate) mod project;
#[cfg_attr(mobile, path = "prompts_mobile.rs")]
pub(crate) mod prompts;
pub(crate) mod remote;
mod remote_proxy;
mod state;
#[cfg_attr(mobile, path = "subsystems_mobile.rs")]
pub(crate) mod subsystems;
#[cfg_attr(mobile, path = "tasks_mobile.rs")]
pub(crate) mod tasks;
#[cfg_attr(mobile, path = "terminal_bridge_mobile.rs")]
pub(crate) mod terminal_bridge;

#[cfg(not(mobile))]
pub use project::project_lock_validated;
pub use state::AppState;

use ralph_contracts::protocol::ProtocolVersionInfo;

#[tauri::command]
pub fn protocol_version_get() -> ProtocolVersionInfo {
    ProtocolVersionInfo::current()
}
