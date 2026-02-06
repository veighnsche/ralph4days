use super::entity::{EntityFile, YamlEntity};
use serde::{Deserialize, Serialize};

/// Feature definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub name: String,
    pub display_name: String,
    #[serde(default)]
    pub acronym: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    // Knowledge context
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub knowledge_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_files: Vec<String>,
}

// Implement YamlEntity trait for Feature
impl YamlEntity for Feature {
    const COLLECTION_NAME: &'static str = "features";
}

/// Manages the features.yaml file
pub type FeaturesFile = EntityFile<Feature>;

/// Feature-specific methods
impl FeaturesFile {
    /// Validate features collection for uniqueness constraints (called on load)
    pub fn validate(&self) -> Result<(), String> {
        let mut seen_names = std::collections::HashSet::new();
        let mut seen_acronyms = std::collections::HashSet::new();

        for feature in self.get_all() {
            // Check name uniqueness
            if !seen_names.insert(&feature.name) {
                return Err(format!(
                    "Duplicate feature name found: '{}'. Each feature must have a unique name.",
                    feature.name
                ));
            }

            // Only validate non-empty acronyms (migration might have empty ones temporarily)
            if !feature.acronym.is_empty() {
                // Check acronym uniqueness
                if !seen_acronyms.insert(&feature.acronym) {
                    return Err(format!(
                        "Duplicate feature acronym found: '{}'. Each feature must have a unique acronym.",
                        feature.acronym
                    ));
                }

                // Validate acronym format
                crate::acronym::validate_acronym_format(&feature.acronym)?;
            }
        }

        Ok(())
    }

    /// Validate acronym is unique within features collection (used during creation)
    fn validate_acronym(&self, acronym: &str, feature_name: &str) -> Result<(), String> {
        crate::acronym::validate_acronym_format(acronym)?;

        if self
            .get_all()
            .iter()
            .any(|f| f.acronym == acronym && f.name != feature_name)
        {
            return Err(format!(
                "Acronym '{}' is already used by another feature. Please choose a unique acronym.",
                acronym
            ));
        }

        Ok(())
    }

    /// Convert feature name to display name
    /// "auth" -> "Auth"
    /// "user-profile" -> "User Profile"
    /// "user_profile" -> "User Profile"
    /// "user-profile_settings" -> "User Profile Settings"
    fn name_to_display_name(name: &str) -> String {
        name.split(|c| c == '-' || c == '_')
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
