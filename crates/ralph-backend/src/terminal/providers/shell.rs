use portable_pty::CommandBuilder;
use std::path::Path;

use super::{AgentProvider, SessionConfig, AGENT_SHELL};

#[derive(Debug, Default)]
pub struct ShellAdapter;

fn resolve_shell_program() -> String {
    std::env::var("SHELL")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "/bin/bash".to_owned())
}

impl AgentProvider for ShellAdapter {
    fn id(&self) -> &'static str {
        AGENT_SHELL
    }

    fn list_models(&self) -> Vec<String> {
        Vec::new()
    }

    fn build_command(
        &self,
        working_dir: &Path,
        _mcp_config: Option<&Path>,
        _config: &SessionConfig,
    ) -> CommandBuilder {
        let shell_program = resolve_shell_program();
        let mut cmd = CommandBuilder::new(&shell_program);
        cmd.cwd(working_dir);
        cmd.arg("-i");
        cmd
    }
}
