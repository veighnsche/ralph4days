use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ModelEntry {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
struct ModelCatalog {
    models: Vec<ModelEntry>,
}

fn parse_models_yaml(yaml: &str) -> Vec<ModelEntry> {
    let parsed = serde_yaml::from_str::<ModelCatalog>(yaml).unwrap_or_else(|err| {
        panic!("Invalid model catalog YAML: {err}");
    });

    let mut deduped = Vec::new();
    for model in parsed.models {
        if !model.name.trim().is_empty()
            && !deduped.iter().any(|m: &ModelEntry| m.name == model.name)
        {
            deduped.push(model);
        }
    }
    deduped
}

pub fn codex_model_entries() -> Vec<ModelEntry> {
    parse_models_yaml(include_str!("codex-models.yaml"))
}

pub fn codex_models() -> Vec<String> {
    codex_model_entries().into_iter().map(|m| m.name).collect()
}

pub fn claudecode_model_entries() -> Vec<ModelEntry> {
    parse_models_yaml(include_str!("claudecode-models.yaml"))
}

pub fn claudecode_models() -> Vec<String> {
    claudecode_model_entries()
        .into_iter()
        .map(|m| m.name)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{parse_models_yaml, ModelEntry};

    #[test]
    fn parses_basic_yaml_list() {
        let yaml = "models:\n  - name: gpt-5-codex\n    description: Codex default\n  - name: gpt-5\n    description: General model\n";
        assert_eq!(
            parse_models_yaml(yaml),
            vec![
                ModelEntry {
                    name: "gpt-5-codex".to_owned(),
                    description: "Codex default".to_owned()
                },
                ModelEntry {
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
            parse_models_yaml(yaml),
            vec![ModelEntry {
                name: "claude-opus-4".to_owned(),
                description: "A".to_owned()
            }]
        );
    }
}
