pub(crate) mod agent_sessions;
pub(crate) mod project;
pub(crate) mod prompts;
mod state;
pub(crate) mod subsystems;
pub(crate) mod tasks;
pub(crate) mod terminal_bridge;

pub use project::{lock_project_validated, validate_project_path};
pub use state::AppState;
