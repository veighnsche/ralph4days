use super::session::build_settings_json;
use core_contracts::terminal_bridge::TerminalAgent;
use core_errors::{codes, err_string, RalphResult};

pub use super::session::SessionConfig;
pub use claudecode::ClaudeCodeAdapter;
pub use codex::CodexAdapter;
pub use model_catalog::ModelEntry;
pub use provider_trait::{AgentProvider, AGENT_CLAUDE, AGENT_CODEX, AGENT_SHELL};
pub use shell::ShellAdapter;

mod claudecode;
mod codex;
mod model_catalog;
#[path = "trait.rs"]
mod provider_trait;
mod shell;

static CLAUDE_ADAPTER: ClaudeCodeAdapter = ClaudeCodeAdapter;
static CODEX_ADAPTER: CodexAdapter = CodexAdapter;
static SHELL_ADAPTER: ShellAdapter = ShellAdapter;

pub fn shell_agent_enabled() -> bool {
    cfg!(debug_assertions)
}

fn default_agent(agent: Option<TerminalAgent>) -> TerminalAgent {
    agent.unwrap_or(TerminalAgent::Codex)
}

pub fn resolve_agent_provider(agent: Option<TerminalAgent>) -> RalphResult<&'static dyn AgentProvider> {
    let resolved = default_agent(agent);
    match resolved {
        TerminalAgent::Codex => Ok(&CODEX_ADAPTER),
        TerminalAgent::Claude => Ok(&CLAUDE_ADAPTER),
        TerminalAgent::Shell => {
            if shell_agent_enabled() {
                Ok(&SHELL_ADAPTER)
            } else {
                Err(err_string(
                    codes::TERMINAL,
                    "Shell terminal sessions are disabled in production builds",
                ))
            }
        }
    }
}

pub fn list_models_for_agent(agent: Option<TerminalAgent>) -> RalphResult<Vec<String>> {
    resolve_agent_provider(agent)?.list_models()
}

pub fn list_model_entries_for_agent(agent: Option<TerminalAgent>) -> RalphResult<Vec<ModelEntry>> {
    match default_agent(agent) {
        TerminalAgent::Claude => Ok(model_catalog::claudecode_model_entries()?),
        TerminalAgent::Shell => Ok(Vec::new()),
        TerminalAgent::Codex => Ok(model_catalog::codex_model_entries()?),
    }
}

fn find_model_entry_for_agent(
    agent: Option<TerminalAgent>,
    selected_model: &str,
) -> RalphResult<Option<ModelEntry>> {
    Ok(list_model_entries_for_agent(agent)?
        .into_iter()
        .find(|entry| {
            entry.name == selected_model || entry.session_model.as_deref() == Some(selected_model)
        }))
}

pub fn resolve_session_model_for_agent(
    agent: Option<TerminalAgent>,
    model: Option<String>,
) -> RalphResult<Option<String>> {
    let Some(selected) = model else {
        return Ok(None);
    };
    let trimmed = selected.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if let Some(entry) = find_model_entry_for_agent(agent, trimmed)? {
        return Ok(Some(entry.session_model.unwrap_or(entry.name)));
    }
    let provider = resolve_agent_provider(agent)?;
    Err(err_string(
        codes::TERMINAL,
        format!("Unknown model '{trimmed}' for agent '{}'", provider.id()),
    ))
}

pub fn resolve_session_effort_for_agent(
    agent: Option<TerminalAgent>,
    model: Option<&str>,
    effort: Option<String>,
) -> RalphResult<Option<String>> {
    let Some(selected) = effort else {
        return Ok(None);
    };
    let normalized = selected.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Ok(None);
    }
    let selected_model = model
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .ok_or_else(|| {
            err_string(
                codes::TERMINAL,
                "Effort requires an explicit model selection",
            )
        })?;
    let model_entry = find_model_entry_for_agent(agent, selected_model)?.ok_or_else(|| {
        err_string(
            codes::TERMINAL,
            format!("Effort validation failed: unknown model '{selected_model}'"),
        )
    })?;
    if model_entry.effort_options.is_empty() {
        return Ok(None);
    }
    if model_entry
        .effort_options
        .iter()
        .any(|level| level == &normalized)
    {
        Ok(Some(normalized))
    } else {
        Err(err_string(
            codes::TERMINAL,
            format!(
                "Invalid effort '{normalized}' for model '{}'. Expected one of: {}",
                model_entry.name,
                model_entry.effort_options.join(", ")
            ),
        ))
    }
}

pub fn merge_post_start_preamble(
    user_preamble: Option<String>,
    provider_preamble: Option<String>,
) -> Option<String> {
    match (provider_preamble, user_preamble) {
        (Some(provider), Some(user)) => Some(format!("{provider}\n{user}")),
        (Some(provider), None) => Some(provider),
        (None, Some(user)) => Some(user),
        (None, None) => None,
    }
}

pub fn resolve_post_start_preamble(
    agent: Option<TerminalAgent>,
    config: &SessionConfig,
    user_preamble: Option<String>,
) -> RalphResult<Option<String>> {
    let provider = resolve_agent_provider(agent)?;
    let provider_preamble = provider.build_post_start_preamble(config);
    Ok(merge_post_start_preamble(user_preamble, provider_preamble))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_codex_provider() {
        let provider = resolve_agent_provider(Some(TerminalAgent::Codex)).expect("codex provider");
        assert_eq!(provider.id(), AGENT_CODEX);
    }

    #[test]
    fn resolves_claude_provider() {
        let provider = resolve_agent_provider(Some(TerminalAgent::Claude)).expect("claude provider");
        assert_eq!(provider.id(), AGENT_CLAUDE);
    }

    #[test]
    fn resolves_codex_provider_as_default() {
        let provider = resolve_agent_provider(None).expect("default provider");
        assert_eq!(provider.id(), AGENT_CODEX);
    }

    #[test]
    fn resolves_shell_provider() {
        let provider = resolve_agent_provider(Some(TerminalAgent::Shell)).expect("shell provider");
        assert_eq!(provider.id(), AGENT_SHELL);
    }

    #[test]
    fn shell_agent_has_empty_model_catalog() {
        let models = list_model_entries_for_agent(Some(TerminalAgent::Shell)).expect("shell models");
        assert!(models.is_empty());
    }

    #[test]
    fn merges_provider_then_user_preamble() {
        let merged =
            merge_post_start_preamble(Some("user".to_owned()), Some("provider".to_owned()))
                .expect("preamble should exist");
        assert_eq!(merged, "provider\nuser");
    }

    #[test]
    fn validates_effort_from_model_capability() {
        let effort = resolve_session_effort_for_agent(
            Some(TerminalAgent::Claude),
            Some("opus"),
            Some("high".into()),
        )
        .expect("effort should resolve");
        assert_eq!(effort.as_deref(), Some("high"));
    }

    #[test]
    fn rejects_effort_when_model_has_no_effort_capability() {
        let effort = resolve_session_effort_for_agent(
            Some(TerminalAgent::Claude),
            Some("sonnet"),
            Some("medium".into()),
        )
        .expect("unsupported-model effort should be ignored");
        assert_eq!(effort, None);
    }

    #[test]
    fn rejects_invalid_effort_level_for_supported_model() {
        let err = resolve_session_effort_for_agent(
            Some(TerminalAgent::Claude),
            Some("opus"),
            Some("max".into()),
        )
        .unwrap_err();
        assert!(err.to_string().contains("Invalid effort"));
    }

    #[test]
    fn rejects_unknown_model_for_agent() {
        let err = resolve_session_model_for_agent(
            Some(TerminalAgent::Claude),
            Some("not-a-real-model".into()),
        )
        .unwrap_err();
        assert!(err.to_string().contains("Unknown model"));
    }
}
