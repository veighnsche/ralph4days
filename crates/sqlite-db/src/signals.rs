use crate::types::{TaskSignal, TaskSignalSummary};
use crate::SqliteDb;
use ralph_errors::{codes, ralph_err, RalphResultExt};
use std::collections::HashMap;

impl SqliteDb {
    pub fn get_task_signals(&self, task_id: u32) -> Result<Vec<TaskSignal>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, task_id, session_id, signal_verb, signal_payload, signal_answered, created \
                 FROM task_comments WHERE task_id = ?1 AND signal_verb IS NOT NULL ORDER BY created ASC, id ASC",
            )
            .ralph_err(codes::DB_READ, "Failed to prepare signal query")?;

        let rows = stmt
            .query_map([task_id], |row| {
                Ok(TaskSignal {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    session_id: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    verb: row.get(3)?,
                    payload: row.get(4)?,
                    answered: row.get(5)?,
                    created_at: row.get(6)?,
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
