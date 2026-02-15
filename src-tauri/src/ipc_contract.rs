// Public, minimal re-exports used by contract tests.
//
// WHY: Integration tests in `src-tauri/tests/*` can't access `pub(crate)` modules.
// Keep this surface intentionally small.

pub use crate::commands::remote::RemoteConnectResult;
pub use ralph_backend::agent_sessions_contract::AgentSessionsByIdArgs;
pub use ralph_backend::project_contract::{ProjectInfo, ProjectScanArgs, RalphProject};
