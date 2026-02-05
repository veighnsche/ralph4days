use super::{DisciplinesFile, FeaturesFile, MetadataFile, TasksFile};
use crate::{Priority, Task, TaskStatus};
use fs2::FileExt;
use std::fs::{self, File};
use std::path::PathBuf;

/// Input for creating a new task (before ID assignment)
pub struct TaskInput {
    pub feature: String,
    pub discipline: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<Priority>,
    pub tags: Vec<String>,
    pub depends_on: Vec<u32>,
    pub acceptance_criteria: Option<Vec<String>>,

    // Required acronyms for auto-created features/disciplines
    pub feature_acronym: String,
    pub discipline_acronym: String,
}

/// Multi-file YAML database coordinator
pub struct YamlDatabase {
    _base_path: PathBuf,
    lock_file: PathBuf,
    tasks: TasksFile,
    pub(crate) features: FeaturesFile,
    pub(crate) disciplines: DisciplinesFile,
    metadata: MetadataFile,
}

impl YamlDatabase {
    /// Create new database from .ralph/db/ path
    pub fn from_path(path: PathBuf) -> Result<Self, String> {
        // Ensure db directory exists
        if !path.exists() {
            fs::create_dir_all(&path)
                .map_err(|e| format!("Failed to create db directory: {}", e))?;
        }

        let lock_file = path.join(".lock");
        let tasks = TasksFile::new(path.join("tasks.yaml"));
        let features = FeaturesFile::new(path.join("features.yaml"));
        let disciplines = DisciplinesFile::new(path.join("disciplines.yaml"));
        let metadata = MetadataFile::new(path.join("metadata.yaml"));

        let mut db = Self {
            _base_path: path,
            lock_file,
            tasks,
            features,
            disciplines,
            metadata,
        };

        // Load all files
        db.load_all()?;

        // Validate collections before proceeding
        db.features.validate()?;
        db.disciplines.validate()?;

        // Run migration if needed (comes after validation)
        crate::migration::migrate_acronyms_if_needed(&mut db)?;

        // Validate again after migration to ensure generated acronyms are valid
        db.features.validate()?;
        db.disciplines.validate()?;

        Ok(db)
    }

    /// Load all database files from disk
    pub fn load_all(&mut self) -> Result<(), String> {
        self.tasks.load()?;
        self.features.load()?;
        self.disciplines.load_with_defaults()?;
        self.metadata.load()?;
        Ok(())
    }

    /// Save all database files atomically
    /// Uses temp files + rename pattern. If phase 1 (write temps) fails,
    /// all temp files are cleaned up. Phase 2 (renames) is best-effort:
    /// on Linux, rename() is atomic per-file and unlikely to fail after
    /// a successful write to the same filesystem.
    pub fn save_all(&self) -> Result<(), String> {
        // Phase 1: Write all to temp files (can rollback cleanly)
        if let Err(e) = self.write_all_temps() {
            self.rollback_temps();
            return Err(e);
        }

        // Phase 2: Commit all temp files (atomic renames)
        // On same-filesystem renames, this is effectively atomic per-file.
        // If a rename fails mid-way, some files are updated and some aren't,
        // but each individual file is either old or new (never corrupted).
        self.tasks.commit_temp()?;
        self.features.commit_temp()?;
        self.disciplines.commit_temp()?;
        self.metadata.commit_temp()?;

        Ok(())
    }

    /// Write all temp files (phase 1 of atomic save)
    fn write_all_temps(&self) -> Result<(), String> {
        self.tasks.save_to_temp()?;
        self.features.save_to_temp()?;
        self.disciplines.save_to_temp()?;
        self.metadata.save_to_temp()?;
        Ok(())
    }

    /// Rollback all temp files (cleanup on error)
    fn rollback_temps(&self) {
        self.tasks.rollback_temp();
        self.features.rollback_temp();
        self.disciplines.rollback_temp();
        self.metadata.rollback_temp();
    }

    /// Create a new task with automatic ID assignment and feature/discipline creation
    /// Thread-safe: Uses exclusive file lock
    ///
    /// # Errors
    /// - Returns error if feature or discipline name is empty
    /// - Returns error if title is empty
    /// - Returns error if dependency references non-existent task
    pub fn create_task(&mut self, task: TaskInput) -> Result<u32, String> {
        // 0. Validate input
        if task.feature.trim().is_empty() {
            return Err("Feature name cannot be empty".to_string());
        }
        if task.discipline.trim().is_empty() {
            return Err("Discipline name cannot be empty".to_string());
        }
        if task.title.trim().is_empty() {
            return Err("Task title cannot be empty".to_string());
        }

        // 1. Acquire exclusive lock (blocks until available)
        let _lock = self.acquire_lock()?;

        // 2. Reload all files from disk (ensure fresh state)
        self.load_all()?;

        // 3. Validate discipline exists (or auto-create if needed)
        self.disciplines
            .ensure_discipline_exists(&task.discipline, &task.discipline_acronym)?;

        // 4. Auto-create feature if needed
        self.features
            .ensure_feature_exists(&task.feature, &task.feature_acronym)?;

        // 5. Assign next ID (uses global counter)
        let next_id = self.metadata.get_next_id(self.tasks.get_all());

        // 6. Create task with assigned ID
        let new_task = Task {
            id: next_id,
            feature: task.feature,
            discipline: task.discipline,
            title: task.title,
            description: task.description,
            status: TaskStatus::Pending,
            priority: task.priority,
            tags: task.tags,
            depends_on: task.depends_on,
            blocked_by: None,
            created: Some(chrono::Utc::now().format("%Y-%m-%d").to_string()),
            updated: None,
            completed: None,
            acceptance_criteria: task.acceptance_criteria.unwrap_or_default(),
        };

        // 7. Validate dependencies exist
        self.validate_dependencies(&new_task)?;

        // 8. Add task to tasks file
        self.tasks.add_task(new_task);

        // 9. Update counters
        self.metadata.rebuild_counters(self.tasks.get_all());

        // 10. Write all files atomically
        self.save_all()?;

        // 11. Lock auto-released on drop
        Ok(next_id)
    }

    /// Validate that all task dependencies reference existing tasks
    fn validate_dependencies(&self, task: &Task) -> Result<(), String> {
        for dep_id in &task.depends_on {
            if !self.tasks.has_task(*dep_id) {
                return Err(format!("Dependency task {} does not exist", dep_id));
            }
        }
        Ok(())
    }

    /// Validate dependencies including circular dependency detection
    /// Used when updating tasks to prevent creating cycles
    fn validate_dependencies_with_cycles(
        &self,
        task_id: u32,
        task: &Task,
    ) -> Result<(), String> {
        // First validate dependencies exist
        self.validate_dependencies(task)?;

        // Check for self-referential dependency
        if task.depends_on.contains(&task_id) {
            return Err(format!(
                "Task {} cannot depend on itself",
                task_id
            ));
        }

        // Check for circular dependencies using depth-first search
        for dep_id in &task.depends_on {
            if self.has_circular_dependency(task_id, *dep_id)? {
                return Err(format!(
                    "Circular dependency detected: task {} would create a cycle with task {}",
                    task_id, dep_id
                ));
            }
        }

        Ok(())
    }

    /// Recursively check if adding a dependency would create a cycle
    /// Returns true if dep_id eventually depends on task_id
    fn has_circular_dependency(&self, task_id: u32, dep_id: u32) -> Result<bool, String> {
        use std::collections::HashSet;

        let mut visited = HashSet::new();
        let mut stack = vec![dep_id];

        while let Some(current_id) = stack.pop() {
            // If we've reached the original task, we have a cycle
            if current_id == task_id {
                return Ok(true);
            }

            // Avoid infinite loops in case of existing cycles
            if !visited.insert(current_id) {
                continue;
            }

            // Find the current task and add its dependencies to the stack
            if let Some(current_task) = self.tasks.get_all().iter().find(|t| t.id == current_id) {
                for &next_dep in &current_task.depends_on {
                    stack.push(next_dep);
                }
            }
        }

        Ok(false)
    }

    /// Acquire exclusive file lock (blocks until available)
    fn acquire_lock(&self) -> Result<FileLock, String> {
        let lock_file = File::create(&self.lock_file)
            .map_err(|e| format!("Failed to create lock file: {}", e))?;

        // Blocking exclusive lock (waits if another process holds it)
        lock_file
            .lock_exclusive()
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;

        Ok(FileLock(lock_file))
    }

    /// Get a task by ID
    pub fn get_task_by_id(&self, id: u32) -> Option<&Task> {
        self.tasks.get_all().iter().find(|t| t.id == id)
    }

    /// Update an existing task
    /// Thread-safe: Uses exclusive file lock
    ///
    /// # Errors
    /// - Returns error if task ID doesn't exist
    /// - Returns error if validation fails (empty fields, invalid dependencies)
    pub fn update_task(&mut self, id: u32, update: TaskInput) -> Result<(), String> {
        // 0. Validate input
        if update.feature.trim().is_empty() {
            return Err("Feature name cannot be empty".to_string());
        }
        if update.discipline.trim().is_empty() {
            return Err("Discipline name cannot be empty".to_string());
        }
        if update.title.trim().is_empty() {
            return Err("Task title cannot be empty".to_string());
        }

        // 1. Acquire exclusive lock
        let _lock = self.acquire_lock()?;

        // 2. Reload all files from disk
        self.load_all()?;

        // 3. Find the task to update
        let task_index = self
            .tasks
            .items_mut()
            .iter()
            .position(|t| t.id == id)
            .ok_or_else(|| format!("Task {} does not exist", id))?;

        // 4. Validate discipline exists (or auto-create if needed)
        self.disciplines
            .ensure_discipline_exists(&update.discipline, &update.discipline_acronym)?;

        // 5. Auto-create feature if needed
        self.features
            .ensure_feature_exists(&update.feature, &update.feature_acronym)?;

        // 6. Create updated task (preserve ID, status, and timestamps)
        let old_task = &self.tasks.items_mut()[task_index];
        let updated_task = Task {
            id,
            feature: update.feature,
            discipline: update.discipline,
            title: update.title,
            description: update.description,
            status: old_task.status, // Preserve status
            priority: update.priority,
            tags: update.tags,
            depends_on: update.depends_on,
            blocked_by: old_task.blocked_by.clone(), // Preserve blocked_by
            created: old_task.created.clone(),       // Preserve created
            updated: Some(chrono::Utc::now().format("%Y-%m-%d").to_string()),
            completed: old_task.completed.clone(), // Preserve completed
            acceptance_criteria: update.acceptance_criteria.unwrap_or_default(),
        };

        // 7. Validate dependencies exist (including checking for cycles)
        self.validate_dependencies_with_cycles(id, &updated_task)?;

        // 8. Update the task in place
        self.tasks.items_mut()[task_index] = updated_task;

        // 9. Update counters
        self.metadata.rebuild_counters(self.tasks.get_all());

        // 10. Write all files atomically
        self.save_all()?;

        Ok(())
    }

    /// Delete a task by ID
    /// Thread-safe: Uses exclusive file lock
    ///
    /// # Errors
    /// - Returns error if task ID doesn't exist
    /// - Returns error if other tasks depend on this task
    pub fn delete_task(&mut self, id: u32) -> Result<(), String> {
        // 1. Acquire exclusive lock
        let _lock = self.acquire_lock()?;

        // 2. Reload all files from disk
        self.load_all()?;

        // 3. Check if any tasks depend on this one
        for task in self.tasks.get_all() {
            if task.depends_on.contains(&id) {
                return Err(format!(
                    "Cannot delete task {}: task {} depends on it",
                    id, task.id
                ));
            }
        }

        // 4. Find and remove the task
        let initial_len = self.tasks.items_mut().len();
        self.tasks.items_mut().retain(|t| t.id != id);

        if self.tasks.items_mut().len() == initial_len {
            return Err(format!("Task {} does not exist", id));
        }

        // 5. Update counters
        self.metadata.rebuild_counters(self.tasks.get_all());

        // 6. Write all files atomically
        self.save_all()?;

        Ok(())
    }

    // Getter methods

    pub fn get_tasks(&self) -> &[Task] {
        self.tasks.get_all()
    }

    pub fn get_features(&self) -> &[super::Feature] {
        self.features.get_all()
    }

    pub fn get_disciplines(&self) -> &[super::Discipline] {
        self.disciplines.get_all()
    }

    pub fn get_project_info(&self) -> &super::ProjectMetadata {
        self.metadata.get_project_info()
    }

    pub fn get_existing_feature_names(&self) -> Vec<String> {
        self.features
            .get_all()
            .iter()
            .map(|f| f.name.clone())
            .collect()
    }

    pub fn rebuild_counters(&mut self) {
        self.metadata.rebuild_counters(self.tasks.get_all());
    }

    pub fn get_next_task_id(&self) -> u32 {
        self.metadata.get_next_id(self.tasks.get_all())
    }
}

/// RAII lock guard - auto-releases lock on drop
struct FileLock(File);

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}
