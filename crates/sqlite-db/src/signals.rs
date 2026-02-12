use crate::types::{TaskSignal, TaskSignalSummary};
use crate::SqliteDb;
use ralph_errors::{codes, ralph_err, RalphResultExt};
use std::collections::HashMap;

pub struct DoneSignalInput {
    pub task_id: u32,
    pub session_id: String,
    pub summary: String,
}

pub struct PartialSignalInput {
    pub task_id: u32,
    pub session_id: String,
    pub summary: String,
    pub remaining: String,
}

pub struct StuckSignalInput {
    pub task_id: u32,
    pub session_id: String,
    pub reason: String,
}

pub struct AskSignalInput {
    pub task_id: u32,
    pub session_id: String,
    pub question: String,
    pub blocking: bool,
    pub options: Option<Vec<String>>,
    pub preferred: Option<String>,
}

pub struct FlagSignalInput {
    pub task_id: u32,
    pub session_id: String,
    pub what: String,
    pub severity: String,
    pub category: String,
}

pub struct LearnedSignalInput {
    pub task_id: u32,
    pub session_id: String,
    pub text: String,
    pub kind: String,
    pub scope: String,
    pub rationale: Option<String>,
}

pub struct SuggestSignalInput {
    pub task_id: u32,
    pub session_id: String,
    pub what: String,
    pub kind: String,
    pub why: String,
}

pub struct BlockedSignalInput {
    pub task_id: u32,
    pub session_id: String,
    pub on: String,
    pub kind: String,
    pub detail: Option<String>,
}

impl SqliteDb {
    fn resolve_discipline_id(&self, discipline_name: Option<&str>, task_id: u32) -> Option<u32> {
        let _ = task_id;
        discipline_name.and_then(|name| {
            self.conn
                .query_row(
                    "SELECT id FROM disciplines WHERE name = ?1",
                    [name],
                    |row| row.get(0),
                )
                .ok()
        })
    }

    pub fn add_signal(
        &self,
        task_id: u32,
        discipline: Option<String>,
        _agent_task_id: Option<u32>,
        priority: Option<String>,
        body: String,
    ) -> Result<(), String> {
        self.add_signal_with_parent(task_id, discipline, priority, body, None)
    }

    pub fn add_signal_with_parent(
        &self,
        task_id: u32,
        discipline: Option<String>,
        _priority: Option<String>,
        body: String,
        parent_signal_id: Option<u32>,
    ) -> Result<(), String> {
        if body.trim().is_empty() {
            return ralph_err!(codes::SIGNAL_OPS, "Signal body cannot be empty");
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
            return ralph_err!(codes::SIGNAL_OPS, "Task {task_id} does not exist");
        }

        if let Some(parent_id) = parent_signal_id {
            let parent_exists: bool = self
                .conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM task_signals WHERE id = ?1 AND task_id = ?2",
                    rusqlite::params![parent_id, task_id],
                    |row| row.get(0),
                )
                .ralph_err(codes::DB_READ, "Failed to check parent signal")?;
            if !parent_exists {
                return ralph_err!(
                    codes::SIGNAL_OPS,
                    "Parent signal {parent_id} does not exist"
                );
            }

            let parent_has_parent: bool = self
                .conn
                .query_row(
                    "SELECT parent_signal_id IS NOT NULL FROM task_signals WHERE id = ?1",
                    [parent_id],
                    |row| row.get(0),
                )
                .ralph_err(codes::DB_READ, "Failed to check parent nesting")?;
            if parent_has_parent {
                return ralph_err!(codes::SIGNAL_OPS, "Cannot reply to a reply (max 2 layers)");
            }
        }

        let now = self.now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let discipline_id: Option<i64> =
            discipline.and_then(|disc_name| self.get_id_from_name("disciplines", &disc_name).ok());

        self.conn
            .execute(
                "INSERT INTO task_signals (task_id, discipline_id, verb, text, created) \
                 VALUES (?1, ?2, 'signal', ?3, ?4)",
                rusqlite::params![task_id, discipline_id, body, now],
            )
            .ralph_err(codes::DB_WRITE, "Failed to insert signal")?;

        Ok(())
    }

    pub fn update_signal(&self, task_id: u32, signal_id: u32, body: String) -> Result<(), String> {
        if body.trim().is_empty() {
            return ralph_err!(codes::SIGNAL_OPS, "Signal body cannot be empty");
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
            return ralph_err!(codes::SIGNAL_OPS, "Task {task_id} does not exist");
        }

        let affected = self
            .conn
            .execute(
                "UPDATE task_signals SET text = ?1 WHERE id = ?2 AND task_id = ?3",
                rusqlite::params![body, signal_id, task_id],
            )
            .ralph_err(codes::DB_WRITE, "Failed to update signal")?;

        if affected == 0 {
            return ralph_err!(
                codes::SIGNAL_OPS,
                "Signal {signal_id} does not exist on task {task_id}"
            );
        }

        Ok(())
    }

    pub fn delete_signal(&self, task_id: u32, signal_id: u32) -> Result<(), String> {
        let task_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM tasks WHERE id = ?1",
                [task_id],
                |row| row.get(0),
            )
            .ralph_err(codes::DB_READ, "Failed to check task")?;
        if !task_exists {
            return ralph_err!(codes::SIGNAL_OPS, "Task {task_id} does not exist");
        }

        let affected = self
            .conn
            .execute(
                "DELETE FROM task_signals WHERE id = ?1 AND task_id = ?2",
                rusqlite::params![signal_id, task_id],
            )
            .ralph_err(codes::DB_WRITE, "Failed to delete signal")?;

        if affected == 0 {
            return ralph_err!(
                codes::SIGNAL_OPS,
                "Signal {signal_id} does not exist on task {task_id}"
            );
        }

        Ok(())
    }

    pub(crate) fn get_signals_for_task(&self, task_id: u32) -> Vec<TaskSignal> {
        let Ok(mut stmt) = self.conn.prepare(
            "SELECT tc.id, COALESCE(d.display_name, 'human') as author, \
             COALESCE(tc.text, tc.summary, tc.reason, tc.question, tc.what, tc.remaining, '') as body, \
             tc.created, tc.session_id, tc.verb, \
             tc.summary, tc.remaining, tc.reason, tc.question, tc.what, tc.\"on\", \
             tc.blocking, tc.severity, tc.category, tc.kind, tc.scope, \
             tc.preferred, tc.options, tc.rationale, tc.why, tc.detail, tc.text, tc.answer \
             FROM task_signals tc \
             LEFT JOIN disciplines d ON tc.discipline_id = d.id \
             WHERE tc.task_id = ?1 \
             ORDER BY tc.id DESC",
        ) else {
            return vec![];
        };

        stmt.query_map([task_id], |row| {
            let blocking_int: Option<i32> = row.get(12)?;
            let options_str: Option<String> = row.get(18)?;

            Ok(TaskSignal {
                id: row.get(0)?,
                author: row.get(1)?,
                body: row.get(2)?,
                created: row.get(3)?,
                session_id: row.get(4)?,
                signal_verb: row.get(5)?,
                parent_signal_id: None,
                priority: None,
                summary: row.get(6)?,
                remaining: row.get(7)?,
                reason: row.get(8)?,
                question: row.get(9)?,
                what: row.get(10)?,
                on: row.get(11)?,
                blocking: blocking_int.map(|b| b != 0),
                severity: row.get(13)?,
                category: row.get(14)?,
                kind: row.get(15)?,
                scope: row.get(16)?,
                preferred: row.get(17)?,
                options: options_str.map(|s| s.lines().map(str::to_owned).collect()),
                rationale: row.get(19)?,
                why: row.get(20)?,
                detail: row.get(21)?,
                text: row.get(22)?,
                answer: row.get(23)?,
            })
        })
        .map_or_else(
            |_| vec![],
            |rows| rows.filter_map(std::result::Result::ok).collect(),
        )
    }

    pub(crate) fn get_all_signals_by_task(&self) -> HashMap<u32, Vec<TaskSignal>> {
        let Ok(mut stmt) = self
            .conn
            .prepare("SELECT DISTINCT task_id FROM task_signals")
        else {
            return HashMap::new();
        };

        let Ok(task_id_rows) = stmt.query_map([], |row| row.get::<_, u32>(0)) else {
            return HashMap::new();
        };

        let task_ids: Vec<u32> = task_id_rows.filter_map(std::result::Result::ok).collect();

        let mut map = HashMap::new();
        for task_id in task_ids {
            let signals = self.get_signals_for_task(task_id);
            if !signals.is_empty() {
                map.insert(task_id, signals);
            }
        }
        map
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
            "SELECT task_id, verb, blocking, severity, answer, session_id \
             FROM task_signals WHERE task_id IN ({}) AND verb != 'signal' ORDER BY task_id, created ASC",
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
                    row.get::<_, Option<i32>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, String>(5)?,
                ))
            })
            .ralph_err(codes::DB_READ, "Failed to query signal summaries")?;

        let mut map: HashMap<u32, TaskSignalSummary> = HashMap::new();
        let mut sessions_per_task: HashMap<u32, std::collections::HashSet<String>> = HashMap::new();

        for row in rows.flatten() {
            let (task_id, verb, blocking, severity, answered, session_id) = row;
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
                    if answered.is_none() && blocking.is_some_and(|b| b != 0) {
                        summary.pending_asks += 1;
                    }
                }
                "flag" => {
                    summary.flag_count += 1;
                    if let Some(sev) = severity {
                        let current_rank = severity_rank(summary.max_flag_severity.as_deref());
                        let new_rank = severity_rank(Some(&sev));
                        if new_rank > current_rank {
                            summary.max_flag_severity = Some(sev);
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
                "UPDATE task_signals SET answer = ?1 WHERE id = ?2 AND verb = 'ask'",
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

    pub fn insert_done_signal(
        &self,
        discipline_name: Option<&str>,
        input: DoneSignalInput,
    ) -> Result<(), String> {
        if input.summary.trim().is_empty() {
            return ralph_err!(codes::SIGNAL_OPS, "Summary cannot be empty");
        }

        let discipline_id = self.resolve_discipline_id(discipline_name, input.task_id);
        let now = self.now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        self.conn
            .execute(
                "INSERT INTO task_signals (task_id, session_id, discipline_id, verb, summary, created) \
                 VALUES (?1, ?2, ?3, 'done', ?4, ?5)",
                rusqlite::params![
                    input.task_id,
                    input.session_id,
                    discipline_id,
                    input.summary.trim(),
                    now
                ],
            )
            .ralph_err(codes::DB_WRITE, "Failed to insert done signal")?;

        Ok(())
    }

    pub fn insert_partial_signal(
        &self,
        discipline_name: Option<&str>,
        input: PartialSignalInput,
    ) -> Result<(), String> {
        if input.summary.trim().is_empty() {
            return ralph_err!(codes::SIGNAL_OPS, "Summary cannot be empty");
        }
        if input.remaining.trim().is_empty() {
            return ralph_err!(codes::SIGNAL_OPS, "Remaining cannot be empty");
        }

        let discipline_id = self.resolve_discipline_id(discipline_name, input.task_id);
        let now = self.now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        self.conn
            .execute(
                "INSERT INTO task_signals (task_id, session_id, discipline_id, verb, summary, remaining, created) \
                 VALUES (?1, ?2, ?3, 'partial', ?4, ?5, ?6)",
                rusqlite::params![
                    input.task_id,
                    input.session_id,
                    discipline_id,
                    input.summary.trim(),
                    input.remaining.trim(),
                    now
                ],
            )
            .ralph_err(codes::DB_WRITE, "Failed to insert partial signal")?;

        Ok(())
    }

    pub fn insert_stuck_signal(
        &self,
        discipline_name: Option<&str>,
        input: StuckSignalInput,
    ) -> Result<(), String> {
        if input.reason.trim().is_empty() {
            return ralph_err!(codes::SIGNAL_OPS, "Reason cannot be empty");
        }

        let discipline_id = self.resolve_discipline_id(discipline_name, input.task_id);
        let now = self.now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        self.conn
            .execute(
                "INSERT INTO task_signals (task_id, session_id, discipline_id, verb, reason, created) \
                 VALUES (?1, ?2, ?3, 'stuck', ?4, ?5)",
                rusqlite::params![
                    input.task_id,
                    input.session_id,
                    discipline_id,
                    input.reason.trim(),
                    now
                ],
            )
            .ralph_err(codes::DB_WRITE, "Failed to insert stuck signal")?;

        Ok(())
    }

    pub fn insert_ask_signal(
        &self,
        discipline_name: Option<&str>,
        input: AskSignalInput,
    ) -> Result<(), String> {
        if input.question.trim().is_empty() {
            return ralph_err!(codes::SIGNAL_OPS, "Question cannot be empty");
        }

        let discipline_id = self.resolve_discipline_id(discipline_name, input.task_id);
        let now = self.now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let options = input.options.map(|opts| opts.join("\n"));

        self.conn
            .execute(
                "INSERT INTO task_signals (task_id, session_id, discipline_id, verb, question, options, preferred, blocking, created) \
                 VALUES (?1, ?2, ?3, 'ask', ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    input.task_id,
                    input.session_id,
                    discipline_id,
                    input.question.trim(),
                    options,
                    input.preferred,
                    i32::from(input.blocking),
                    now
                ],
            )
            .ralph_err(codes::DB_WRITE, "Failed to insert ask signal")?;

        Ok(())
    }

    pub fn insert_flag_signal(
        &self,
        discipline_name: Option<&str>,
        input: FlagSignalInput,
    ) -> Result<(), String> {
        if input.what.trim().is_empty() {
            return ralph_err!(codes::SIGNAL_OPS, "What cannot be empty");
        }

        let discipline_id = self.resolve_discipline_id(discipline_name, input.task_id);
        let now = self.now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        self.conn
            .execute(
                "INSERT INTO task_signals (task_id, session_id, discipline_id, verb, what, severity, category, created) \
                 VALUES (?1, ?2, ?3, 'flag', ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    input.task_id,
                    input.session_id,
                    discipline_id,
                    input.what.trim(),
                    input.severity,
                    input.category,
                    now
                ],
            )
            .ralph_err(codes::DB_WRITE, "Failed to insert flag signal")?;

        Ok(())
    }

    pub fn insert_learned_signal(
        &self,
        discipline_name: Option<&str>,
        input: LearnedSignalInput,
    ) -> Result<(), String> {
        if input.text.trim().is_empty() {
            return ralph_err!(codes::SIGNAL_OPS, "Text cannot be empty");
        }

        let discipline_id = self.resolve_discipline_id(discipline_name, input.task_id);
        let now = self.now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        self.conn
            .execute(
                "INSERT INTO task_signals (task_id, session_id, discipline_id, verb, text, kind, scope, rationale, created) \
                 VALUES (?1, ?2, ?3, 'learned', ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    input.task_id,
                    input.session_id,
                    discipline_id,
                    input.text.trim(),
                    input.kind,
                    input.scope,
                    input.rationale,
                    now
                ],
            )
            .ralph_err(codes::DB_WRITE, "Failed to insert learned signal")?;

        Ok(())
    }

    pub fn insert_suggest_signal(
        &self,
        discipline_name: Option<&str>,
        input: SuggestSignalInput,
    ) -> Result<(), String> {
        if input.what.trim().is_empty() {
            return ralph_err!(codes::SIGNAL_OPS, "What cannot be empty");
        }
        if input.why.trim().is_empty() {
            return ralph_err!(codes::SIGNAL_OPS, "Why cannot be empty");
        }

        let discipline_id = self.resolve_discipline_id(discipline_name, input.task_id);
        let now = self.now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        self.conn
            .execute(
                "INSERT INTO task_signals (task_id, session_id, discipline_id, verb, what, kind, why, created) \
                 VALUES (?1, ?2, ?3, 'suggest', ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    input.task_id,
                    input.session_id,
                    discipline_id,
                    input.what.trim(),
                    input.kind,
                    input.why.trim(),
                    now
                ],
            )
            .ralph_err(codes::DB_WRITE, "Failed to insert suggest signal")?;

        Ok(())
    }

    pub fn insert_blocked_signal(
        &self,
        discipline_name: Option<&str>,
        input: BlockedSignalInput,
    ) -> Result<(), String> {
        if input.on.trim().is_empty() {
            return ralph_err!(codes::SIGNAL_OPS, "On cannot be empty");
        }

        let discipline_id = self.resolve_discipline_id(discipline_name, input.task_id);
        let now = self.now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        self.conn
            .execute(
                "INSERT INTO task_signals (task_id, session_id, discipline_id, verb, \"on\", kind, detail, created) \
                 VALUES (?1, ?2, ?3, 'blocked', ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    input.task_id,
                    input.session_id,
                    discipline_id,
                    input.on.trim(),
                    input.kind,
                    input.detail,
                    now
                ],
            )
            .ralph_err(codes::DB_WRITE, "Failed to insert blocked signal")?;

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
