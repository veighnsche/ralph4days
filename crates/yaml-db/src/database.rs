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
}

/// Multi-file YAML database coordinator
pub struct YamlDatabase {
    _base_path: PathBuf,
    lock_file: PathBuf,
    tasks: TasksFile,
    features: FeaturesFile,
    disciplines: DisciplinesFile,
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
            .ensure_discipline_exists(&task.discipline)?;

        // 4. Auto-create feature if needed
        self.features.ensure_feature_exists(&task.feature)?;

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
