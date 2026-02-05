use crate::prd::Task;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Manages the tasks.yaml file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksFile {
    #[serde(skip)]
    path: PathBuf,

    tasks: Vec<Task>,
}

impl TasksFile {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            tasks: Vec::new(),
        }
    }

    /// Load tasks from YAML file
    pub fn load(&mut self) -> Result<(), String> {
        if !self.path.exists() {
            // File doesn't exist yet, start with empty list
            self.tasks = Vec::new();
            return Ok(());
        }

        let content = fs::read_to_string(&self.path)
            .map_err(|e| format!("Failed to read tasks file: {}", e))?;

        let data: TasksData = serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse tasks YAML: {}", e))?;

        self.tasks = data.tasks;
        Ok(())
    }

    /// Save tasks to YAML file
    pub fn save(&self) -> Result<(), String> {
        let data = TasksData {
            tasks: self.tasks.clone(),
        };

        let yaml = serde_yaml::to_string(&data)
            .map_err(|e| format!("Failed to serialize tasks: {}", e))?;

        fs::write(&self.path, yaml).map_err(|e| format!("Failed to write tasks file: {}", e))?;

        Ok(())
    }

    /// Save to temporary file (atomic write pattern - step 1)
    pub fn save_to_temp(&self) -> Result<(), String> {
        let temp_path = self.path.with_extension("yaml.tmp");

        let data = TasksData {
            tasks: self.tasks.clone(),
        };

        let yaml = serde_yaml::to_string(&data)
            .map_err(|e| format!("Failed to serialize tasks: {}", e))?;

        fs::write(&temp_path, yaml)
            .map_err(|e| format!("Failed to write temp tasks file: {}", e))?;

        Ok(())
    }

    /// Commit temporary file (atomic write pattern - step 2)
    pub fn commit_temp(&self) -> Result<(), String> {
        let temp_path = self.path.with_extension("yaml.tmp");
        fs::rename(&temp_path, &self.path)
            .map_err(|e| format!("Failed to rename temp tasks file: {}", e))?;
        Ok(())
    }

    /// Rollback temporary file (cleanup on error)
    pub fn rollback_temp(&self) {
        let temp_path = self.path.with_extension("yaml.tmp");
        let _ = fs::remove_file(&temp_path); // Ignore errors
    }

    /// Add a task to the list
    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    /// Get all tasks
    pub fn get_all(&self) -> &[Task] {
        &self.tasks
    }

    /// Check if a task with given ID exists
    pub fn has_task(&self, id: u32) -> bool {
        self.tasks.iter().any(|t| t.id == id)
    }
}

/// YAML structure for tasks.yaml file
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TasksData {
    tasks: Vec<Task>,
}
