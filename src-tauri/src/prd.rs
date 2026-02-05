use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Done,
    Blocked,
    Skipped,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PRD {
    pub schema_version: String,
    pub project: ProjectMetadata,
    pub tasks: Vec<Task>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub _counters: HashMap<String, HashMap<String, u32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub feature: String,
    pub discipline: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: TaskStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<Priority>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub acceptance_criteria: Vec<String>,
}

impl PRD {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read PRD file: {}", e))?;

        let mut prd: PRD =
            serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse YAML: {}", e))?;

        // Validate all tasks have valid disciplines
        for task in &prd.tasks {
            Self::validate_discipline(&task.discipline)?;
        }

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
                .or_insert_with(HashMap::new);

            let counter = feature_counters
                .entry(task.discipline.clone())
                .or_insert(0);
            *counter = (*counter).max(task.id);
        }
    }

    pub fn get_next_id(&self) -> u32 {
        self.tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1
    }

    pub fn validate_discipline(discipline: &str) -> Result<(), String> {
        const VALID: &[&str] = &[
            "frontend",
            "backend",
            "database",
            "testing",
            "infra",
            "security",
            "docs",
            "design",
            "promo",
            "api",
        ];

        if VALID.contains(&discipline) {
            Ok(())
        } else {
            Err(format!(
                "Invalid discipline '{}'. Must be one of: {}",
                discipline,
                VALID.join(", ")
            ))
        }
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
            _counters: HashMap::new(),
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
        });

        // Next ID should be 6
        assert_eq!(prd.get_next_id(), 6);
    }

    #[test]
    fn test_validate_discipline() {
        assert!(PRD::validate_discipline("frontend").is_ok());
        assert!(PRD::validate_discipline("backend").is_ok());
        assert!(PRD::validate_discipline("infra").is_ok());
        assert!(PRD::validate_discipline("docs").is_ok());
        assert!(PRD::validate_discipline("promo").is_ok());

        assert!(PRD::validate_discipline("infrastructure").is_err());
        assert!(PRD::validate_discipline("documentation").is_err());
        assert!(PRD::validate_discipline("marketing").is_err());
        assert!(PRD::validate_discipline("invalid").is_err());
        assert!(PRD::validate_discipline("Frontend").is_err()); // uppercase
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
                },
            ],
            _counters: HashMap::new(),
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

    #[test]
    fn test_rejects_invalid_disciplines() {
        let yaml_content = r#"
schema_version: "1.0"
project:
  title: "Test Project"
tasks:
  - id: 1
    feature: "auth"
    discipline: "infrastructure"
    title: "Old discipline name"
    status: "pending"
"#;

        // Write to temp file
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_invalid_discipline.yaml");
        std::fs::write(&test_file, yaml_content).unwrap();

        // Attempt to load - should FAIL HARD
        let result = PRD::from_file(&test_file);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Invalid discipline"));
        assert!(error.contains("infrastructure"));

        // Cleanup
        std::fs::remove_file(&test_file).ok();
    }
}
