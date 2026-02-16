use crate::types::*;
use crate::SqliteDb;
use core_errors::{codes, ralph_err, RalphResult, RalphResultExt};

impl SqliteDb {
    pub fn create_subsystem(&self, input: SubsystemInput) -> RalphResult<()> {
        if input.name.trim().is_empty() {
            return ralph_err!(codes::FEATURE_OPS, "Subsystem name cannot be empty");
        }
        if input.display_name.trim().is_empty() {
            return ralph_err!(codes::FEATURE_OPS, "Subsystem display name cannot be empty");
        }

        crate::acronym::validate_acronym_format(&input.acronym)?;

        if self.check_exists("subsystems", "name", &input.name)? {
            return ralph_err!(
                codes::FEATURE_OPS,
                "Subsystem '{}' already exists",
                input.name
            );
        }

        if self.check_exists("subsystems", "acronym", &input.acronym)? {
            return ralph_err!(
                codes::FEATURE_OPS,
                "Acronym '{}' is already used by another subsystem",
                input.acronym
            );
        }

        let now = self.now().format("%Y-%m-%d").to_string();

        self.conn
            .execute(
                "INSERT INTO subsystems (name, display_name, acronym, description, created, status) \
                 VALUES (?1, ?2, ?3, ?4, ?5, 'active')",
                rusqlite::params![
                    input.name,
                    input.display_name,
                    input.acronym,
                    input.description,
                    now,
                ],
            )
            .ralph_err(codes::DB_WRITE, "Failed to insert subsystem")?;

        Ok(())
    }

    pub fn update_subsystem(&self, input: SubsystemInput) -> RalphResult<()> {
        if input.display_name.trim().is_empty() {
            return ralph_err!(codes::FEATURE_OPS, "Subsystem display name cannot be empty");
        }

        crate::acronym::validate_acronym_format(&input.acronym)?;

        if !self.check_exists("subsystems", "name", &input.name)? {
            return ralph_err!(
                codes::FEATURE_OPS,
                "Subsystem '{}' does not exist",
                input.name
            );
        }

        if self.check_exists_excluding(
            "subsystems",
            "acronym",
            &input.acronym,
            "name",
            &input.name,
        )? {
            return ralph_err!(
                codes::FEATURE_OPS,
                "Acronym '{}' is already used by another subsystem",
                input.acronym
            );
        }

        self.conn
            .execute(
                "UPDATE subsystems SET display_name = ?1, acronym = ?2, description = ?3 WHERE name = ?4",
                rusqlite::params![
                    input.display_name,
                    input.acronym,
                    input.description,
                    input.name,
                ],
            )
            .ralph_err(codes::DB_WRITE, "Failed to update subsystem")?;

        Ok(())
    }

    pub fn delete_subsystem(&self, name: String) -> RalphResult<()> {
        let subsystem_id = self.get_id_from_name("subsystems", &name)?;

        let mut stmt = self
            .conn
            .prepare(
                "SELECT rt.id, td.title \
                 FROM runtime_tasks rt \
                 JOIN task_details td ON rt.details_id = td.id \
                 WHERE rt.subsystem_id = ?1",
            )
            .ralph_err(codes::DB_READ, "Failed to prepare query")?;

        let tasks = stmt
            .query_map([subsystem_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .ralph_err(codes::DB_READ, "Failed to query tasks")?;

        let mut tasks_out: Vec<(u32, String)> = Vec::new();
        for row in tasks {
            tasks_out.push(row.ralph_err(codes::DB_READ, "Failed to decode task row")?);
        }

        if let Some((task_id, task_title)) = tasks_out.first() {
            return ralph_err!(
                codes::FEATURE_OPS,
                "Cannot delete subsystem '{name}': task {task_id} ('{task_title}') belongs to it"
            );
        }

        let affected = self
            .conn
            .execute("DELETE FROM subsystems WHERE name = ?1", [&name])
            .ralph_err(codes::DB_WRITE, "Failed to delete subsystem")?;

        if affected == 0 {
            return ralph_err!(codes::FEATURE_OPS, "Subsystem '{name}' does not exist");
        }

        Ok(())
    }

    pub fn get_subsystems(&self) -> RalphResult<Vec<Subsystem>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, display_name, acronym, description, created, status \
             FROM subsystems ORDER BY name",
            )
            .ralph_err(codes::DB_READ, "Failed to prepare subsystems list query")?;

        let mut comments_map = self.get_all_comments_by_subsystem()?;

        let rows = stmt
            .query_map([], |row| {
                let status_str: String = row.get(6)?;
                let name: String = row.get(1)?;
                let status = SubsystemStatus::parse(&status_str).ok_or_else(|| {
                    rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Invalid subsystem status '{status_str}'"),
                        )),
                    )
                })?;
                Ok(Subsystem {
                    id: row.get(0)?,
                    name,
                    display_name: row.get(2)?,
                    acronym: row.get(3)?,
                    description: row.get(4)?,
                    created: row.get(5)?,
                    status,
                    comments: vec![],
                })
            })
            .ralph_err(codes::DB_READ, "Failed to query subsystems")?;

        let mut out = Vec::new();
        for row in rows {
            let mut subsystem = row.ralph_err(codes::DB_READ, "Failed to decode subsystem row")?;
            subsystem.comments = comments_map.remove(&subsystem.name).unwrap_or_default();
            out.push(subsystem);
        }

        Ok(out)
    }
}
