use crate::errors::{codes, ralph_err, ralph_map_err};
use crate::types::*;
use crate::SqliteDb;

impl SqliteDb {
    pub fn create_discipline(
        &self,
        name: String,
        display_name: String,
        acronym: String,
        icon: String,
        color: String,
    ) -> Result<(), String> {
        if name.trim().is_empty() {
            return ralph_err!(codes::DISCIPLINE_OPS, "Discipline name cannot be empty");
        }
        if display_name.trim().is_empty() {
            return ralph_err!(
                codes::DISCIPLINE_OPS,
                "Discipline display name cannot be empty"
            );
        }

        crate::acronym::validate_acronym_format(&acronym)?;

        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM disciplines WHERE name = ?1",
                [&name],
                |row| row.get(0),
            )
            .map_err(ralph_map_err!(codes::DB_READ, "Failed to check discipline"))?;
        if exists {
            return ralph_err!(codes::DISCIPLINE_OPS, "Discipline '{name}' already exists");
        }

        let acronym_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM disciplines WHERE acronym = ?1",
                [&acronym],
                |row| row.get(0),
            )
            .map_err(ralph_map_err!(codes::DB_READ, "Failed to check acronym"))?;
        if acronym_exists {
            return ralph_err!(
                codes::DISCIPLINE_OPS,
                "Acronym '{acronym}' is already used by another discipline"
            );
        }

        self.conn
            .execute(
                "INSERT INTO disciplines (name, display_name, acronym, icon, color) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![name, display_name, acronym, icon, color],
            )
            .map_err(ralph_map_err!(
                codes::DB_WRITE,
                "Failed to insert discipline"
            ))?;

        Ok(())
    }

    pub fn update_discipline(
        &self,
        name: String,
        display_name: String,
        acronym: String,
        icon: String,
        color: String,
    ) -> Result<(), String> {
        if display_name.trim().is_empty() {
            return ralph_err!(
                codes::DISCIPLINE_OPS,
                "Discipline display name cannot be empty"
            );
        }

        crate::acronym::validate_acronym_format(&acronym)?;

        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM disciplines WHERE name = ?1",
                [&name],
                |row| row.get(0),
            )
            .map_err(ralph_map_err!(codes::DB_READ, "Failed to check discipline"))?;
        if !exists {
            return ralph_err!(codes::DISCIPLINE_OPS, "Discipline '{name}' does not exist");
        }

        let acronym_conflict: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM disciplines WHERE acronym = ?1 AND name != ?2",
                rusqlite::params![acronym, name],
                |row| row.get(0),
            )
            .map_err(ralph_map_err!(codes::DB_READ, "Failed to check acronym"))?;
        if acronym_conflict {
            return ralph_err!(
                codes::DISCIPLINE_OPS,
                "Acronym '{acronym}' is already used by another discipline"
            );
        }

        self.conn
            .execute(
                "UPDATE disciplines SET display_name = ?1, acronym = ?2, icon = ?3, color = ?4 WHERE name = ?5",
                rusqlite::params![display_name, acronym, icon, color, name],
            )
            .map_err(ralph_map_err!(codes::DB_WRITE, "Failed to update discipline"))?;

        Ok(())
    }

    pub fn delete_discipline(&self, name: String) -> Result<(), String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title FROM tasks WHERE discipline = ?1")
            .map_err(ralph_map_err!(codes::DB_READ, "Failed to prepare query"))?;

        let tasks: Vec<(u32, String)> = stmt
            .query_map([&name], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(ralph_map_err!(codes::DB_READ, "Failed to query tasks"))?
            .filter_map(std::result::Result::ok)
            .collect();

        if let Some((task_id, task_title)) = tasks.first() {
            return ralph_err!(
                codes::DISCIPLINE_OPS,
                "Cannot delete discipline '{name}': task {task_id} ('{task_title}') belongs to it"
            );
        }

        let affected = self
            .conn
            .execute("DELETE FROM disciplines WHERE name = ?1", [&name])
            .map_err(ralph_map_err!(
                codes::DB_WRITE,
                "Failed to delete discipline"
            ))?;

        if affected == 0 {
            return ralph_err!(codes::DISCIPLINE_OPS, "Discipline '{name}' does not exist");
        }

        Ok(())
    }

    pub fn get_disciplines(&self) -> Vec<Discipline> {
        let Ok(mut stmt) = self.conn.prepare(
            "SELECT name, display_name, acronym, icon, color, system_prompt, skills, \
             conventions, mcp_servers \
             FROM disciplines ORDER BY name",
        ) else {
            return vec![];
        };

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
        .map_or_else(
            |_| vec![],
            |rows| rows.filter_map(std::result::Result::ok).collect(),
        )
    }

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
            let exists: bool = self
                .conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM disciplines WHERE name = ?1",
                    [name],
                    |row| row.get(0),
                )
                .map_err(ralph_map_err!(codes::DB_READ, "Failed to check discipline"))?;

            if !exists {
                self.conn
                    .execute(
                        "INSERT INTO disciplines (name, display_name, acronym, icon, color) \
                         VALUES (?1, ?2, ?3, ?4, ?5)",
                        rusqlite::params![name, display_name, acronym, icon, color],
                    )
                    .map_err(ralph_map_err!(codes::DB_WRITE, "Failed to seed discipline"))?;
            }
        }

        Ok(())
    }
}
