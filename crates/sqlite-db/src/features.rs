use crate::types::*;
use crate::SqliteDb;

impl SqliteDb {
    /// Create a new feature.
    pub fn create_feature(
        &self,
        name: String,
        display_name: String,
        acronym: String,
        description: Option<String>,
    ) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("Feature name cannot be empty".to_string());
        }
        if display_name.trim().is_empty() {
            return Err("Feature display name cannot be empty".to_string());
        }

        crate::acronym::validate_acronym_format(&acronym)?;

        // Check name uniqueness (PK will catch it but better error message)
        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM features WHERE name = ?1",
                [&name],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check feature: {}", e))?;
        if exists {
            return Err(format!("Feature '{}' already exists", name));
        }

        // Check acronym uniqueness (UNIQUE constraint will catch it but better error)
        let acronym_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM features WHERE acronym = ?1",
                [&acronym],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check acronym: {}", e))?;
        if acronym_exists {
            return Err(format!(
                "Acronym '{}' is already used by another feature",
                acronym
            ));
        }

        let now = chrono::Utc::now().format("%Y-%m-%d").to_string();

        self.conn
            .execute(
                "INSERT INTO features (name, display_name, acronym, description, created) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![name, display_name, acronym, description, now],
            )
            .map_err(|e| format!("Failed to insert feature: {}", e))?;

        Ok(())
    }

    /// Update an existing feature. Preserves: created, knowledge_paths, context_files.
    pub fn update_feature(
        &self,
        name: String,
        display_name: String,
        acronym: String,
        description: Option<String>,
    ) -> Result<(), String> {
        if display_name.trim().is_empty() {
            return Err("Feature display name cannot be empty".to_string());
        }

        crate::acronym::validate_acronym_format(&acronym)?;

        // Verify feature exists
        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM features WHERE name = ?1",
                [&name],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check feature: {}", e))?;
        if !exists {
            return Err(format!("Feature '{}' does not exist", name));
        }

        // Check acronym uniqueness (exclude self)
        let acronym_conflict: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM features WHERE acronym = ?1 AND name != ?2",
                rusqlite::params![acronym, name],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check acronym: {}", e))?;
        if acronym_conflict {
            return Err(format!(
                "Acronym '{}' is already used by another feature",
                acronym
            ));
        }

        // Update only mutable fields (preserves created, knowledge_paths, context_files)
        self.conn
            .execute(
                "UPDATE features SET display_name = ?1, acronym = ?2, description = ?3 WHERE name = ?4",
                rusqlite::params![display_name, acronym, description, name],
            )
            .map_err(|e| format!("Failed to update feature: {}", e))?;

        Ok(())
    }

    /// Delete a feature by name. Fails if any tasks reference it (RESTRICT FK).
    pub fn delete_feature(&self, name: String) -> Result<(), String> {
        // Check if any tasks reference this feature (better error than FK violation)
        let mut stmt = self
            .conn
            .prepare("SELECT id, title FROM tasks WHERE feature = ?1")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let tasks: Vec<(u32, String)> = stmt
            .query_map([&name], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| format!("Failed to query tasks: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        if let Some((task_id, task_title)) = tasks.first() {
            return Err(format!(
                "Cannot delete feature '{}': task {} ('{}') belongs to it",
                name, task_id, task_title
            ));
        }

        let affected = self
            .conn
            .execute("DELETE FROM features WHERE name = ?1", [&name])
            .map_err(|e| format!("Failed to delete feature: {}", e))?;

        if affected == 0 {
            return Err(format!("Feature '{}' does not exist", name));
        }

        Ok(())
    }

    /// Get all features.
    pub fn get_features(&self) -> Vec<Feature> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT name, display_name, acronym, description, created, knowledge_paths, context_files \
                 FROM features ORDER BY name",
            )
            .unwrap();

        stmt.query_map([], |row| {
            let kp_json: String = row.get(5)?;
            let cf_json: String = row.get(6)?;
            Ok(Feature {
                name: row.get(0)?,
                display_name: row.get(1)?,
                acronym: row.get(2)?,
                description: row.get(3)?,
                created: row.get(4)?,
                knowledge_paths: serde_json::from_str(&kp_json).unwrap_or_default(),
                context_files: serde_json::from_str(&cf_json).unwrap_or_default(),
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }
}
