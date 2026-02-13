use super::session::build_settings_json;
pub use super::session::SessionConfig;
pub use claudecode::ClaudeCodeAdapter;
pub use codex::CodexAdapter;
pub use model_catalog::ModelEntry;
pub use provider_trait::{AgentProvider, AGENT_CLAUDE, AGENT_CODEX};

mod claudecode;
mod codex;
mod model_catalog;
#[path = "trait.rs"]
mod provider_trait;

static CLAUDE_ADAPTER: ClaudeCodeAdapter = ClaudeCodeAdapter;
static CODEX_ADAPTER: CodexAdapter = CodexAdapter;

pub fn resolve_agent_provider(agent: Option<&str>) -> &'static dyn AgentProvider {
    match normalize_agent(agent).as_deref() {
        Some(AGENT_CODEX) => &CODEX_ADAPTER,
        _ => &CLAUDE_ADAPTER,
    }
}

pub fn list_models_for_agent(agent: Option<&str>) -> Vec<String> {
    resolve_agent_provider(agent).list_models()
}

pub fn list_model_entries_for_agent(agent: Option<&str>) -> Vec<ModelEntry> {
    match normalize_agent(agent).as_deref() {
        Some(AGENT_CODEX) => model_catalog::codex_model_entries(),
        _ => model_catalog::claudecode_model_entries(),
    }
}

fn find_model_entry_for_agent(agent: Option<&str>, selected_model: &str) -> Option<ModelEntry> {
    list_model_entries_for_agent(agent)
        .into_iter()
        .find(|entry| {
            entry.name == selected_model || entry.session_model.as_deref() == Some(selected_model)
        })
}

pub fn resolve_session_model_for_agent(
    agent: Option<&str>,
    model: Option<String>,
) -> Result<Option<String>, String> {
    let Some(selected) = model else {
        return Ok(None);
    };
    let trimmed = selected.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if let Some(entry) = find_model_entry_for_agent(agent, trimmed) {
        return Ok(Some(entry.session_model.unwrap_or(entry.name)));
    }
    Err(format!(
        "Unknown model '{trimmed}' for agent '{}'",
        resolve_agent_provider(agent).id()
    ))
}

pub fn resolve_session_effort_for_agent(
    agent: Option<&str>,
    model: Option<&str>,
    effort: Option<String>,
) -> Result<Option<String>, String> {
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
        .ok_or_else(|| "Effort requires an explicit model selection".to_owned())?;
    let model_entry = find_model_entry_for_agent(agent, selected_model)
        .ok_or_else(|| format!("Effort validation failed: unknown model '{selected_model}'"))?;
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
        Err(format!(
            "Invalid effort '{normalized}' for model '{}'. Expected one of: {}",
            model_entry.name,
            model_entry.effort_options.join(", ")
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
    agent: Option<&str>,
    config: &SessionConfig,
    user_preamble: Option<String>,
) -> Option<String> {
    let provider = resolve_agent_provider(agent);
    let provider_preamble = provider.build_post_start_preamble(config);
    merge_post_start_preamble(user_preamble, provider_preamble)
}

fn normalize_agent(agent: Option<&str>) -> Option<String> {
    let raw = agent?;
    let normalized = raw.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }
    match normalized.as_str() {
        "claude-code" => Some(AGENT_CLAUDE.to_owned()),
        _ => Some(normalized),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_codex_provider() {
        let provider = resolve_agent_provider(Some("codex"));
        assert_eq!(provider.id(), AGENT_CODEX);
    }

    #[test]
    fn resolves_claude_provider_for_alias() {
        let provider = resolve_agent_provider(Some("claude-code"));
        assert_eq!(provider.id(), AGENT_CLAUDE);
    }

    #[test]
    fn resolves_claude_provider_as_default() {
        let provider = resolve_agent_provider(None);
        assert_eq!(provider.id(), AGENT_CLAUDE);
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
        let effort =
            resolve_session_effort_for_agent(Some("claude"), Some("opus"), Some("high".into()))
                .expect("effort should resolve");
        assert_eq!(effort.as_deref(), Some("high"));
    }

    #[test]
    fn rejects_effort_when_model_has_no_effort_capability() {
        let effort =
            resolve_session_effort_for_agent(Some("claude"), Some("sonnet"), Some("medium".into()))
                .expect("unsupported-model effort should be ignored");
        assert_eq!(effort, None);
    }

    #[test]
    fn rejects_invalid_effort_level_for_supported_model() {
        let err =
            resolve_session_effort_for_agent(Some("claude"), Some("opus"), Some("max".into()))
                .unwrap_err();
        assert!(err.contains("Invalid effort"));
    }

    #[test]
    fn rejects_unknown_model_for_agent() {
        let err = resolve_session_model_for_agent(Some("claude"), Some("not-a-real-model".into()))
            .unwrap_err();
        assert!(err.contains("Unknown model"));
    }
}
