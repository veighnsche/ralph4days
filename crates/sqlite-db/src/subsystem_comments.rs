use crate::types::SubsystemComment;
use crate::SqliteDb;
use ralph_errors::{codes, ralph_err, RalphResultExt};
use rusqlite::OptionalExtension;
use std::collections::HashMap;

pub struct AddSubsystemCommentInput {
    pub subsystem_name: String,
    pub category: String,
    pub discipline: Option<String>,
    pub agent_task_id: Option<u32>,
    pub body: String,
    pub summary: Option<String>,
    pub reason: Option<String>,
    pub source_iteration: Option<u32>,
}

impl SqliteDb {
    fn resolve_subsystem_id(&self, subsystem_name: &str) -> Result<i64, String> {
        let subsystem_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM subsystems WHERE name = ?1",
                [subsystem_name],
                |row| row.get(0),
            )
            .optional()
            .ralph_err(codes::DB_READ, "Failed to check subsystem")?;

        subsystem_id.ok_or_else(|| {
            format!(
                "[R-{}] Subsystem '{}' does not exist",
                codes::FEATURE_OPS,
                subsystem_name
            )
        })
    }

    pub fn add_subsystem_comment(&self, input: AddSubsystemCommentInput) -> Result<(), String> {
        if input.body.trim().is_empty() {
            return ralph_err!(codes::FEATURE_OPS, "Comment body cannot be empty");
        }
        if input.category.trim().is_empty() {
            return ralph_err!(codes::FEATURE_OPS, "Comment category cannot be empty");
        }

        let subsystem_id = self.resolve_subsystem_id(&input.subsystem_name)?;

        let discipline_id: Option<i64> = if let Some(ref disc) = input.discipline {
            self.conn
                .query_row(
                    "SELECT id FROM disciplines WHERE name = ?1",
                    [disc],
                    |row| row.get(0),
                )
                .optional()
                .ralph_err(codes::DB_READ, "Failed to check discipline")?
        } else {
            None
        };

        let now = self.now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        self.conn
            .execute(
                "INSERT INTO subsystem_comments \
                 (subsystem_id, category, discipline_id, agent_task_id, body, summary, reason, source_iteration, created) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    subsystem_id,
                    input.category.trim(),
                    discipline_id,
                    input.agent_task_id,
                    input.body.trim(),
                    input.summary,
                    input.reason,
                    input.source_iteration,
                    now,
                ],
            )
            .ralph_err(codes::DB_WRITE, "Failed to insert subsystem comment")?;

        Ok(())
    }

    pub fn update_subsystem_comment(
        &self,
        subsystem_name: &str,
        comment_id: u32,
        body: &str,
        summary: Option<String>,
        reason: Option<String>,
    ) -> Result<(), String> {
        if body.trim().is_empty() {
            return ralph_err!(codes::FEATURE_OPS, "Comment body cannot be empty");
        }

        let subsystem_id = self.resolve_subsystem_id(subsystem_name)?;

        let now = self.now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let affected = self
            .conn
            .execute(
                "UPDATE subsystem_comments SET body = ?1, summary = ?2, reason = ?3, updated = ?4 \
                 WHERE id = ?5 AND subsystem_id = ?6",
                rusqlite::params![body.trim(), summary, reason, now, comment_id, subsystem_id],
            )
            .ralph_err(codes::DB_WRITE, "Failed to update subsystem comment")?;

        if affected == 0 {
            return ralph_err!(
                codes::FEATURE_OPS,
                "Comment {comment_id} does not exist on subsystem '{subsystem_name}'"
            );
        }

        Ok(())
    }

    pub fn delete_subsystem_comment(
        &self,
        subsystem_name: &str,
        comment_id: u32,
    ) -> Result<(), String> {
        let subsystem_id = self.resolve_subsystem_id(subsystem_name)?;

        let affected = self
            .conn
            .execute(
                "DELETE FROM subsystem_comments WHERE id = ?1 AND subsystem_id = ?2",
                rusqlite::params![comment_id, subsystem_id],
            )
            .ralph_err(codes::DB_WRITE, "Failed to delete subsystem comment")?;

        if affected == 0 {
            return ralph_err!(
                codes::FEATURE_OPS,
                "Comment {comment_id} does not exist on subsystem '{subsystem_name}'"
            );
        }

        Ok(())
    }

    pub(crate) fn get_all_comments_by_subsystem(&self) -> HashMap<String, Vec<SubsystemComment>> {
        let Ok(mut stmt) = self.conn.prepare(
            "SELECT fc.id, f.name, fc.category, d.name, fc.agent_task_id, fc.body, fc.summary, fc.reason, fc.source_iteration, fc.created, fc.updated \
             FROM subsystem_comments fc \
             JOIN subsystems f ON fc.subsystem_id = f.id \
             LEFT JOIN disciplines d ON fc.discipline_id = d.id \
             ORDER BY f.name, fc.id DESC",
        ) else {
            return HashMap::new();
        };

        let mut map: HashMap<String, Vec<SubsystemComment>> = HashMap::new();

        let Ok(rows) = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(1)?,
                SubsystemComment {
                    id: row.get(0)?,
                    category: row.get(2)?,
                    discipline: row.get(3)?,
                    agent_task_id: row.get(4)?,
                    body: row.get(5)?,
                    summary: row.get(6)?,
                    reason: row.get(7)?,
                    source_iteration: row.get(8)?,
                    created: row.get(9)?,
                    updated: row.get(10)?,
                },
            ))
        }) else {
            return HashMap::new();
        };

        for row in rows.flatten() {
            map.entry(row.0).or_default().push(row.1);
        }

        map
    }
}
