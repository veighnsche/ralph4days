use super::{DisciplinesFile, FeaturesFile, MetadataFile, TasksFile};
use crate::{Priority, Task, TaskProvenance};
use fs2::FileExt;
use std::fs::{self, File};
use std::path::PathBuf;

mod comments;
mod disciplines;
mod features;
mod stats;
mod tasks;

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
    // Execution context
    pub context_files: Vec<String>,
    pub output_artifacts: Vec<String>,
    pub hints: Option<String>,
    pub estimated_turns: Option<u32>,
    pub provenance: Option<TaskProvenance>,
}

impl Default for TaskInput {
    fn default() -> Self {
        Self {
            feature: String::new(),
            discipline: String::new(),
            title: String::new(),
            description: None,
            priority: None,
            tags: vec![],
            depends_on: vec![],
            acceptance_criteria: None,
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
        }
    }
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

    /// Acquire exclusive file lock (blocks until available)
    fn acquire_lock(&self) -> Result<FileLock, String> {
        let lock_file = File::create(&self.lock_file)
            .map_err(|e| format!("Failed to create lock file: {}", e))?;

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
