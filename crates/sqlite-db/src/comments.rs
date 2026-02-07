use crate::types::*;
use crate::SqliteDb;
use std::collections::HashMap;

impl SqliteDb {
    /// Add a comment to an existing task.
    pub fn add_comment(
        &self,
        task_id: u32,
        author: CommentAuthor,
        agent_task_id: Option<u32>,
        body: String,
    ) -> Result<(), String> {
        if body.trim().is_empty() {
            return Err("Comment body cannot be empty".to_owned());
        }
        if author == CommentAuthor::Agent && agent_task_id.is_none() {
            return Err("agent_task_id is required for agent comments".to_owned());
        }
        if author == CommentAuthor::Human && agent_task_id.is_some() {
            return Err("agent_task_id must not be set for human comments".to_owned());
        }

        // Verify task exists
        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM tasks WHERE id = ?1",
                [task_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check task: {e}"))?;
        if !exists {
            return Err(format!("Task {task_id} does not exist"));
        }

        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        self.conn
            .execute(
                "INSERT INTO task_comments (task_id, author, agent_task_id, body, created) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![task_id, author.as_str(), agent_task_id, body, now],
            )
            .map_err(|e| format!("Failed to insert comment: {e}"))?;

        Ok(())
    }

    /// Update a comment's body by stable ID (not array index).
    pub fn update_comment(
        &self,
        task_id: u32,
        comment_id: u32,
        body: String,
    ) -> Result<(), String> {
        if body.trim().is_empty() {
            return Err("Comment body cannot be empty".to_owned());
        }

        // Verify task exists
        let task_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM tasks WHERE id = ?1",
                [task_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check task: {e}"))?;
        if !task_exists {
            return Err(format!("Task {task_id} does not exist"));
        }

        let affected = self
            .conn
            .execute(
                "UPDATE task_comments SET body = ?1 WHERE id = ?2 AND task_id = ?3",
                rusqlite::params![body, comment_id, task_id],
            )
            .map_err(|e| format!("Failed to update comment: {e}"))?;

        if affected == 0 {
            return Err(format!(
                "Comment {comment_id} does not exist on task {task_id}"
            ));
        }

        Ok(())
    }

    /// Delete a comment by stable ID (not array index).
    pub fn delete_comment(&self, task_id: u32, comment_id: u32) -> Result<(), String> {
        // Verify task exists
        let task_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM tasks WHERE id = ?1",
                [task_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check task: {e}"))?;
        if !task_exists {
            return Err(format!("Task {task_id} does not exist"));
        }

        let affected = self
            .conn
            .execute(
                "DELETE FROM task_comments WHERE id = ?1 AND task_id = ?2",
                rusqlite::params![comment_id, task_id],
            )
            .map_err(|e| format!("Failed to delete comment: {e}"))?;

        if affected == 0 {
            return Err(format!(
                "Comment {comment_id} does not exist on task {task_id}"
            ));
        }

        Ok(())
    }

    /// Get comments for a single task.
    pub(crate) fn get_comments_for_task(&self, task_id: u32) -> Vec<TaskComment> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, author, agent_task_id, body, created \
                 FROM task_comments WHERE task_id = ?1 ORDER BY id",
            )
            .unwrap();

        stmt.query_map([task_id], |row| {
            let author_str: String = row.get(1)?;
            Ok(TaskComment {
                id: row.get(0)?,
                author: CommentAuthor::parse(&author_str).unwrap_or(CommentAuthor::Human),
                agent_task_id: row.get(2)?,
                body: row.get(3)?,
                created: row.get(4)?,
            })
        })
        .unwrap()
        .filter_map(std::result::Result::ok)
        .collect()
    }

    /// Get all comments keyed by task_id (for bulk loading).
    pub(crate) fn get_all_comments_by_task(&self) -> HashMap<u32, Vec<TaskComment>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, task_id, author, agent_task_id, body, created \
                 FROM task_comments ORDER BY task_id, id",
            )
            .unwrap();

        let mut map: HashMap<u32, Vec<TaskComment>> = HashMap::new();

        let rows = stmt
            .query_map([], |row| {
                let author_str: String = row.get(2)?;
                Ok((
                    row.get::<_, u32>(1)?,
                    TaskComment {
                        id: row.get(0)?,
                        author: CommentAuthor::parse(&author_str).unwrap_or(CommentAuthor::Human),
                        agent_task_id: row.get(3)?,
                        body: row.get(4)?,
                        created: row.get(5)?,
                    },
                ))
            })
            .unwrap();

        for row in rows.flatten() {
            map.entry(row.0).or_default().push(row.1);
        }

        map
    }
}
