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

        if self.check_exists("disciplines", "name", &input.name)? {
            return ralph_err!(
                codes::DISCIPLINE_OPS,
                "Discipline '{}' already exists",
                input.name
            );
        }

        if self.check_exists("disciplines", "acronym", &input.acronym)? {
            return ralph_err!(
                codes::DISCIPLINE_OPS,
                "Acronym '{}' is already used by another discipline",
                input.acronym
            );
        }

        self.conn
            .execute(
                "INSERT INTO disciplines (name, display_name, acronym, icon, color, \
                 description, system_prompt, agent, model, effort, thinking, conventions, \
                 stack_id, image_path, crops, image_prompt) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, NULL, ?13, ?14, ?15)",
                rusqlite::params![
                    input.name,
                    input.display_name,
                    input.acronym,
                    input.icon,
                    input.color,
                    input.description,
                    input.system_prompt,
                    input.agent,
                    input.model,
                    input.effort,
                    input.thinking,
                    input.conventions,
                    input.image_path,
                    input.crops,
                    input.image_prompt
                ],
            )
            .ralph_err(codes::DB_WRITE, "Failed to insert discipline")?;

        let discipline_id = self.conn.last_insert_rowid();

        let skills: Vec<String> = serde_json::from_str(&input.skills).unwrap_or_default();
        self.insert_string_list(
            "discipline_skills",
            "discipline_id",
            discipline_id,
            "skill",
            &skills,
        )?;

        let mcp_servers: Vec<McpServerConfig> =
            serde_json::from_str(&input.mcp_servers).unwrap_or_default();
        self.insert_mcp_servers(discipline_id, &mcp_servers)?;

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

        if !self.check_exists("disciplines", "name", &input.name)? {
            return ralph_err!(
                codes::DISCIPLINE_OPS,
                "Discipline '{}' does not exist",
                input.name
            );
        }

        if self.check_exists_excluding(
            "disciplines",
            "acronym",
            &input.acronym,
            "name",
            &input.name,
        )? {
            return ralph_err!(
                codes::DISCIPLINE_OPS,
                "Acronym '{}' is already used by another discipline",
                input.acronym
            );
        }

        let discipline_id: i64 = self
            .conn
            .query_row(
                "SELECT id FROM disciplines WHERE name = ?1",
                [&input.name],
                |row| row.get(0),
            )
            .ralph_err(codes::DB_READ, "Failed to get discipline ID")?;

        self.conn
            .execute(
                "UPDATE disciplines SET display_name = ?1, acronym = ?2, icon = ?3, color = ?4, \
                 description = ?5, system_prompt = ?6, agent = ?7, model = ?8, effort = ?9, \
                 thinking = ?10, conventions = ?11, image_path = ?12, \
                 crops = ?13, image_prompt = ?14 WHERE name = ?15",
                rusqlite::params![
                    input.display_name,
                    input.acronym,
                    input.icon,
                    input.color,
                    input.description,
                    input.system_prompt,
                    input.agent,
                    input.model,
                    input.effort,
                    input.thinking,
                    input.conventions,
                    input.image_path,
                    input.crops,
                    input.image_prompt,
                    input.name
                ],
            )
            .ralph_err(codes::DB_WRITE, "Failed to update discipline")?;

        self.conn
            .execute(
                "DELETE FROM discipline_skills WHERE discipline_id = ?1",
                [discipline_id],
            )
            .ralph_err(codes::DB_WRITE, "Failed to delete old skills")?;

        self.conn
            .execute(
                "DELETE FROM discipline_mcp_servers WHERE discipline_id = ?1",
                [discipline_id],
            )
            .ralph_err(codes::DB_WRITE, "Failed to delete old MCP servers")?;

        let skills: Vec<String> = serde_json::from_str(&input.skills).unwrap_or_default();
        self.insert_string_list(
            "discipline_skills",
            "discipline_id",
            discipline_id,
            "skill",
            &skills,
        )?;

        let mcp_servers: Vec<McpServerConfig> =
            serde_json::from_str(&input.mcp_servers).unwrap_or_default();
        self.insert_mcp_servers(discipline_id, &mcp_servers)?;

        Ok(())
    }

    pub fn delete_discipline(&self, name: String) -> Result<(), String> {
        let discipline_id = self.get_id_from_name("disciplines", &name)?;

        let mut stmt = self
            .conn
            .prepare(
                "SELECT rt.id, td.title \
                 FROM runtime_tasks rt \
                 JOIN task_details td ON rt.details_id = td.id \
                 WHERE td.discipline_id = ?1",
            )
            .ralph_err(codes::DB_READ, "Failed to prepare query")?;

        let tasks: Vec<(u32, String)> = stmt
            .query_map([discipline_id], |row| Ok((row.get(0)?, row.get(1)?)))
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
            "SELECT id, name, display_name, acronym, icon, color, description, system_prompt, \
             agent, model, effort, thinking, conventions, stack_id, image_path, crops, image_prompt \
             FROM disciplines ORDER BY rowid",
        ) else {
            return vec![];
        };

        let Ok(rows) = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                Discipline {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    display_name: row.get(2)?,
                    acronym: row.get(3)?,
                    icon: row.get(4)?,
                    color: row.get(5)?,
                    description: row.get(6)?,
                    system_prompt: row.get(7)?,
                    agent: row.get(8)?,
                    model: row.get(9)?,
                    effort: row.get(10)?,
                    thinking: row.get(11)?,
                    skills: vec![],
                    conventions: row.get(12)?,
                    mcp_servers: vec![],
                    stack_id: row.get(13)?,
                    image_path: row.get(14)?,
                    crops: row.get(15)?,
                    image_prompt: row.get(16)?,
                },
            ))
        }) else {
            return vec![];
        };

        let mut disciplines: Vec<Discipline> = vec![];
        for row in rows.filter_map(std::result::Result::ok) {
            let (discipline_id, mut discipline) = row;
            discipline.skills =
                self.read_string_list("discipline_skills", "discipline_id", discipline_id, "skill");
            discipline.mcp_servers = self.read_mcp_servers(discipline_id);
            disciplines.push(discipline);
        }

        disciplines
    }
}
