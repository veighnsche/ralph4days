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
}
