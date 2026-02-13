use crate::diagnostics;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ModelEntry {
    pub name: String,
    pub display: String,
    pub description: String,
    pub session_model: Option<String>,
    pub effort_options: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CodexModelCatalog {
    models: Vec<ModelSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct ModelSpec {
    name: String,
    #[serde(default)]
    display: Option<String>,
    description: String,
    #[serde(default)]
    session_model: Option<String>,
    #[serde(default)]
    effort_options: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ClaudeCodeModelCatalog {
    models: Vec<ModelSpec>,
}

impl From<ModelSpec> for ModelEntry {
    fn from(spec: ModelSpec) -> Self {
        let ModelSpec {
            name,
            display,
            description,
            session_model,
            effort_options,
        } = spec;
        Self {
            display: display.unwrap_or_else(|| name.clone()),
            name,
            description,
            session_model,
            effort_options,
        }
    }
}

fn dedupe_valid_models(models: Vec<ModelSpec>) -> Vec<ModelSpec> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();
    for model in models {
        let trimmed_name = model.name.trim();
        if trimmed_name.is_empty() {
            continue;
        }
        if seen.insert(trimmed_name.to_owned()) {
            deduped.push(model);
        }
    }
    deduped
}

fn parse_codex_models_yaml(yaml: &str) -> Vec<ModelSpec> {
    let parsed = serde_yaml::from_str::<CodexModelCatalog>(yaml).unwrap_or_else(|err| {
        diagnostics::emit_warning(
            "terminal-providers",
            "model-catalog-parse-codex",
            &format!("Invalid codex model catalog YAML: {err}"),
        );
        tracing::warn!("Invalid codex model catalog YAML: {err}");
        return CodexModelCatalog { models: vec![] };
    });
    dedupe_valid_models(parsed.models)
}

fn parse_claudecode_models_yaml(yaml: &str) -> Vec<ModelSpec> {
    let parsed = serde_yaml::from_str::<ClaudeCodeModelCatalog>(yaml).unwrap_or_else(|err| {
        diagnostics::emit_warning(
            "terminal-providers",
            "model-catalog-parse-claudecode",
            &format!("Invalid claudecode model catalog YAML: {err}"),
        );
        tracing::warn!("Invalid claudecode model catalog YAML: {err}");
        ClaudeCodeModelCatalog { models: vec![] }
    });
    dedupe_valid_models(parsed.models)
}

fn codex_specs() -> Vec<ModelSpec> {
    parse_codex_models_yaml(include_str!("codex-models.yaml"))
}

fn claudecode_specs() -> Vec<ModelSpec> {
    parse_claudecode_models_yaml(include_str!("claudecode-models.yaml"))
}

pub fn codex_model_entries() -> Vec<ModelEntry> {
    codex_specs().into_iter().map(ModelEntry::from).collect()
}

pub fn codex_models() -> Vec<String> {
    codex_model_entries().into_iter().map(|m| m.name).collect()
}

pub fn claudecode_model_entries() -> Vec<ModelEntry> {
    claudecode_specs()
        .into_iter()
        .map(ModelEntry::from)
        .collect()
}

pub fn claudecode_models() -> Vec<String> {
    claudecode_model_entries()
        .into_iter()
        .map(|m| m.name)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{parse_claudecode_models_yaml, parse_codex_models_yaml};

    #[test]
    fn parses_basic_yaml_list() {
        let yaml = "models:\n  - name: gpt-5-codex\n    description: Codex default\n  - name: gpt-5\n    description: General model\n";
        assert_eq!(
            parse_codex_models_yaml(yaml),
            vec![
                super::ModelSpec {
                    name: "gpt-5-codex".to_owned(),
                    display: None,
                    description: "Codex default".to_owned(),
                    session_model: None,
                    effort_options: vec![],
                },
                super::ModelSpec {
                    name: "gpt-5".to_owned(),
                    display: None,
                    description: "General model".to_owned(),
                    session_model: None,
                    effort_options: vec![],
                }
            ]
        );
    }

    #[test]
    fn ignores_duplicates_by_name() {
        let yaml = "models:\n  - name: claude-opus-4\n    description: A\n  - name: claude-opus-4\n    description: B\n";
        assert_eq!(
            parse_claudecode_models_yaml(yaml),
            vec![super::ModelSpec {
                name: "claude-opus-4".to_owned(),
                display: None,
                description: "A".to_owned(),
                session_model: None,
                effort_options: vec![],
            }]
        );
    }

    #[test]
    fn resolves_claudecode_session_model_from_mapping() {
        let yaml = "models:\n  - name: opus-4.6\n    description: Hi\n    session_model: opus-4.6\n    effort_options: [low,medium,high]\n";
        let specs = parse_claudecode_models_yaml(yaml);
        assert_eq!(specs[0].session_model.as_deref(), Some("opus-4.6"));
        assert_eq!(specs[0].effort_options, vec!["low", "medium", "high"]);
    }
}
