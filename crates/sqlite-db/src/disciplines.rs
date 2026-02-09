use crate::types::*;
use crate::SqliteDb;
use ralph_errors::{codes, ralph_err, RalphResultExt};

impl SqliteDb {
    pub fn create_discipline(&self, input: crate::types::DisciplineInput) -> Result<(), String> {
        if input.name.trim().is_empty() {
            return ralph_err!(codes::DISCIPLINE_OPS, "Discipline name cannot be empty");
        }
        if input.display_name.trim().is_empty() {
            return ralph_err!(
                codes::DISCIPLINE_OPS,
                "Discipline display name cannot be empty"
            );
        }

        crate::acronym::validate_acronym_format(&input.acronym)?;

        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM disciplines WHERE name = ?1",
                [&input.name],
                |row| row.get(0),
            )
            .ralph_err(codes::DB_READ, "Failed to check discipline")?;
        if exists {
            return ralph_err!(
                codes::DISCIPLINE_OPS,
                "Discipline '{}' already exists",
                input.name
            );
        }

        let acronym_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM disciplines WHERE acronym = ?1",
                [&input.acronym],
                |row| row.get(0),
            )
            .ralph_err(codes::DB_READ, "Failed to check acronym")?;
        if acronym_exists {
            return ralph_err!(
                codes::DISCIPLINE_OPS,
                "Acronym '{}' is already used by another discipline",
                input.acronym
            );
        }

        self.conn
            .execute(
                "INSERT INTO disciplines (name, display_name, acronym, icon, color, \
                 description, system_prompt, skills, conventions, mcp_servers, stack_id, \
                 image_path, crops, image_prompt) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, NULL, ?11, ?12, ?13)",
                rusqlite::params![
                    input.name,
                    input.display_name,
                    input.acronym,
                    input.icon,
                    input.color,
                    input.description,
                    input.system_prompt,
                    input.skills,
                    input.conventions,
                    input.mcp_servers,
                    input.image_path,
                    input.crops,
                    input.image_prompt
                ],
            )
            .ralph_err(codes::DB_WRITE, "Failed to insert discipline")?;

        Ok(())
    }

    pub fn update_discipline(&self, input: crate::types::DisciplineInput) -> Result<(), String> {
        if input.display_name.trim().is_empty() {
            return ralph_err!(
                codes::DISCIPLINE_OPS,
                "Discipline display name cannot be empty"
            );
        }

        crate::acronym::validate_acronym_format(&input.acronym)?;

        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM disciplines WHERE name = ?1",
                [&input.name],
                |row| row.get(0),
            )
            .ralph_err(codes::DB_READ, "Failed to check discipline")?;
        if !exists {
            return ralph_err!(
                codes::DISCIPLINE_OPS,
                "Discipline '{}' does not exist",
                input.name
            );
        }

        let acronym_conflict: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM disciplines WHERE acronym = ?1 AND name != ?2",
                rusqlite::params![input.acronym, input.name],
                |row| row.get(0),
            )
            .ralph_err(codes::DB_READ, "Failed to check acronym")?;
        if acronym_conflict {
            return ralph_err!(
                codes::DISCIPLINE_OPS,
                "Acronym '{}' is already used by another discipline",
                input.acronym
            );
        }

        self.conn
            .execute(
                "UPDATE disciplines SET display_name = ?1, acronym = ?2, icon = ?3, color = ?4, \
                 description = ?5, system_prompt = ?6, skills = ?7, conventions = ?8, \
                 mcp_servers = ?9, image_path = ?10, crops = ?11, image_prompt = ?12 \
                 WHERE name = ?13",
                rusqlite::params![
                    input.display_name,
                    input.acronym,
                    input.icon,
                    input.color,
                    input.description,
                    input.system_prompt,
                    input.skills,
                    input.conventions,
                    input.mcp_servers,
                    input.image_path,
                    input.crops,
                    input.image_prompt,
                    input.name
                ],
            )
            .ralph_err(codes::DB_WRITE, "Failed to update discipline")?;

        Ok(())
    }

    pub fn delete_discipline(&self, name: String) -> Result<(), String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title FROM tasks WHERE discipline = ?1")
            .ralph_err(codes::DB_READ, "Failed to prepare query")?;

        let tasks: Vec<(u32, String)> = stmt
            .query_map([&name], |row| Ok((row.get(0)?, row.get(1)?)))
            .ralph_err(codes::DB_READ, "Failed to query tasks")?
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
            .ralph_err(codes::DB_WRITE, "Failed to delete discipline")?;

        if affected == 0 {
            return ralph_err!(codes::DISCIPLINE_OPS, "Discipline '{name}' does not exist");
        }

        Ok(())
    }

    pub fn get_disciplines(&self) -> Vec<Discipline> {
        let Ok(mut stmt) = self.conn.prepare(
            "SELECT name, display_name, acronym, icon, color, description, system_prompt, skills, \
             conventions, mcp_servers, stack_id, image_path, crops, image_prompt \
             FROM disciplines ORDER BY rowid",
        ) else {
            return vec![];
        };

        stmt.query_map([], |row| {
            let skills_json: String = row.get(7)?;
            let mcp_json: String = row.get(9)?;
            Ok(Discipline {
                name: row.get(0)?,
                display_name: row.get(1)?,
                acronym: row.get(2)?,
                icon: row.get(3)?,
                color: row.get(4)?,
                description: row.get(5)?,
                system_prompt: row.get(6)?,
                skills: serde_json::from_str(&skills_json).unwrap_or_default(),
                conventions: row.get(8)?,
                mcp_servers: serde_json::from_str(&mcp_json).unwrap_or_default(),
                stack_id: row.get(10)?,
                image_path: row.get(11)?,
                crops: row.get(12)?,
                image_prompt: row.get(13)?,
            })
        })
        .map_or_else(
            |_| vec![],
            |rows| rows.filter_map(std::result::Result::ok).collect(),
        )
    }
}
