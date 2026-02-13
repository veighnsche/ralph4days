use portable_pty::CommandBuilder;
use std::path::Path;

use super::SessionConfig;

pub const AGENT_CLAUDE: &str = "claude";
pub const AGENT_CODEX: &str = "codex";
pub const AGENT_SHELL: &str = "shell";

pub trait AgentProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn build_post_start_preamble(&self, _config: &SessionConfig) -> Option<String> {
        None
    }
    fn list_models(&self) -> Vec<String>;
    fn build_command(
        &self,
        working_dir: &Path,
        mcp_config: Option<&Path>,
        config: &SessionConfig,
    ) -> CommandBuilder;
}
