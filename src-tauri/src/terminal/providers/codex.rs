use portable_pty::CommandBuilder;
use std::path::Path;

use super::model_catalog;
use super::{AgentProvider, SessionConfig, AGENT_CODEX};

#[derive(Debug, Default)]
pub struct CodexAdapter;

fn apply_permission_level(cmd: &mut CommandBuilder, permission_level: Option<&str>) {
    match permission_level.map(str::trim) {
        Some("safe") => cmd.args([
            "--sandbox",
            "workspace-write",
            "--ask-for-approval",
            "untrusted",
        ]),
        Some("auto") => cmd.arg("--full-auto"),
        Some("full_auto") => cmd.arg("--dangerously-bypass-approvals-and-sandbox"),
        _ => cmd.args([
            "--sandbox",
            "workspace-write",
            "--ask-for-approval",
            "on-request",
        ]),
    }
}

impl AgentProvider for CodexAdapter {
    fn id(&self) -> &'static str {
        AGENT_CODEX
    }

    fn build_post_start_preamble(&self, config: &SessionConfig) -> Option<String> {
        if config.thinking == Some(true) {
            return Some("Use high-effort reasoning for this session.".to_owned());
        }
        None
    }

    fn list_models(&self) -> Vec<String> {
        model_catalog::codex_models()
    }

    fn build_command(
        &self,
        working_dir: &Path,
        _mcp_config: Option<&Path>,
        config: &SessionConfig,
    ) -> CommandBuilder {
        let mut cmd = CommandBuilder::new("codex");
        cmd.cwd(working_dir);
        apply_permission_level(&mut cmd, config.permission_level.as_deref());

        if let Some(model) = &config.model {
            cmd.args(["--model", model]);
        }

        cmd
    }
}
