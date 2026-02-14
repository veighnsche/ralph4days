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
        if let Some(effort) = &config.effort {
            cmd.arg("--config");
            cmd.arg(format!("model_reasoning_effort={effort}"));
        }

        cmd
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::SessionInitSettings;

    fn to_argv_strings(cmd: &CommandBuilder) -> Vec<String> {
        cmd.get_argv()
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn includes_reasoning_effort_config_override_when_effort_is_set() {
        let adapter = CodexAdapter;
        let config = SessionConfig {
            agent: Some(AGENT_CODEX.to_owned()),
            model: Some("gpt-5.3-codex".to_owned()),
            effort: Some("high".to_owned()),
            thinking: None,
            permission_level: None,
            init_settings: SessionInitSettings::default(),
            post_start_preamble: None,
        };

        let cmd = adapter.build_command(Path::new("/tmp"), None, &config);
        let argv = to_argv_strings(&cmd);

        let config_flag_index = argv
            .iter()
            .position(|arg| arg == "--config")
            .expect("Expected --config flag");
        assert_eq!(
            argv.get(config_flag_index + 1).map(String::as_str),
            Some("model_reasoning_effort=high")
        );
    }

    #[test]
    fn omits_reasoning_effort_config_override_when_effort_is_unset() {
        let adapter = CodexAdapter;
        let config = SessionConfig {
            agent: Some(AGENT_CODEX.to_owned()),
            model: Some("gpt-5.3-codex".to_owned()),
            effort: None,
            thinking: None,
            permission_level: None,
            init_settings: SessionInitSettings::default(),
            post_start_preamble: None,
        };

        let cmd = adapter.build_command(Path::new("/tmp"), None, &config);
        let argv = to_argv_strings(&cmd);

        assert!(!argv.iter().any(|arg| arg == "--config"));
        assert!(!argv
            .iter()
            .any(|arg| arg.starts_with("model_reasoning_effort=")));
    }
}
