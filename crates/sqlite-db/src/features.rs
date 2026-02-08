use crate::types::*;
use crate::SqliteDb;
use ralph_errors::{codes, ralph_err, ralph_map_err};
use ralph_rag::{check_deduplication, select_for_pruning, DeduplicationResult, FeatureLearning};

impl SqliteDb {
    pub fn create_feature(&self, input: FeatureInput) -> Result<(), String> {
        if input.name.trim().is_empty() {
            return ralph_err!(codes::FEATURE_OPS, "Feature name cannot be empty");
        }
        if input.display_name.trim().is_empty() {
            return ralph_err!(codes::FEATURE_OPS, "Feature display name cannot be empty");
        }

        crate::acronym::validate_acronym_format(&input.acronym)?;

        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM features WHERE name = ?1",
                [&input.name],
                |row| row.get(0),
            )
            .map_err(ralph_map_err!(codes::DB_READ, "Failed to check feature"))?;
        if exists {
            return ralph_err!(
                codes::FEATURE_OPS,
                "Feature '{}' already exists",
                input.name
            );
        }

        let acronym_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM features WHERE acronym = ?1",
                [&input.acronym],
                |row| row.get(0),
            )
            .map_err(ralph_map_err!(codes::DB_READ, "Failed to check acronym"))?;
        if acronym_exists {
            return ralph_err!(
                codes::FEATURE_OPS,
                "Acronym '{}' is already used by another feature",
                input.acronym
            );
        }

        let now = self.now().format("%Y-%m-%d").to_string();
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
            .map_err(ralph_map_err!(codes::DB_WRITE, "Failed to insert feature"))?;

        Ok(())
    }

    pub fn update_feature(&self, input: FeatureInput) -> Result<(), String> {
        if input.display_name.trim().is_empty() {
            return ralph_err!(codes::FEATURE_OPS, "Feature display name cannot be empty");
        }

        crate::acronym::validate_acronym_format(&input.acronym)?;

        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM features WHERE name = ?1",
                [&input.name],
                |row| row.get(0),
            )
            .map_err(ralph_map_err!(codes::DB_READ, "Failed to check feature"))?;
        if !exists {
            return ralph_err!(
                codes::FEATURE_OPS,
                "Feature '{}' does not exist",
                input.name
            );
        }

        let acronym_conflict: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM features WHERE acronym = ?1 AND name != ?2",
                rusqlite::params![input.acronym, input.name],
                |row| row.get(0),
            )
            .map_err(ralph_map_err!(codes::DB_READ, "Failed to check acronym"))?;
        if acronym_conflict {
            return ralph_err!(
                codes::FEATURE_OPS,
                "Acronym '{}' is already used by another feature",
                input.acronym
            );
        }

        let kp_json = serde_json::to_string(&input.knowledge_paths).unwrap_or_else(|_| "[]".into());
        let cf_json = serde_json::to_string(&input.context_files).unwrap_or_else(|_| "[]".into());
        let deps_json = serde_json::to_string(&input.dependencies).unwrap_or_else(|_| "[]".into());

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
            .map_err(ralph_map_err!(codes::DB_WRITE, "Failed to update feature"))?;

        Ok(())
    }

    pub fn delete_feature(&self, name: String) -> Result<(), String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title FROM tasks WHERE feature = ?1")
            .map_err(ralph_map_err!(codes::DB_READ, "Failed to prepare query"))?;

        let tasks: Vec<(u32, String)> = stmt
            .query_map([&name], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(ralph_map_err!(codes::DB_READ, "Failed to query tasks"))?
            .filter_map(std::result::Result::ok)
            .collect();

        if let Some((task_id, task_title)) = tasks.first() {
            return ralph_err!(
                codes::FEATURE_OPS,
                "Cannot delete feature '{name}': task {task_id} ('{task_title}') belongs to it"
            );
        }

        let affected = self
            .conn
            .execute("DELETE FROM features WHERE name = ?1", [&name])
            .map_err(ralph_map_err!(codes::DB_WRITE, "Failed to delete feature"))?;

        if affected == 0 {
            return ralph_err!(codes::FEATURE_OPS, "Feature '{name}' does not exist");
        }

        Ok(())
    }

    pub fn get_features(&self) -> Vec<Feature> {
        let Ok(mut stmt) = self.conn.prepare(
            "SELECT name, display_name, acronym, description, created, \
             knowledge_paths, context_files, architecture, boundaries, learnings, dependencies \
             FROM features ORDER BY name",
        ) else {
            return vec![];
        };

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
        .map_or_else(
            |_| vec![],
            |rows| rows.filter_map(std::result::Result::ok).collect(),
        )
    }

    pub fn append_feature_learning(
        &self,
        feature_name: &str,
        mut learning: FeatureLearning,
        max_learnings: usize,
    ) -> Result<bool, String> {
        learning.created = self.now().to_rfc3339();

        let learnings_json: String = self
            .conn
            .query_row(
                "SELECT learnings FROM features WHERE name = ?1",
                [feature_name],
                |row| row.get(0),
            )
            .map_err(ralph_map_err!(codes::DB_READ, "Feature not found"))?;

        let mut learnings: Vec<FeatureLearning> =
            serde_json::from_str(&learnings_json).unwrap_or_default();

        match check_deduplication(&learning.text, &learnings) {
            DeduplicationResult::Duplicate { existing_index } => {
                learnings[existing_index].record_re_observation();
                let updated = serde_json::to_string(&learnings)
                    .map_err(ralph_map_err!(codes::DB_WRITE, "JSON error"))?;
                self.conn
                    .execute(
                        "UPDATE features SET learnings = ?1 WHERE name = ?2",
                        rusqlite::params![updated, feature_name],
                    )
                    .map_err(ralph_map_err!(
                        codes::DB_WRITE,
                        "Failed to update learnings"
                    ))?;
                Ok(false)
            }
            DeduplicationResult::Conflict {
                existing_index,
                new_text,
            } => {
                learnings[existing_index] = learning;
                learnings[existing_index].reason =
                    Some(format!("Replaced conflicting learning: {new_text}"));
                let updated = serde_json::to_string(&learnings)
                    .map_err(ralph_map_err!(codes::DB_WRITE, "JSON error"))?;
                self.conn
                    .execute(
                        "UPDATE features SET learnings = ?1 WHERE name = ?2",
                        rusqlite::params![updated, feature_name],
                    )
                    .map_err(ralph_map_err!(
                        codes::DB_WRITE,
                        "Failed to update learnings"
                    ))?;
                Ok(true)
            }
            DeduplicationResult::Unique => {
                learnings.push(learning);

                let to_prune = select_for_pruning(&learnings, max_learnings);
                if !to_prune.is_empty() {
                    let mut sorted_prune = to_prune;
                    sorted_prune.sort_unstable_by(|a, b| b.cmp(a));
                    for idx in sorted_prune {
                        learnings.remove(idx);
                    }
                }

                if learnings.len() > max_learnings {
                    return ralph_err!(
                        codes::FEATURE_OPS,
                        "Learnings full — all are protected. Review and remove some manually."
                    );
                }

                let updated = serde_json::to_string(&learnings)
                    .map_err(ralph_map_err!(codes::DB_WRITE, "JSON error"))?;
                self.conn
                    .execute(
                        "UPDATE features SET learnings = ?1 WHERE name = ?2",
                        rusqlite::params![updated, feature_name],
                    )
                    .map_err(ralph_map_err!(
                        codes::DB_WRITE,
                        "Failed to update learnings"
                    ))?;
                Ok(true)
            }
        }
    }

    pub fn remove_feature_learning(&self, feature_name: &str, index: usize) -> Result<(), String> {
        let learnings_json: String = self
            .conn
            .query_row(
                "SELECT learnings FROM features WHERE name = ?1",
                [feature_name],
                |row| row.get(0),
            )
            .map_err(ralph_map_err!(codes::DB_READ, "Feature not found"))?;

        let mut learnings: Vec<FeatureLearning> =
            serde_json::from_str(&learnings_json).unwrap_or_default();

        if index >= learnings.len() {
            return ralph_err!(
                codes::FEATURE_OPS,
                "Learning index {} out of range (feature has {} learnings)",
                index,
                learnings.len()
            );
        }

        learnings.remove(index);

        let updated = serde_json::to_string(&learnings)
            .map_err(ralph_map_err!(codes::DB_WRITE, "JSON error"))?;
        self.conn
            .execute(
                "UPDATE features SET learnings = ?1 WHERE name = ?2",
                rusqlite::params![updated, feature_name],
            )
            .map_err(ralph_map_err!(
                codes::DB_WRITE,
                "Failed to update learnings"
            ))?;

        Ok(())
    }

    pub fn add_feature_context_file(
        &self,
        feature_name: &str,
        file_path: &str,
        max_files: usize,
    ) -> Result<bool, String> {
        if file_path.starts_with('/') || file_path.starts_with('\\') {
            return ralph_err!(codes::FEATURE_OPS, "Context file path must be relative");
        }
        if file_path.contains("..") {
            return ralph_err!(codes::FEATURE_OPS, "Context file path cannot contain '..'");
        }

        let cf_json: String = self
            .conn
            .query_row(
                "SELECT context_files FROM features WHERE name = ?1",
                [feature_name],
                |row| row.get(0),
            )
            .map_err(ralph_map_err!(codes::DB_READ, "Feature not found"))?;

        let mut files: Vec<String> = serde_json::from_str(&cf_json).unwrap_or_default();

        if files.iter().any(|f| f == file_path) {
            return Ok(false);
        }

        if files.len() >= max_files {
            return ralph_err!(
                codes::FEATURE_OPS,
                "Context files limit ({max_files}) reached for feature '{feature_name}'"
            );
        }

        files.push(file_path.to_owned());

        let updated =
            serde_json::to_string(&files).map_err(ralph_map_err!(codes::DB_WRITE, "JSON error"))?;
        self.conn
            .execute(
                "UPDATE features SET context_files = ?1 WHERE name = ?2",
                rusqlite::params![updated, feature_name],
            )
            .map_err(ralph_map_err!(
                codes::DB_WRITE,
                "Failed to update context_files"
            ))?;

        Ok(true)
    }
}
