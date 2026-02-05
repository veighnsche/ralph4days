use crate::Task;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

/// Project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
}

/// Manages the metadata.yaml file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataFile {
    #[serde(skip)]
    path: PathBuf,

    schema_version: String,
    pub project: ProjectMetadata,

    /// Counters track highest task ID per feature+discipline
    /// Structure: { feature: { discipline: max_id } }
    #[serde(
        rename = "_counters",
        default,
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    counters: BTreeMap<String, BTreeMap<String, u32>>,
}

impl MetadataFile {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            schema_version: "1.0".to_string(),
            project: ProjectMetadata {
                title: "Untitled Project".to_string(),
                description: None,
                created: Some(chrono::Utc::now().format("%Y-%m-%d").to_string()),
            },
            counters: BTreeMap::new(),
        }
    }

    /// Load metadata from YAML file
    pub fn load(&mut self) -> Result<(), String> {
        if !self.path.exists() {
            // File doesn't exist yet, use defaults
            return Ok(());
        }

        let content = fs::read_to_string(&self.path)
            .map_err(|e| format!("Failed to read metadata file: {}", e))?;

        let data: MetadataFile = serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse metadata YAML: {}", e))?;

        self.schema_version = data.schema_version;
        self.project = data.project;
        self.counters = data.counters;

        Ok(())
    }

    /// Save metadata to YAML file
    pub fn save(&self) -> Result<(), String> {
        let yaml = serde_yaml::to_string(self)
            .map_err(|e| format!("Failed to serialize metadata: {}", e))?;

        fs::write(&self.path, yaml).map_err(|e| format!("Failed to write metadata file: {}", e))?;

        Ok(())
    }

    /// Save to temporary file (atomic write pattern - step 1)
    pub fn save_to_temp(&self) -> Result<(), String> {
        let temp_path = self.path.with_extension("yaml.tmp");

        let yaml = serde_yaml::to_string(self)
            .map_err(|e| format!("Failed to serialize metadata: {}", e))?;

        fs::write(&temp_path, yaml)
            .map_err(|e| format!("Failed to write temp metadata file: {}", e))?;

        Ok(())
    }

    /// Commit temporary file (atomic write pattern - step 2)
    pub fn commit_temp(&self) -> Result<(), String> {
        let temp_path = self.path.with_extension("yaml.tmp");
        fs::rename(&temp_path, &self.path)
            .map_err(|e| format!("Failed to rename temp metadata file: {}", e))?;
        Ok(())
    }

    /// Rollback temporary file (cleanup on error)
    pub fn rollback_temp(&self) {
        let temp_path = self.path.with_extension("yaml.tmp");
        let _ = fs::remove_file(&temp_path); // Ignore errors
    }

    /// Rebuild counters from task list
    /// Tracks the highest task ID for each feature+discipline combination
    pub fn rebuild_counters(&mut self, tasks: &[Task]) {
        self.counters.clear();

        for task in tasks {
            let feature_counters = self
                .counters
                .entry(task.feature.clone())
                .or_insert_with(BTreeMap::new);

            let current_max = feature_counters.get(&task.discipline).copied().unwrap_or(0);
            if task.id > current_max {
                feature_counters.insert(task.discipline.clone(), task.id);
            }
        }
    }

    /// Get the next available task ID (global max + 1)
    pub fn get_next_id(&self, tasks: &[Task]) -> u32 {
        tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1
    }

    /// Get project info
    pub fn get_project_info(&self) -> &ProjectMetadata {
        &self.project
    }

    /// Get counters (read-only)
    pub fn get_counters(&self) -> &BTreeMap<String, BTreeMap<String, u32>> {
        &self.counters
    }
}
