use super::entity::{EntityFile, YamlEntity};
use serde::{Deserialize, Serialize};

/// Feature definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub name: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
}

// Implement YamlEntity trait for Feature
impl YamlEntity for Feature {
    const COLLECTION_NAME: &'static str = "features";
}

/// Manages the features.yaml file
pub type FeaturesFile = EntityFile<Feature>;

/// Feature-specific methods
impl FeaturesFile {
    /// Auto-populate: Create feature if it doesn't exist
    pub fn ensure_feature_exists(&mut self, name: &str) -> Result<(), String> {
        if !self.get_all().iter().any(|f| f.name == name) {
            self.add(Feature {
                name: name.to_string(),
                display_name: Self::name_to_display_name(name),
                description: None,
                created: Some(chrono::Utc::now().format("%Y-%m-%d").to_string()),
            });
        }
        Ok(())
    }

    /// Convert feature name to display name
    /// "auth" -> "Auth"
    /// "user-profile" -> "User Profile"
    fn name_to_display_name(name: &str) -> String {
        name.split('-')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}
