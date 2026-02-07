use crate::types::*;
use crate::SqliteDb;
use ralph_rag::{check_deduplication, select_for_pruning, DeduplicationResult, FeatureLearning};

impl SqliteDb {
    /// Create a new feature.
    pub fn create_feature(&self, input: FeatureInput) -> Result<(), String> {
        if input.name.trim().is_empty() {
            return Err("Feature name cannot be empty".to_owned());
        }
        if input.display_name.trim().is_empty() {
            return Err("Feature display name cannot be empty".to_owned());
        }

        crate::acronym::validate_acronym_format(&input.acronym)?;

        // Check name uniqueness (PK will catch it but better error message)
        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM features WHERE name = ?1",
                [&input.name],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check feature: {e}"))?;
        if exists {
            return Err(format!("Feature '{}' already exists", input.name));
        }

        // Check acronym uniqueness (UNIQUE constraint will catch it but better error)
        let acronym_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM features WHERE acronym = ?1",
                [&input.acronym],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check acronym: {e}"))?;
        if acronym_exists {
            return Err(format!(
                "Acronym '{}' is already used by another feature",
                input.acronym
            ));
        }

        let now = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let kp_json = serde_json::to_string(&input.knowledge_paths).unwrap_or_else(|_| "[]".into());
        let cf_json = serde_json::to_string(&input.context_files).unwrap_or_else(|_| "[]".into());
        let deps_json = serde_json::to_string(&input.dependencies).unwrap_or_else(|_| "[]".into());

        self.conn
            .execute(
                "INSERT INTO features (name, display_name, acronym, description, created, \
                 knowledge_paths, context_files, architecture, boundaries, learnings, dependencies) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, '[]', ?10)",
                rusqlite::params![
                    input.name,
                    input.display_name,
                    input.acronym,
                    input.description,
                    now,
                    kp_json,
                    cf_json,
                    input.architecture,
                    input.boundaries,
                    deps_json,
                ],
            )
            .map_err(|e| format!("Failed to insert feature: {e}"))?;

        Ok(())
    }

    /// Update an existing feature. Preserves: created, learnings.
    pub fn update_feature(&self, input: FeatureInput) -> Result<(), String> {
        if input.display_name.trim().is_empty() {
            return Err("Feature display name cannot be empty".to_owned());
        }

        crate::acronym::validate_acronym_format(&input.acronym)?;

        // Verify feature exists
        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM features WHERE name = ?1",
                [&input.name],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check feature: {e}"))?;
        if !exists {
            return Err(format!("Feature '{}' does not exist", input.name));
        }

        // Check acronym uniqueness (exclude self)
        let acronym_conflict: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM features WHERE acronym = ?1 AND name != ?2",
                rusqlite::params![input.acronym, input.name],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check acronym: {e}"))?;
        if acronym_conflict {
            return Err(format!(
                "Acronym '{}' is already used by another feature",
                input.acronym
            ));
        }

        let kp_json = serde_json::to_string(&input.knowledge_paths).unwrap_or_else(|_| "[]".into());
        let cf_json = serde_json::to_string(&input.context_files).unwrap_or_else(|_| "[]".into());
        let deps_json = serde_json::to_string(&input.dependencies).unwrap_or_else(|_| "[]".into());

        // Update mutable fields (preserves created, learnings)
        self.conn
            .execute(
                "UPDATE features SET display_name = ?1, acronym = ?2, description = ?3, \
                 architecture = ?4, boundaries = ?5, knowledge_paths = ?6, context_files = ?7, \
                 dependencies = ?8 WHERE name = ?9",
                rusqlite::params![
                    input.display_name,
                    input.acronym,
                    input.description,
                    input.architecture,
                    input.boundaries,
                    kp_json,
                    cf_json,
                    deps_json,
                    input.name,
                ],
            )
            .map_err(|e| format!("Failed to update feature: {e}"))?;

        Ok(())
    }

    /// Delete a feature by name. Fails if any tasks reference it (RESTRICT FK).
    pub fn delete_feature(&self, name: String) -> Result<(), String> {
        // Check if any tasks reference this feature (better error than FK violation)
        let mut stmt = self
            .conn
            .prepare("SELECT id, title FROM tasks WHERE feature = ?1")
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let tasks: Vec<(u32, String)> = stmt
            .query_map([&name], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| format!("Failed to query tasks: {e}"))?
            .filter_map(std::result::Result::ok)
            .collect();

        if let Some((task_id, task_title)) = tasks.first() {
            return Err(format!(
                "Cannot delete feature '{name}': task {task_id} ('{task_title}') belongs to it"
            ));
        }

        let affected = self
            .conn
            .execute("DELETE FROM features WHERE name = ?1", [&name])
            .map_err(|e| format!("Failed to delete feature: {e}"))?;

        if affected == 0 {
            return Err(format!("Feature '{name}' does not exist"));
        }

        Ok(())
    }

    /// Get all features.
    pub fn get_features(&self) -> Vec<Feature> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT name, display_name, acronym, description, created, \
                 knowledge_paths, context_files, architecture, boundaries, learnings, dependencies \
                 FROM features ORDER BY name",
            )
            .unwrap();

        stmt.query_map([], |row| {
            let kp_json: String = row.get(5)?;
            let cf_json: String = row.get(6)?;
            let arch: Option<String> = row.get(7)?;
            let bounds: Option<String> = row.get(8)?;
            let learnings_json: String = row.get(9)?;
            let deps_json: String = row.get(10)?;
            Ok(Feature {
                name: row.get(0)?,
                display_name: row.get(1)?,
                acronym: row.get(2)?,
                description: row.get(3)?,
                created: row.get(4)?,
                knowledge_paths: serde_json::from_str(&kp_json).unwrap_or_default(),
                context_files: serde_json::from_str(&cf_json).unwrap_or_default(),
                architecture: arch,
                boundaries: bounds,
                learnings: serde_json::from_str(&learnings_json).unwrap_or_default(),
                dependencies: serde_json::from_str(&deps_json).unwrap_or_default(),
            })
        })
        .unwrap()
        .filter_map(std::result::Result::ok)
        .collect()
    }

    /// Append a learning to a feature. Uses deduplication to prevent spam.
    /// Returns Ok(true) if added, Ok(false) if deduped (existing hit_count incremented).
    pub fn append_feature_learning(
        &self,
        feature_name: &str,
        learning: FeatureLearning,
        max_learnings: usize,
    ) -> Result<bool, String> {
        // Read current learnings
        let learnings_json: String = self
            .conn
            .query_row(
                "SELECT learnings FROM features WHERE name = ?1",
                [feature_name],
                |row| row.get(0),
            )
            .map_err(|e| format!("Feature '{feature_name}' not found: {e}"))?;

        let mut learnings: Vec<FeatureLearning> =
            serde_json::from_str(&learnings_json).unwrap_or_default();

        // Check deduplication
        match check_deduplication(&learning.text, &learnings) {
            DeduplicationResult::Duplicate { existing_index } => {
                // Increment hit_count on existing
                learnings[existing_index].record_re_observation();
                let updated =
                    serde_json::to_string(&learnings).map_err(|e| format!("JSON error: {e}"))?;
                self.conn
                    .execute(
                        "UPDATE features SET learnings = ?1 WHERE name = ?2",
                        rusqlite::params![updated, feature_name],
                    )
                    .map_err(|e| format!("Failed to update learnings: {e}"))?;
                Ok(false)
            }
            DeduplicationResult::Conflict {
                existing_index,
                new_text,
            } => {
                // Replace the conflicting learning with the new one
                learnings[existing_index] = learning;
                // Update reason to note the conflict
                learnings[existing_index].reason =
                    Some(format!("Replaced conflicting learning: {new_text}"));
                let updated =
                    serde_json::to_string(&learnings).map_err(|e| format!("JSON error: {e}"))?;
                self.conn
                    .execute(
                        "UPDATE features SET learnings = ?1 WHERE name = ?2",
                        rusqlite::params![updated, feature_name],
                    )
                    .map_err(|e| format!("Failed to update learnings: {e}"))?;
                Ok(true)
            }
            DeduplicationResult::Unique => {
                // Add new learning, enforce cap
                learnings.push(learning);

                let to_prune = select_for_pruning(&learnings, max_learnings);
                if !to_prune.is_empty() {
                    // Remove in reverse order to preserve indices
                    let mut sorted_prune = to_prune;
                    sorted_prune.sort_unstable_by(|a, b| b.cmp(a));
                    for idx in sorted_prune {
                        learnings.remove(idx);
                    }
                }

                // If still over cap (all protected), reject
                if learnings.len() > max_learnings {
                    return Err(
                        "Learnings full — all are protected. Review and remove some manually.".to_owned(),
                    );
                }

                let updated =
                    serde_json::to_string(&learnings).map_err(|e| format!("JSON error: {e}"))?;
                self.conn
                    .execute(
                        "UPDATE features SET learnings = ?1 WHERE name = ?2",
                        rusqlite::params![updated, feature_name],
                    )
                    .map_err(|e| format!("Failed to update learnings: {e}"))?;
                Ok(true)
            }
        }
    }

    /// Remove a learning from a feature by index.
    pub fn remove_feature_learning(&self, feature_name: &str, index: usize) -> Result<(), String> {
        let learnings_json: String = self
            .conn
            .query_row(
                "SELECT learnings FROM features WHERE name = ?1",
                [feature_name],
                |row| row.get(0),
            )
            .map_err(|e| format!("Feature '{feature_name}' not found: {e}"))?;

        let mut learnings: Vec<FeatureLearning> =
            serde_json::from_str(&learnings_json).unwrap_or_default();

        if index >= learnings.len() {
            return Err(format!(
                "Learning index {} out of range (feature has {} learnings)",
                index,
                learnings.len()
            ));
        }

        learnings.remove(index);

        let updated =
            serde_json::to_string(&learnings).map_err(|e| format!("JSON error: {e}"))?;
        self.conn
            .execute(
                "UPDATE features SET learnings = ?1 WHERE name = ?2",
                rusqlite::params![updated, feature_name],
            )
            .map_err(|e| format!("Failed to update learnings: {e}"))?;

        Ok(())
    }

    /// Add a context file to a feature. Idempotent (no-op if already present).
    /// Returns Ok(true) if added, Ok(false) if already present.
    pub fn add_feature_context_file(
        &self,
        feature_name: &str,
        file_path: &str,
        max_files: usize,
    ) -> Result<bool, String> {
        // Validate path: reject absolute paths and ".."
        if file_path.starts_with('/') || file_path.starts_with('\\') {
            return Err("Context file path must be relative".to_owned());
        }
        if file_path.contains("..") {
            return Err("Context file path cannot contain '..'".to_owned());
        }

        let cf_json: String = self
            .conn
            .query_row(
                "SELECT context_files FROM features WHERE name = ?1",
                [feature_name],
                |row| row.get(0),
            )
            .map_err(|e| format!("Feature '{feature_name}' not found: {e}"))?;

        let mut files: Vec<String> = serde_json::from_str(&cf_json).unwrap_or_default();

        // Idempotent: already present
        if files.iter().any(|f| f == file_path) {
            return Ok(false);
        }

        // Cap enforcement
        if files.len() >= max_files {
            return Err(format!(
                "Context files limit ({max_files}) reached for feature '{feature_name}'"
            ));
        }

        files.push(file_path.to_owned());

        let updated = serde_json::to_string(&files).map_err(|e| format!("JSON error: {e}"))?;
        self.conn
            .execute(
                "UPDATE features SET context_files = ?1 WHERE name = ?2",
                rusqlite::params![updated, feature_name],
            )
            .map_err(|e| format!("Failed to update context_files: {e}"))?;

        Ok(true)
    }
}
