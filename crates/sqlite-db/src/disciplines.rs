use crate::types::*;
use crate::SqliteDb;

impl SqliteDb {
    /// Create a new discipline.
    pub fn create_discipline(
        &self,
        name: String,
        display_name: String,
        acronym: String,
        icon: String,
        color: String,
    ) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("Discipline name cannot be empty".to_string());
        }
        if display_name.trim().is_empty() {
            return Err("Discipline display name cannot be empty".to_string());
        }

        crate::acronym::validate_acronym_format(&acronym)?;

        // Check name uniqueness
        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM disciplines WHERE name = ?1",
                [&name],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check discipline: {}", e))?;
        if exists {
            return Err(format!("Discipline '{}' already exists", name));
        }

        // Check acronym uniqueness
        let acronym_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM disciplines WHERE acronym = ?1",
                [&acronym],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check acronym: {}", e))?;
        if acronym_exists {
            return Err(format!(
                "Acronym '{}' is already used by another discipline",
                acronym
            ));
        }

        self.conn
            .execute(
                "INSERT INTO disciplines (name, display_name, acronym, icon, color) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![name, display_name, acronym, icon, color],
            )
            .map_err(|e| format!("Failed to insert discipline: {}", e))?;

        Ok(())
    }

    /// Update an existing discipline. Preserves: system_prompt, skills, conventions, mcp_servers.
    pub fn update_discipline(
        &self,
        name: String,
        display_name: String,
        acronym: String,
        icon: String,
        color: String,
    ) -> Result<(), String> {
        if display_name.trim().is_empty() {
            return Err("Discipline display name cannot be empty".to_string());
        }

        crate::acronym::validate_acronym_format(&acronym)?;

        // Verify discipline exists
        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM disciplines WHERE name = ?1",
                [&name],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check discipline: {}", e))?;
        if !exists {
            return Err(format!("Discipline '{}' does not exist", name));
        }

        // Check acronym uniqueness (exclude self)
        let acronym_conflict: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM disciplines WHERE acronym = ?1 AND name != ?2",
                rusqlite::params![acronym, name],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check acronym: {}", e))?;
        if acronym_conflict {
            return Err(format!(
                "Acronym '{}' is already used by another discipline",
                acronym
            ));
        }

        // Update only mutable fields (preserves system_prompt, skills, conventions, mcp_servers)
        self.conn
            .execute(
                "UPDATE disciplines SET display_name = ?1, acronym = ?2, icon = ?3, color = ?4 WHERE name = ?5",
                rusqlite::params![display_name, acronym, icon, color, name],
            )
            .map_err(|e| format!("Failed to update discipline: {}", e))?;

        Ok(())
    }

    /// Delete a discipline by name. Fails if any tasks reference it.
    pub fn delete_discipline(&self, name: String) -> Result<(), String> {
        // Check if any tasks reference this discipline
        let mut stmt = self
            .conn
            .prepare("SELECT id, title FROM tasks WHERE discipline = ?1")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let tasks: Vec<(u32, String)> = stmt
            .query_map([&name], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| format!("Failed to query tasks: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        if let Some((task_id, task_title)) = tasks.first() {
            return Err(format!(
                "Cannot delete discipline '{}': task {} ('{}') belongs to it",
                name, task_id, task_title
            ));
        }

        let affected = self
            .conn
            .execute("DELETE FROM disciplines WHERE name = ?1", [&name])
            .map_err(|e| format!("Failed to delete discipline: {}", e))?;

        if affected == 0 {
            return Err(format!("Discipline '{}' does not exist", name));
        }

        Ok(())
    }

    /// Get all disciplines.
    pub fn get_disciplines(&self) -> Vec<Discipline> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT name, display_name, acronym, icon, color, system_prompt, skills, \
                 conventions, mcp_servers \
                 FROM disciplines ORDER BY name",
            )
            .unwrap();

        stmt.query_map([], |row| {
            let skills_json: String = row.get(6)?;
            let mcp_json: String = row.get(8)?;
            Ok(Discipline {
                name: row.get(0)?,
                display_name: row.get(1)?,
                acronym: row.get(2)?,
                icon: row.get(3)?,
                color: row.get(4)?,
                system_prompt: row.get(5)?,
                skills: serde_json::from_str(&skills_json).unwrap_or_default(),
                conventions: row.get(7)?,
                mcp_servers: serde_json::from_str(&mcp_json).unwrap_or_default(),
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    /// Seed 10 default disciplines. Skips any that already exist.
    pub fn seed_defaults(&self) -> Result<(), String> {
        let defaults = [
            ("frontend", "Frontend", "FRNT", "Monitor", "#3b82f6"),
            ("backend", "Backend", "BACK", "Server", "#8b5cf6"),
            ("wiring", "Wiring", "WIRE", "Cable", "#06b6d4"),
            ("database", "Database", "DTBS", "Database", "#10b981"),
            ("testing", "Testing", "TEST", "FlaskConical", "#f59e0b"),
            ("infra", "Infrastructure", "INFR", "Cloud", "#6366f1"),
            ("security", "Security", "SECR", "Shield", "#ef4444"),
            ("docs", "Documentation", "DOCS", "BookOpen", "#14b8a6"),
            ("design", "Design", "DSGN", "Palette", "#ec4899"),
            ("api", "API", "APIS", "Plug", "#84cc16"),
        ];

        for (name, display_name, acronym, icon, color) in defaults {
            // Skip if already exists
            let exists: bool = self
                .conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM disciplines WHERE name = ?1",
                    [name],
                    |row| row.get(0),
                )
                .map_err(|e| format!("Failed to check discipline: {}", e))?;

            if !exists {
                self.conn
                    .execute(
                        "INSERT INTO disciplines (name, display_name, acronym, icon, color) \
                         VALUES (?1, ?2, ?3, ?4, ?5)",
                        rusqlite::params![name, display_name, acronym, icon, color],
                    )
                    .map_err(|e| format!("Failed to seed discipline '{}': {}", name, e))?;
            }
        }

        Ok(())
    }
}
