use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalPrompts {
    pub positive: String,
    pub negative: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisciplinePrompt {
    pub subject: String,
    pub accent_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackPrompts {
    pub name: String,
    pub description: String,
    pub positive: String,
    pub negative: String,
    pub disciplines: HashMap<String, DisciplinePrompt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagePromptsConfig {
    pub global: GlobalPrompts,
    pub stacks: HashMap<u8, StackPrompts>,
}

impl ImagePromptsConfig {
    pub fn load() -> Result<Self, String> {
        let yaml_content = include_str!("defaults/image_prompts.yaml");
        serde_yaml::from_str(yaml_content)
            .map_err(|e| format!("Failed to parse image_prompts.yaml: {e}"))
    }

    /// Build a complete prompt for a specific discipline
    /// Returns (positive_prompt, negative_prompt)
    pub fn build_prompt(
        &self,
        stack_id: u8,
        discipline_name: &str,
    ) -> Result<(String, String), String> {
        let stack = self
            .stacks
            .get(&stack_id)
            .ok_or_else(|| format!("Stack {stack_id} not found"))?;

        let discipline = stack.disciplines.get(discipline_name).ok_or_else(|| {
            format!("Discipline '{discipline_name}' not found in stack {stack_id}")
        })?;

        let positive = format!(
            "{}\n\n{}\n\nSubject: {}. Accent color: {}.",
            self.global.positive.trim(),
            stack.positive.trim(),
            discipline.subject.trim(),
            discipline.accent_color
        );

        let negative = format!("{}, {}", stack.negative.trim(), self.global.negative.trim());

        Ok((positive, negative))
    }

    /// Get all available disciplines for a stack
    pub fn get_stack_disciplines(&self, stack_id: u8) -> Result<Vec<String>, String> {
        let stack = self
            .stacks
            .get(&stack_id)
            .ok_or_else(|| format!("Stack {stack_id} not found"))?;

        Ok(stack.disciplines.keys().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_image_prompts() {
        let config = ImagePromptsConfig::load().expect("Failed to load image prompts");

        assert!(!config.global.positive.is_empty());
        assert!(!config.global.negative.is_empty());

        assert_eq!(config.stacks.len(), 4);

        for stack_id in 1..=4 {
            assert!(config.stacks.contains_key(&stack_id));
        }
    }

    #[test]
    fn test_build_prompt_stack_1() {
        let config = ImagePromptsConfig::load().expect("Failed to load image prompts");

        let (positive, negative) = config
            .build_prompt(1, "implementation")
            .expect("Failed to build prompt");

        assert!(positive.contains("Vertical 9:16"));
        assert!(positive.contains("server blade"));
        assert!(positive.contains("#3b82f6"));

        assert!(negative.contains("people"));
        assert!(negative.contains("blurry"));
    }

    #[test]
    fn test_get_stack_disciplines() {
        let config = ImagePromptsConfig::load().expect("Failed to load image prompts");

        let disciplines = config
            .get_stack_disciplines(2)
            .expect("Failed to get disciplines");

        assert_eq!(disciplines.len(), 8);
        assert!(disciplines.contains(&"frontend".to_owned()));
        assert!(disciplines.contains(&"backend".to_owned()));
    }
}
