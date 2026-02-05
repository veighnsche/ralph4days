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
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: TaskStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<Priority>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,
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

        // CRITICAL: Validate ALL task IDs - FAIL HARD on invalid format
        for task in &prd.tasks {
            Self::validate_task_id(&task.id).map_err(|e| {
                format!(
                    "Invalid task ID '{}' in PRD. All task IDs must use 3-tier format (feature/discipline/number). {}",
                    task.id, e
                )
            })?;
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

    // CRITICAL BUG FIX: Initialize counter from existing tasks
    pub fn rebuild_counters(&mut self) {
        self._counters.clear();

        for task in &self.tasks {
            if let Some((feature, discipline, num)) = Self::parse_tier_id(&task.id) {
                let feature_counters = self._counters.entry(feature).or_insert_with(HashMap::new);

                let counter = feature_counters.entry(discipline).or_insert(0);
                *counter = (*counter).max(num);
            }
        }
    }

    fn parse_tier_id(id: &str) -> Option<(String, String, u32)> {
        let parts: Vec<&str> = id.split('/').collect();
        if parts.len() == 3 {
            if let Ok(num) = parts[2].parse::<u32>() {
                return Some((parts[0].to_string(), parts[1].to_string(), num));
            }
        }
        None
    }

    pub fn generate_task_id(&mut self, feature: &str, discipline: &str) -> Result<String, String> {
        // Validate inputs first
        Self::validate_feature_name(feature)?;
        Self::validate_discipline(discipline)?;

        let feature_counters = self
            ._counters
            .entry(feature.to_string())
            .or_insert_with(HashMap::new);

        let counter = feature_counters.entry(discipline.to_string()).or_insert(0);

        *counter += 1;
        let new_id = format!("{}/{}/{}", feature, discipline, counter);

        // CRITICAL: Check for duplicate IDs (safety check)
        if self.tasks.iter().any(|t| t.id == new_id) {
            return Err(format!("Duplicate task ID would be created: {}", new_id));
        }

        Ok(new_id)
    }

    pub fn validate_task_id(id: &str) -> Result<(), String> {
        use regex::Regex;
        use std::sync::OnceLock;

        static TIER_REGEX: OnceLock<Regex> = OnceLock::new();
        let re = TIER_REGEX.get_or_init(|| Regex::new(r"^[a-z0-9-]+/[a-z]+/\d+$").unwrap());

        if re.is_match(id) {
            Ok(())
        } else {
            Err(format!("Invalid task ID format: {}", id))
        }
    }

    fn validate_feature_name(feature: &str) -> Result<(), String> {
        if feature.is_empty() {
            return Err("Feature name cannot be empty".to_string());
        }
        if feature.contains('/') {
            return Err("Feature name cannot contain slashes".to_string());
        }
        if !feature
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err("Feature must be lowercase with hyphens only".to_string());
        }
        Ok(())
    }

    fn validate_discipline(discipline: &str) -> Result<(), String> {
        const VALID: &[&str] = &[
            "frontend",
            "backend",
            "database",
            "testing",
            "infrastructure",
            "security",
            "documentation",
            "design",
            "marketing",
            "api",
        ];

        if VALID.contains(&discipline) {
            Ok(())
        } else {
            Err(format!("Invalid discipline: {}", discipline))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_task_id() {
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
        let id1 = prd.generate_task_id("auth", "frontend").unwrap();
        assert_eq!(id1, "auth/frontend/1");

        // Second ID should be 2
        let id2 = prd.generate_task_id("auth", "frontend").unwrap();
        assert_eq!(id2, "auth/frontend/2");

        // Different discipline should start at 1
        let id3 = prd.generate_task_id("auth", "backend").unwrap();
        assert_eq!(id3, "auth/backend/1");

        // Different feature should start at 1
        let id4 = prd.generate_task_id("search", "frontend").unwrap();
        assert_eq!(id4, "search/frontend/1");
    }

    #[test]
    fn test_validate_task_id() {
        assert!(PRD::validate_task_id("auth/frontend/1").is_ok());
        assert!(PRD::validate_task_id("search-users/backend/42").is_ok());
        assert!(PRD::validate_task_id("profile/api/100").is_ok());

        assert!(PRD::validate_task_id("Auth/frontend/1").is_err()); // uppercase
        assert!(PRD::validate_task_id("auth/Frontend/1").is_err()); // uppercase
        assert!(PRD::validate_task_id("auth/frontend").is_err()); // missing number
        assert!(PRD::validate_task_id("auth/frontend/").is_err()); // empty number
        assert!(PRD::validate_task_id("auth/1").is_err()); // missing discipline
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
                    id: "auth/frontend/1".to_string(),
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
                    id: "auth/frontend/2".to_string(),
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
                    id: "auth/backend/1".to_string(),
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

        // Next IDs should continue from existing tasks
        let next_id = prd.generate_task_id("auth", "frontend").unwrap();
        assert_eq!(next_id, "auth/frontend/3");

        let next_id = prd.generate_task_id("auth", "backend").unwrap();
        assert_eq!(next_id, "auth/backend/2");
    }

    #[test]
    fn test_rejects_old_style_ids() {
        // Create a PRD YAML string with old-style IDs
        let yaml_content = r#"
schema_version: "1.0"
project:
  title: "Test Project"
tasks:
  - id: "task-001"
    title: "Old style ID"
    status: "pending"
"#;

        // Write to temp file
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_old_style.yaml");
        std::fs::write(&test_file, yaml_content).unwrap();

        // Attempt to load - should FAIL HARD
        let result = PRD::from_file(&test_file);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Invalid task ID"));
        assert!(error.contains("task-001"));
        assert!(error.contains("3-tier format"));

        // Cleanup
        std::fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_accepts_only_3tier_ids() {
        // Create a PRD YAML string with valid 3-tier IDs
        let yaml_content = r#"
schema_version: "1.0"
project:
  title: "Test Project"
tasks:
  - id: "auth/frontend/1"
    title: "Valid 3-tier ID"
    status: "pending"
  - id: "search/backend/2"
    title: "Another valid ID"
    status: "done"
"#;

        // Write to temp file
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_3tier.yaml");
        std::fs::write(&test_file, yaml_content).unwrap();

        // Should load successfully
        let result = PRD::from_file(&test_file);

        assert!(result.is_ok());
        let prd = result.unwrap();
        assert_eq!(prd.tasks.len(), 2);
        assert_eq!(prd.tasks[0].id, "auth/frontend/1");
        assert_eq!(prd.tasks[1].id, "search/backend/2");

        // Cleanup
        std::fs::remove_file(&test_file).ok();
    }
}
