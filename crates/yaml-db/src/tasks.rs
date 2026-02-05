use super::entity::{EntityFile, YamlEntity};
use crate::Task;

// Implement YamlEntity trait for Task
impl YamlEntity for Task {
    const COLLECTION_NAME: &'static str = "tasks";
}

/// Manages the tasks.yaml file
pub type TasksFile = EntityFile<Task>;

/// Task-specific methods
impl TasksFile {
    /// Add a task to the list
    pub fn add_task(&mut self, task: Task) {
        self.add(task);
    }

    /// Check if a task with given ID exists
    pub fn has_task(&self, id: u32) -> bool {
        self.get_all().iter().any(|t| t.id == id)
    }
}
