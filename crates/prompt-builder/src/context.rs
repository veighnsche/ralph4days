use sqlite_db::{Discipline, Feature, ProjectMetadata, Task};
use std::collections::HashMap;

/// Everything the caller knows. Pure data — no I/O.
/// The caller pre-reads files and pre-queries the database.
pub struct PromptContext {
    // Database state (pre-queried by caller)
    pub features: Vec<Feature>,
    pub tasks: Vec<Task>,
    pub disciplines: Vec<Discipline>,
    pub metadata: ProjectMetadata,

    // Pre-read file contents (caller reads context_files + knowledge_paths)
    // Key: relative path, Value: file content
    pub file_contents: HashMap<String, String>,

    // State files
    pub progress_txt: Option<String>,
    pub learnings_txt: Option<String>,
    pub claude_ralph_md: Option<String>,

    // Paths (for MCP scripts to reference sqlite3)
    pub project_path: String,
    pub db_path: String,
    pub script_dir: String,

    // Prompt-specific input
    pub user_input: Option<String>,
    pub target_task_id: Option<u32>,
    pub target_feature: Option<String>,
}

impl PromptContext {
    /// Find the task targeted by this prompt.
    pub fn target_task(&self) -> Option<&Task> {
        let id = self.target_task_id?;
        self.tasks.iter().find(|t| t.id == id)
    }

    /// Find the feature for the target task.
    pub fn target_task_feature(&self) -> Option<&Feature> {
        let task = self.target_task()?;
        self.features.iter().find(|f| f.name == task.feature)
    }

    /// Find the discipline for the target task.
    pub fn target_task_discipline(&self) -> Option<&Discipline> {
        let task = self.target_task()?;
        self.disciplines.iter().find(|d| d.name == task.discipline)
    }

    /// Find a feature by name.
    pub fn feature_by_name(&self, name: &str) -> Option<&Feature> {
        self.features.iter().find(|f| f.name == name)
    }
}

#[cfg(test)]
pub fn test_context() -> PromptContext {
    PromptContext {
        features: vec![],
        tasks: vec![],
        disciplines: vec![],
        metadata: ProjectMetadata {
            title: "Test Project".to_string(),
            description: Some("A test project".to_string()),
            created: None,
        },
        file_contents: HashMap::new(),
        progress_txt: None,
        learnings_txt: None,
        claude_ralph_md: None,
        project_path: "/tmp/test-project".to_string(),
        db_path: "/tmp/test-project/.ralph/db/ralph.db".to_string(),
        script_dir: "/tmp/ralph-mcp".to_string(),
        user_input: None,
        target_task_id: None,
        target_feature: None,
    }
}
