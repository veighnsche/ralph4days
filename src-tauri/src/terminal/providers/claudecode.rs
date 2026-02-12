use portable_pty::CommandBuilder;
use std::path::Path;

use super::model_catalog;
use super::{build_settings_json, AgentProvider, SessionConfig, AGENT_CLAUDE};

#[derive(Debug, Default)]
pub struct ClaudeCodeAdapter;

impl AgentProvider for ClaudeCodeAdapter {
    fn id(&self) -> &'static str {
        AGENT_CLAUDE
    }

    fn list_models(&self) -> Vec<String> {
        model_catalog::claudecode_models()
    }

    fn build_command(
        &self,
        working_dir: &Path,
        mcp_config: Option<&Path>,
        config: &SessionConfig,
    ) -> CommandBuilder {
        let mut cmd = CommandBuilder::new("claude");
        cmd.cwd(working_dir);

        cmd.args(["--permission-mode", "bypassPermissions"]);
        cmd.arg("--verbose");
        cmd.arg("--no-chrome");

        if let Some(model) = &config.model {
            cmd.args(["--model", model]);
        }
        if let Some(effort) = &config.effort {
            cmd.args(["--effort", effort]);
        }

        let settings_json = build_settings_json(&config.init_settings, config.thinking);
        cmd.args(["--settings", &settings_json]);

        if let Some(mcp_config) = mcp_config {
            cmd.args(["--mcp-config", &mcp_config.to_string_lossy()]);
        }

        cmd
    }
}
