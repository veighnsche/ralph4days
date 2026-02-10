use crate::types::*;
use crate::SqliteDb;
use ralph_errors::{codes, ralph_err, RalphResultExt};

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
            .ralph_err(codes::DB_READ, "Failed to check feature")?;
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
            .ralph_err(codes::DB_READ, "Failed to check acronym")?;
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
                 knowledge_paths, context_files, dependencies, status) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'active')",
                rusqlite::params![
                    input.name,
                    input.display_name,
                    input.acronym,
                    input.description,
                    now,
                    kp_json,
                    cf_json,
                    deps_json,
                ],
            )
            .ralph_err(codes::DB_WRITE, "Failed to insert feature")?;

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
            .ralph_err(codes::DB_READ, "Failed to check feature")?;
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
            .ralph_err(codes::DB_READ, "Failed to check acronym")?;
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
                 knowledge_paths = ?4, context_files = ?5, dependencies = ?6 WHERE name = ?7",
                rusqlite::params![
                    input.display_name,
                    input.acronym,
                    input.description,
                    kp_json,
                    cf_json,
                    deps_json,
                    input.name,
                ],
            )
            .ralph_err(codes::DB_WRITE, "Failed to update feature")?;

        Ok(())
    }

    pub fn delete_feature(&self, name: String) -> Result<(), String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title FROM tasks WHERE feature = ?1")
            .ralph_err(codes::DB_READ, "Failed to prepare query")?;

        let tasks: Vec<(u32, String)> = stmt
            .query_map([&name], |row| Ok((row.get(0)?, row.get(1)?)))
            .ralph_err(codes::DB_READ, "Failed to query tasks")?
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
            .ralph_err(codes::DB_WRITE, "Failed to delete feature")?;

        if affected == 0 {
            return ralph_err!(codes::FEATURE_OPS, "Feature '{name}' does not exist");
        }

        Ok(())
    }

    pub fn get_features(&self) -> Vec<Feature> {
        let Ok(mut stmt) = self.conn.prepare(
            "SELECT name, display_name, acronym, description, created, \
             knowledge_paths, context_files, dependencies, status \
             FROM features ORDER BY name",
        ) else {
            return vec![];
        };

        let mut comments_map = self.get_all_comments_by_feature();

        stmt.query_map([], |row| {
            let kp_json: String = row.get(5)?;
            let cf_json: String = row.get(6)?;
            let deps_json: String = row.get(7)?;
            let status_str: String = row.get(8)?;
            let name: String = row.get(0)?;
            Ok(Feature {
                name,
                display_name: row.get(1)?,
                acronym: row.get(2)?,
                description: row.get(3)?,
                created: row.get(4)?,
                knowledge_paths: serde_json::from_str(&kp_json).unwrap_or_default(),
                context_files: serde_json::from_str(&cf_json).unwrap_or_default(),
                dependencies: serde_json::from_str(&deps_json).unwrap_or_default(),
                status: FeatureStatus::parse(&status_str).unwrap_or(FeatureStatus::Active),
                comments: vec![],
            })
        })
        .map_or_else(
            |_| vec![],
            |rows| {
                rows.filter_map(std::result::Result::ok)
                    .map(|mut f| {
                        f.comments = comments_map.remove(&f.name).unwrap_or_default();
                        f
                    })
                    .collect()
            },
        )
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
            .ralph_err(codes::DB_READ, "Feature not found")?;

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

        let updated = serde_json::to_string(&files).ralph_err(codes::DB_WRITE, "JSON error")?;
        self.conn
            .execute(
                "UPDATE features SET context_files = ?1 WHERE name = ?2",
                rusqlite::params![updated, feature_name],
            )
            .ralph_err(codes::DB_WRITE, "Failed to update context_files")?;

        Ok(true)
    }
}
