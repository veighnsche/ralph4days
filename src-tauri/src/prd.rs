use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

// Re-export types from yaml-db crate
pub use yaml_db::{Priority, ProjectMetadata, Task, TaskStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PRD {
    pub schema_version: String,
    pub project: ProjectMetadata,
    pub tasks: Vec<Task>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub _counters: BTreeMap<String, BTreeMap<String, u32>>,
}

impl PRD {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read PRD file: {}", e))?;

        let mut prd: PRD =
            serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse YAML: {}", e))?;

        // Rebuild counters from existing tasks
        prd.rebuild_counters();

        Ok(prd)
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let yaml =
            serde_yaml::to_string(self).map_err(|e| format!("Failed to serialize YAML: {}", e))?;

        fs::write(path, yaml).map_err(|e| format!("Failed to write PRD file: {}", e))
    }

    // Rebuild counters from existing tasks
    pub fn rebuild_counters(&mut self) {
        self._counters.clear();

        for task in &self.tasks {
            let feature_counters = self
                ._counters
                .entry(task.feature.clone())
                .or_insert_with(BTreeMap::new);

            let counter = feature_counters.entry(task.discipline.clone()).or_insert(0);
            *counter = (*counter).max(task.id);
        }
    }

    pub fn get_next_id(&self) -> u32 {
        self.tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_next_id() {
        let mut prd = PRD {
            schema_version: "1.0".to_string(),
            project: ProjectMetadata {
                title: "Test".to_string(),
                description: None,
                created: None,
            },
            tasks: Vec::new(),
            _counters: BTreeMap::new(),
        };

        // First ID should be 1
        assert_eq!(prd.get_next_id(), 1);

        // Add a task with ID 1
        prd.tasks.push(Task {
            id: 1,
            feature: "auth".to_string(),
            discipline: "frontend".to_string(),
            title: "Task 1".to_string(),
            description: None,
            status: TaskStatus::Pending,
            priority: None,
            tags: Vec::new(),
            depends_on: Vec::new(),
            blocked_by: None,
            created: None,
            updated: None,
            completed: None,
            acceptance_criteria: Vec::new(),
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
            attempt_notes: vec![],
        });

        // Next ID should be 2
        assert_eq!(prd.get_next_id(), 2);

        // Add a task with ID 5 (skipping some IDs)
        prd.tasks.push(Task {
            id: 5,
            feature: "search".to_string(),
            discipline: "backend".to_string(),
            title: "Task 5".to_string(),
            description: None,
            status: TaskStatus::Pending,
            priority: None,
            tags: Vec::new(),
            depends_on: Vec::new(),
            blocked_by: None,
            created: None,
            updated: None,
            completed: None,
            acceptance_criteria: Vec::new(),
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
            attempt_notes: vec![],
        });

        // Next ID should be 6
        assert_eq!(prd.get_next_id(), 6);
    }

    #[test]
    fn test_counter_persistence() {
        let mut prd = PRD {
            schema_version: "1.0".to_string(),
            project: ProjectMetadata {
                title: "Test".to_string(),
                description: None,
                created: None,
            },
            tasks: vec![
                Task {
                    id: 1,
                    feature: "auth".to_string(),
                    discipline: "frontend".to_string(),
                    title: "Task 1".to_string(),
                    description: None,
                    status: TaskStatus::Done,
                    priority: None,
                    tags: Vec::new(),
                    depends_on: Vec::new(),
                    blocked_by: None,
                    created: None,
                    updated: None,
                    completed: None,
                    acceptance_criteria: Vec::new(),
                    context_files: vec![],
                    output_artifacts: vec![],
                    hints: None,
                    estimated_turns: None,
                    provenance: None,
                    attempt_notes: vec![],
                },
                Task {
                    id: 2,
                    feature: "auth".to_string(),
                    discipline: "frontend".to_string(),
                    title: "Task 2".to_string(),
                    description: None,
                    status: TaskStatus::Pending,
                    priority: None,
                    tags: Vec::new(),
                    depends_on: Vec::new(),
                    blocked_by: None,
                    created: None,
                    updated: None,
                    completed: None,
                    acceptance_criteria: Vec::new(),
                    context_files: vec![],
                    output_artifacts: vec![],
                    hints: None,
                    estimated_turns: None,
                    provenance: None,
                    attempt_notes: vec![],
                },
                Task {
                    id: 3,
                    feature: "auth".to_string(),
                    discipline: "backend".to_string(),
                    title: "Task 3".to_string(),
                    description: None,
                    status: TaskStatus::Pending,
                    priority: None,
                    tags: Vec::new(),
                    depends_on: Vec::new(),
                    blocked_by: None,
                    created: None,
                    updated: None,
                    completed: None,
                    acceptance_criteria: Vec::new(),
                    context_files: vec![],
                    output_artifacts: vec![],
                    hints: None,
                    estimated_turns: None,
                    provenance: None,
                    attempt_notes: vec![],
                },
            ],
            _counters: BTreeMap::new(),
        };

        // Rebuild counters from existing tasks
        prd.rebuild_counters();

        // Check counters were built correctly
        assert_eq!(prd._counters.get("auth").unwrap().get("frontend"), Some(&2));
        assert_eq!(prd._counters.get("auth").unwrap().get("backend"), Some(&3));
    }

    #[test]
    fn test_load_prd_with_new_format() {
        let yaml_content = r#"
schema_version: "1.0"
project:
  title: "Test Project"
tasks:
  - id: 1
    feature: "auth"
    discipline: "frontend"
    title: "Login form"
    status: "pending"
  - id: 2
    feature: "search"
    discipline: "backend"
    title: "Search API"
    status: "done"
"#;

        // Write to temp file
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_new_format.yaml");
        std::fs::write(&test_file, yaml_content).unwrap();

        // Should load successfully
        let result = PRD::from_file(&test_file);

        assert!(result.is_ok());
        let prd = result.unwrap();
        assert_eq!(prd.tasks.len(), 2);
        assert_eq!(prd.tasks[0].id, 1);
        assert_eq!(prd.tasks[0].feature, "auth");
        assert_eq!(prd.tasks[0].discipline, "frontend");
        assert_eq!(prd.tasks[1].id, 2);
        assert_eq!(prd.tasks[1].feature, "search");
        assert_eq!(prd.tasks[1].discipline, "backend");

        // Cleanup
        std::fs::remove_file(&test_file).ok();
    }
}
