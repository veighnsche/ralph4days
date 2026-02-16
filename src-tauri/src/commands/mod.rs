pub(crate) mod agent_sessions;
mod executor;
pub(crate) mod project;
pub(crate) mod prompts;
pub(crate) mod remote;
mod remote_proxy;
mod state;
pub(crate) mod subsystems;
pub(crate) mod tasks;
pub(crate) mod terminal_bridge;

#[cfg(not(mobile))]
pub use project::project_lock_validated;
pub use state::AppState;

use core_contracts::protocol::ProtocolVersionInfo;

#[tauri::command]
pub fn protocol_version_get() -> ProtocolVersionInfo {
    ProtocolVersionInfo::current()
}

#[tauri::command]
pub fn mobile_mode_get() -> bool {
    cfg!(mobile)
}
