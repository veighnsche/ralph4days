use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ModelEntry {
    pub name: String,
    pub description: String,
    pub session_model: Option<String>,
    pub effort_options: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CodexModelCatalog {
    models: Vec<CodexModelSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct CodexModelSpec {
    name: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeCodeModelCatalog {
    models: Vec<ClaudeCodeModelSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct ClaudeCodeModelSpec {
    name: String,
    description: String,
    #[serde(default)]
    session_model: Option<String>,
    #[serde(default)]
    effort_options: Vec<String>,
}

fn parse_codex_models_yaml(yaml: &str) -> Vec<CodexModelSpec> {
    let parsed = serde_yaml::from_str::<CodexModelCatalog>(yaml).unwrap_or_else(|err| {
        panic!("Invalid model catalog YAML: {err}");
    });

    let mut deduped = Vec::new();
    for model in parsed.models {
        if !model.name.trim().is_empty()
            && !deduped
                .iter()
                .any(|m: &CodexModelSpec| m.name == model.name)
        {
            deduped.push(model);
        }
    }
    deduped
}

fn parse_claudecode_models_yaml(yaml: &str) -> Vec<ClaudeCodeModelSpec> {
    let parsed = serde_yaml::from_str::<ClaudeCodeModelCatalog>(yaml).unwrap_or_else(|err| {
        panic!("Invalid model catalog YAML: {err}");
    });

    let mut deduped = Vec::new();
    for model in parsed.models {
        if !model.name.trim().is_empty()
            && !deduped
                .iter()
                .any(|m: &ClaudeCodeModelSpec| m.name == model.name)
        {
            deduped.push(model);
        }
    }
    deduped
}

fn codex_specs() -> Vec<CodexModelSpec> {
    parse_codex_models_yaml(include_str!("codex-models.yaml"))
}

fn claudecode_specs() -> Vec<ClaudeCodeModelSpec> {
    parse_claudecode_models_yaml(include_str!("claudecode-models.yaml"))
}

pub fn codex_model_entries() -> Vec<ModelEntry> {
    codex_specs()
        .into_iter()
        .map(|m| ModelEntry {
            name: m.name,
            description: m.description,
            session_model: None,
            effort_options: vec![],
        })
        .collect()
}

pub fn codex_models() -> Vec<String> {
    codex_model_entries().into_iter().map(|m| m.name).collect()
}

pub fn claudecode_model_entries() -> Vec<ModelEntry> {
    claudecode_specs()
        .into_iter()
        .map(|m| ModelEntry {
            name: m.name,
            description: m.description,
            session_model: m.session_model,
            effort_options: m.effort_options,
        })
        .collect()
}

pub fn claudecode_models() -> Vec<String> {
    claudecode_model_entries()
        .into_iter()
        .map(|m| m.name)
        .collect()
}

pub fn resolve_codex_session_model(selected_model: &str) -> String {
    let trimmed = selected_model.trim();
    if trimmed.is_empty() {
        return trimmed.to_owned();
    }
    codex_specs()
        .into_iter()
        .find(|m| m.name == trimmed)
        .map_or_else(|| trimmed.to_owned(), |m| m.name)
}

pub fn resolve_claudecode_session_model(selected_model: &str) -> String {
    let trimmed = selected_model.trim();
    if trimmed.is_empty() {
        return trimmed.to_owned();
    }
    claudecode_specs()
        .into_iter()
        .find(|m| m.name == trimmed)
        .map_or_else(|| trimmed.to_owned(), |m| m.session_model.unwrap_or(m.name))
}

#[cfg(test)]
mod tests {
    use super::{
        parse_claudecode_models_yaml, parse_codex_models_yaml, resolve_claudecode_session_model,
        resolve_codex_session_model,
    };

    #[test]
    fn parses_basic_yaml_list() {
        let yaml = "models:\n  - name: gpt-5-codex\n    description: Codex default\n  - name: gpt-5\n    description: General model\n";
        assert_eq!(
            parse_codex_models_yaml(yaml),
            vec![
                super::CodexModelSpec {
                    name: "gpt-5-codex".to_owned(),
                    description: "Codex default".to_owned()
                },
                super::CodexModelSpec {
                    name: "gpt-5".to_owned(),
                    description: "General model".to_owned()
                }
            ]
        );
    }

    #[test]
    fn ignores_duplicates_by_name() {
        let yaml = "models:\n  - name: claude-opus-4\n    description: A\n  - name: claude-opus-4\n    description: B\n";
        assert_eq!(
            parse_claudecode_models_yaml(yaml),
            vec![super::ClaudeCodeModelSpec {
                name: "claude-opus-4".to_owned(),
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

    #[test]
    fn resolve_functions_fall_back_to_selected_name() {
        assert_eq!(resolve_codex_session_model("custom-codex"), "custom-codex");
        assert_eq!(
            resolve_claudecode_session_model("custom-claude"),
            "custom-claude"
        );
    }
}
