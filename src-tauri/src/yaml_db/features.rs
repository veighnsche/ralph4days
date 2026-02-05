use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Feature definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub name: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created: String,
}

/// Manages the features.yaml file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesFile {
    #[serde(skip)]
    path: PathBuf,

    features: Vec<Feature>,
}

impl FeaturesFile {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            features: Vec::new(),
        }
    }

    /// Load features from YAML file
    pub fn load(&mut self) -> Result<(), String> {
        if !self.path.exists() {
            // File doesn't exist yet, start with empty list
            self.features = Vec::new();
            return Ok(());
        }

        let content = fs::read_to_string(&self.path)
            .map_err(|e| format!("Failed to read features file: {}", e))?;

        let data: FeaturesData = serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse features YAML: {}", e))?;

        self.features = data.features;
        Ok(())
    }

    /// Save features to YAML file
    pub fn save(&self) -> Result<(), String> {
        let data = FeaturesData {
            features: self.features.clone(),
        };

        let yaml = serde_yaml::to_string(&data)
            .map_err(|e| format!("Failed to serialize features: {}", e))?;

        fs::write(&self.path, yaml)
            .map_err(|e| format!("Failed to write features file: {}", e))?;

        Ok(())
    }

    /// Save to temporary file (atomic write pattern - step 1)
    pub fn save_to_temp(&self) -> Result<(), String> {
        let temp_path = self.path.with_extension("yaml.tmp");

        let data = FeaturesData {
            features: self.features.clone(),
        };

        let yaml = serde_yaml::to_string(&data)
            .map_err(|e| format!("Failed to serialize features: {}", e))?;

        fs::write(&temp_path, yaml)
            .map_err(|e| format!("Failed to write temp features file: {}", e))?;

        Ok(())
    }

    /// Commit temporary file (atomic write pattern - step 2)
    pub fn commit_temp(&self) -> Result<(), String> {
        let temp_path = self.path.with_extension("yaml.tmp");
        fs::rename(&temp_path, &self.path)
            .map_err(|e| format!("Failed to rename temp features file: {}", e))?;
        Ok(())
    }

    /// Rollback temporary file (cleanup on error)
    pub fn rollback_temp(&self) {
        let temp_path = self.path.with_extension("yaml.tmp");
        let _ = fs::remove_file(&temp_path); // Ignore errors
    }

    /// Get all features
    pub fn get_all(&self) -> &[Feature] {
        &self.features
    }

    /// Auto-populate: Create feature if it doesn't exist
    pub fn ensure_feature_exists(&mut self, name: &str) -> Result<(), String> {
        if !self.features.iter().any(|f| f.name == name) {
            self.features.push(Feature {
                name: name.to_string(),
                display_name: Self::name_to_display_name(name),
                description: None,
                created: chrono::Utc::now().format("%Y-%m-%d").to_string(),
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

/// YAML structure for features.yaml file
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FeaturesData {
    features: Vec<Feature>,
}
