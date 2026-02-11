use crate::types::FeatureComment;
use crate::SqliteDb;
use ralph_errors::{codes, ralph_err, RalphResultExt};
use rusqlite::OptionalExtension;
use std::collections::HashMap;

pub struct AddFeatureCommentInput {
    pub feature_name: String,
    pub category: String,
    pub discipline: Option<String>,
    pub agent_task_id: Option<u32>,
    pub body: String,
    pub summary: Option<String>,
    pub reason: Option<String>,
    pub source_iteration: Option<u32>,
}

impl SqliteDb {
    pub fn add_feature_comment(&self, input: AddFeatureCommentInput) -> Result<(), String> {
        if input.body.trim().is_empty() {
            return ralph_err!(codes::FEATURE_OPS, "Comment body cannot be empty");
        }
        if input.category.trim().is_empty() {
            return ralph_err!(codes::FEATURE_OPS, "Comment category cannot be empty");
        }

        let feature_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM features WHERE name = ?1",
                [&input.feature_name],
                |row| row.get(0),
            )
            .optional()
            .ralph_err(codes::DB_READ, "Failed to check feature")?;

        let feature_id = feature_id.ok_or_else(|| {
            format!(
                "[R-{}] Feature '{}' does not exist",
                codes::FEATURE_OPS,
                input.feature_name
            )
        })?;

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
                "INSERT INTO feature_comments \
                 (feature_id, category, discipline_id, agent_task_id, body, summary, reason, source_iteration, created) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    feature_id,
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
            .ralph_err(codes::DB_WRITE, "Failed to insert feature comment")?;

        Ok(())
    }

    pub fn update_feature_comment(
        &self,
        feature_name: &str,
        comment_id: u32,
        body: &str,
        summary: Option<String>,
        reason: Option<String>,
    ) -> Result<(), String> {
        if body.trim().is_empty() {
            return ralph_err!(codes::FEATURE_OPS, "Comment body cannot be empty");
        }

        let feature_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM features WHERE name = ?1",
                [feature_name],
                |row| row.get(0),
            )
            .optional()
            .ralph_err(codes::DB_READ, "Failed to check feature")?;

        let feature_id = feature_id.ok_or_else(|| {
            format!(
                "[R-{}] Feature '{}' does not exist",
                codes::FEATURE_OPS,
                feature_name
            )
        })?;

        let now = self.now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let affected = self
            .conn
            .execute(
                "UPDATE feature_comments SET body = ?1, summary = ?2, reason = ?3, updated = ?4 \
                 WHERE id = ?5 AND feature_id = ?6",
                rusqlite::params![body.trim(), summary, reason, now, comment_id, feature_id],
            )
            .ralph_err(codes::DB_WRITE, "Failed to update feature comment")?;

        if affected == 0 {
            return ralph_err!(
                codes::FEATURE_OPS,
                "Comment {comment_id} does not exist on feature '{feature_name}'"
            );
        }

        Ok(())
    }

    pub fn delete_feature_comment(
        &self,
        feature_name: &str,
        comment_id: u32,
    ) -> Result<(), String> {
        let feature_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM features WHERE name = ?1",
                [feature_name],
                |row| row.get(0),
            )
            .optional()
            .ralph_err(codes::DB_READ, "Failed to check feature")?;

        let feature_id = feature_id.ok_or_else(|| {
            format!(
                "[R-{}] Feature '{}' does not exist",
                codes::FEATURE_OPS,
                feature_name
            )
        })?;

        let affected = self
            .conn
            .execute(
                "DELETE FROM feature_comments WHERE id = ?1 AND feature_id = ?2",
                rusqlite::params![comment_id, feature_id],
            )
            .ralph_err(codes::DB_WRITE, "Failed to delete feature comment")?;

        if affected == 0 {
            return ralph_err!(
                codes::FEATURE_OPS,
                "Comment {comment_id} does not exist on feature '{feature_name}'"
            );
        }

        Ok(())
    }

    pub(crate) fn get_all_comments_by_feature(&self) -> HashMap<String, Vec<FeatureComment>> {
        let Ok(mut stmt) = self.conn.prepare(
            "SELECT fc.id, f.name, fc.category, d.name, fc.agent_task_id, fc.body, fc.summary, fc.reason, fc.source_iteration, fc.created, fc.updated \
             FROM feature_comments fc \
             JOIN features f ON fc.feature_id = f.id \
             LEFT JOIN disciplines d ON fc.discipline_id = d.id \
             ORDER BY f.name, fc.id DESC",
        ) else {
            return HashMap::new();
        };

        let mut map: HashMap<String, Vec<FeatureComment>> = HashMap::new();

        let Ok(rows) = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(1)?,
                FeatureComment {
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
