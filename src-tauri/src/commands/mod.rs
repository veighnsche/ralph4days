pub(crate) mod features;
pub(crate) mod project;
pub(crate) mod prompts;
mod state;
pub(crate) mod tasks;
pub(crate) mod terminal;

pub use project::{set_locked_project, validate_project_path};
pub use state::AppState;
