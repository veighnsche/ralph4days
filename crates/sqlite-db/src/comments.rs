use crate::types::{TaskComment, TaskSignalSummary};
use crate::SqliteDb;
use ralph_errors::{codes, ralph_err, RalphResultExt};
use std::collections::HashMap;

impl SqliteDb {
    pub fn add_comment(
        &self,
        task_id: u32,
        discipline: Option<String>,
        _agent_task_id: Option<u32>,
        priority: Option<String>,
        body: String,
    ) -> Result<(), String> {
        self.add_comment_with_parent(task_id, discipline, priority, body, None)
    }

    pub fn add_comment_with_parent(
        &self,
        task_id: u32,
        discipline: Option<String>,
        priority: Option<String>,
        body: String,
        parent_comment_id: Option<u32>,
    ) -> Result<(), String> {
        if body.trim().is_empty() {
            return ralph_err!(codes::COMMENT_OPS, "Comment body cannot be empty");
        }

        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM tasks WHERE id = ?1",
                [task_id],
                |row| row.get(0),
            )
            .ralph_err(codes::DB_READ, "Failed to check task")?;
        if !exists {
            return ralph_err!(codes::COMMENT_OPS, "Task {task_id} does not exist");
        }

        if let Some(parent_id) = parent_comment_id {
            let parent_exists: bool = self
                .conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM task_comments WHERE id = ?1 AND task_id = ?2",
                    rusqlite::params![parent_id, task_id],
                    |row| row.get(0),
                )
                .ralph_err(codes::DB_READ, "Failed to check parent comment")?;
            if !parent_exists {
                return ralph_err!(
                    codes::COMMENT_OPS,
                    "Parent comment {parent_id} does not exist"
                );
            }

            let parent_has_parent: bool = self
                .conn
                .query_row(
                    "SELECT parent_comment_id IS NOT NULL FROM task_comments WHERE id = ?1",
                    [parent_id],
                    |row| row.get(0),
                )
                .ralph_err(codes::DB_READ, "Failed to check parent nesting")?;
            if parent_has_parent {
                return ralph_err!(codes::COMMENT_OPS, "Cannot reply to a reply (max 2 layers)");
            }
        }

        let now = self.now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let author = if parent_comment_id.is_some() {
            "human".to_owned()
        } else {
            discipline.unwrap_or_else(|| "human".to_owned())
        };

        self.conn
            .execute(
                "INSERT INTO task_comments (task_id, author, body, created, parent_comment_id, priority) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![task_id, author, body, now, parent_comment_id, priority],
            )
            .ralph_err(codes::DB_WRITE, "Failed to insert comment")?;

        Ok(())
    }

    pub fn update_comment(
        &self,
        task_id: u32,
        comment_id: u32,
        body: String,
    ) -> Result<(), String> {
        if body.trim().is_empty() {
            return ralph_err!(codes::COMMENT_OPS, "Comment body cannot be empty");
        }

        let task_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM tasks WHERE id = ?1",
                [task_id],
                |row| row.get(0),
            )
            .ralph_err(codes::DB_READ, "Failed to check task")?;
        if !task_exists {
            return ralph_err!(codes::COMMENT_OPS, "Task {task_id} does not exist");
        }

        let affected = self
            .conn
            .execute(
                "UPDATE task_comments SET body = ?1 WHERE id = ?2 AND task_id = ?3",
                rusqlite::params![body, comment_id, task_id],
            )
            .ralph_err(codes::DB_WRITE, "Failed to update comment")?;

        if affected == 0 {
            return ralph_err!(
                codes::COMMENT_OPS,
                "Comment {comment_id} does not exist on task {task_id}"
            );
        }

        Ok(())
    }

    pub fn delete_comment(&self, task_id: u32, comment_id: u32) -> Result<(), String> {
        let task_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM tasks WHERE id = ?1",
                [task_id],
                |row| row.get(0),
            )
            .ralph_err(codes::DB_READ, "Failed to check task")?;
        if !task_exists {
            return ralph_err!(codes::COMMENT_OPS, "Task {task_id} does not exist");
        }

        let affected = self
            .conn
            .execute(
                "DELETE FROM task_comments WHERE id = ?1 AND task_id = ?2",
                rusqlite::params![comment_id, task_id],
            )
            .ralph_err(codes::DB_WRITE, "Failed to delete comment")?;

        if affected == 0 {
            return ralph_err!(
                codes::COMMENT_OPS,
                "Comment {comment_id} does not exist on task {task_id}"
            );
        }

        Ok(())
    }

    pub(crate) fn get_comments_for_task(&self, task_id: u32) -> Vec<TaskComment> {
        let Ok(mut stmt) = self.conn.prepare(
            "SELECT id, author, body, created, session_id, signal_verb, signal_payload, signal_answered, parent_comment_id, priority \
             FROM task_comments WHERE task_id = ?1 AND signal_verb IS NULL ORDER BY id DESC",
        ) else {
            return vec![];
        };

        stmt.query_map([task_id], |row| {
            Ok(TaskComment {
                id: row.get(0)?,
                author: row.get(1)?,
                body: row.get(2)?,
                created: row.get(3)?,
                session_id: row.get(4)?,
                signal_verb: row.get(5)?,
                signal_payload: row.get(6)?,
                signal_answered: row.get(7)?,
                parent_comment_id: row.get(8)?,
                priority: row.get(9)?,
            })
        })
        .map_or_else(
            |_| vec![],
            |rows| rows.filter_map(std::result::Result::ok).collect(),
        )
    }

    pub(crate) fn get_all_comments_by_task(&self) -> HashMap<u32, Vec<TaskComment>> {
        let Ok(mut stmt) = self.conn.prepare(
            "SELECT tc.id, tc.task_id, COALESCE(d.display_name, tc.author) as author, tc.body, tc.created, tc.session_id, tc.signal_verb, tc.signal_payload, tc.signal_answered, tc.parent_comment_id, tc.priority \
             FROM task_comments tc \
             LEFT JOIN disciplines d ON tc.author = d.name \
             ORDER BY tc.task_id, tc.id DESC",
        ) else {
            return HashMap::new();
        };

        let mut map: HashMap<u32, Vec<TaskComment>> = HashMap::new();

        let Ok(rows) = stmt.query_map([], |row| {
            Ok((
                row.get::<_, u32>(1)?,
                TaskComment {
                    id: row.get(0)?,
                    author: row.get(2)?,
                    body: row.get(3)?,
                    created: row.get(4)?,
                    session_id: row.get(5)?,
                    signal_verb: row.get(6)?,
                    signal_payload: row.get(7)?,
                    signal_answered: row.get(8)?,
                    parent_comment_id: row.get(9)?,
                    priority: row.get(10)?,
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

    pub fn get_task_signals(&self, task_id: u32) -> Result<Vec<TaskComment>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, author, body, created, session_id, signal_verb, signal_payload, signal_answered, parent_comment_id, priority \
                 FROM task_comments WHERE task_id = ?1 AND signal_verb IS NOT NULL ORDER BY created ASC, id ASC",
            )
            .ralph_err(codes::DB_READ, "Failed to prepare signal query")?;

        let rows = stmt
            .query_map([task_id], |row| {
                Ok(TaskComment {
                    id: row.get(0)?,
                    author: row.get(1)?,
                    body: row.get(2)?,
                    created: row.get(3)?,
                    session_id: row.get(4)?,
                    signal_verb: row.get(5)?,
                    signal_payload: row.get(6)?,
                    signal_answered: row.get(7)?,
                    parent_comment_id: row.get(8)?,
                    priority: row.get(9)?,
                })
            })
            .ralph_err(codes::DB_READ, "Failed to query signals")?;

        Ok(rows.filter_map(std::result::Result::ok).collect())
    }

    pub fn get_signal_summaries(
        &self,
        task_ids: &[u32],
    ) -> Result<HashMap<u32, TaskSignalSummary>, String> {
        if task_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let placeholders: Vec<String> = task_ids.iter().map(|_| "?".to_owned()).collect();
        let sql = format!(
            "SELECT task_id, signal_verb, signal_payload, signal_answered, session_id \
             FROM task_comments WHERE task_id IN ({}) AND signal_verb IS NOT NULL ORDER BY task_id, created ASC",
            placeholders.join(",")
        );

        let mut stmt = self
            .conn
            .prepare(&sql)
            .ralph_err(codes::DB_READ, "Failed to prepare summary query")?;

        let params: Vec<Box<dyn rusqlite::types::ToSql>> = task_ids
            .iter()
            .map(|id| Box::new(*id) as Box<dyn rusqlite::types::ToSql>)
            .collect();
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params.iter().map(std::convert::AsRef::as_ref).collect();

        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok((
                    row.get::<_, u32>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })
            .ralph_err(codes::DB_READ, "Failed to query signal summaries")?;

        let mut map: HashMap<u32, TaskSignalSummary> = HashMap::new();
        let mut sessions_per_task: HashMap<u32, std::collections::HashSet<String>> = HashMap::new();

        for row in rows.flatten() {
            let (task_id, verb, payload, answered, session_id) = row;
            let summary = map.entry(task_id).or_insert_with(|| TaskSignalSummary {
                pending_asks: 0,
                flag_count: 0,
                max_flag_severity: None,
                last_closing_verb: None,
                session_count: 0,
                learned_count: 0,
            });

            sessions_per_task
                .entry(task_id)
                .or_default()
                .insert(session_id);

            match verb.as_str() {
                "ask" => {
                    if answered.is_none() {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&payload) {
                            if parsed
                                .get("blocking")
                                .and_then(serde_json::Value::as_bool)
                                .unwrap_or(false)
                            {
                                summary.pending_asks += 1;
                            }
                        }
                    }
                }
                "flag" => {
                    summary.flag_count += 1;
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&payload) {
                        if let Some(sev) = parsed.get("severity").and_then(|v| v.as_str()) {
                            let current_rank = severity_rank(summary.max_flag_severity.as_deref());
                            let new_rank = severity_rank(Some(sev));
                            if new_rank > current_rank {
                                summary.max_flag_severity = Some(sev.to_owned());
                            }
                        }
                    }
                }
                "learned" => {
                    summary.learned_count += 1;
                }
                "done" | "partial" | "stuck" => {
                    summary.last_closing_verb = Some(verb);
                }
                _ => {}
            }
        }

        for (task_id, sessions) in sessions_per_task {
            if let Some(summary) = map.get_mut(&task_id) {
                summary.session_count = sessions.len() as u32;
            }
        }

        Ok(map)
    }

    pub fn answer_ask(&self, signal_id: u32, answer: String) -> Result<(), String> {
        if answer.trim().is_empty() {
            return ralph_err!(codes::SIGNAL_OPS, "Answer cannot be empty");
        }

        let affected = self
            .conn
            .execute(
                "UPDATE task_comments SET signal_answered = ?1 WHERE id = ?2 AND signal_verb = 'ask'",
                rusqlite::params![answer, signal_id],
            )
            .ralph_err(codes::DB_WRITE, "Failed to answer ask signal")?;

        if affected == 0 {
            return ralph_err!(
                codes::SIGNAL_OPS,
                "Signal {signal_id} does not exist or is not an ask"
            );
        }

        Ok(())
    }
}

fn severity_rank(severity: Option<&str>) -> u8 {
    match severity {
        Some("info") => 1,
        Some("warning") => 2,
        Some("blocking") => 3,
        _ => 0,
    }
}
