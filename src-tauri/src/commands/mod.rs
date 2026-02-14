pub(crate) mod agent_sessions;
pub(crate) mod project;
pub(crate) mod prompts;
pub(crate) mod protocol;
pub(crate) mod remote;
mod state;
pub(crate) mod subsystems;
pub(crate) mod tasks;
pub(crate) mod terminal_bridge;

pub use project::{project_lock_validated, project_validate_path};
pub use state::AppState;
