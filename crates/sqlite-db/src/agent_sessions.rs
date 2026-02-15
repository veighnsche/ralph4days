use crate::types::{AgentSession, AgentSessionCreateInput, AgentSessionUpdateInput};
use crate::SqliteDb;
use ralph_errors::{codes, err_string, ralph_err, RalphResult, RalphResultExt};
use rusqlite::OptionalExtension;

impl SqliteDb {
    pub fn create_human_agent_session(&self, input: AgentSessionCreateInput) -> RalphResult<()> {
        if input.id.trim().is_empty() {
            return ralph_err!(codes::TASK_VALIDATION, "Session id cannot be empty");
        }
        if input.kind.trim().is_empty() {
            return ralph_err!(codes::TASK_VALIDATION, "Session kind cannot be empty");
        }

        if self.check_exists("agent_sessions", "id", &input.id)? {
            return ralph_err!(codes::TASK_OPS, "Session '{}' already exists", input.id);
        }

        let now = self.now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        self.conn
            .execute(
                "INSERT INTO agent_sessions (id, session_number, kind, started_by, task_id, agent, model, launch_command, \
                 post_start_preamble, init_prompt, started, status) \
                 VALUES (?1, (SELECT COALESCE(MAX(session_number), 0) + 1 FROM agent_sessions), ?2, 'human', ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'running')",
                rusqlite::params![
                    input.id,
                    input.kind,
                    input.task_id,
                    input.agent,
                    input.model,
                    input.launch_command,
                    input.post_start_preamble,
                    input.init_prompt,
                    now,
                ],
            )
            .ralph_err(codes::DB_WRITE, "Failed to create human agent session")?;

        Ok(())
    }

    pub fn update_human_agent_session(&self, input: AgentSessionUpdateInput) -> RalphResult<()> {
        if input.id.trim().is_empty() {
            return ralph_err!(codes::TASK_VALIDATION, "Session id cannot be empty");
        }

        let existing = self.get_agent_session_by_id(&input.id)?.ok_or_else(|| {
            err_string(
                codes::TASK_OPS,
                format!("Session '{}' does not exist", input.id),
            )
        })?;

        if existing.started_by != "human" {
            return ralph_err!(
                codes::TASK_OPS,
                "Session '{}' is not human-started",
                input.id
            );
        }

        let output_bytes_i64 = input.output_bytes.map(i64::from);

        self.conn
            .execute(
                "UPDATE agent_sessions SET \
                 kind = COALESCE(?1, kind), \
                 task_id = COALESCE(?2, task_id), \
                 agent = COALESCE(?3, agent), \
                 model = COALESCE(?4, model), \
                 launch_command = COALESCE(?5, launch_command), \
                 post_start_preamble = COALESCE(?6, post_start_preamble), \
                 init_prompt = COALESCE(?7, init_prompt), \
                 ended = COALESCE(?8, ended), \
                 exit_code = COALESCE(?9, exit_code), \
                 closing_verb = COALESCE(?10, closing_verb), \
                 status = COALESCE(?11, status), \
                 prompt_hash = COALESCE(?12, prompt_hash), \
                 output_bytes = COALESCE(?13, output_bytes), \
                 error_text = COALESCE(?14, error_text) \
                 WHERE id = ?15 AND started_by = 'human'",
                rusqlite::params![
                    input.kind,
                    input.task_id,
                    input.agent,
                    input.model,
                    input.launch_command,
                    input.post_start_preamble,
                    input.init_prompt,
                    input.ended,
                    input.exit_code,
                    input.closing_verb,
                    input.status,
                    input.prompt_hash,
                    output_bytes_i64,
                    input.error_text,
                    input.id,
                ],
            )
            .ralph_err(codes::DB_WRITE, "Failed to update human agent session")?;

        Ok(())
    }

    pub fn delete_human_agent_session(&self, id: &str) -> RalphResult<()> {
        if id.trim().is_empty() {
            return ralph_err!(codes::TASK_VALIDATION, "Session id cannot be empty");
        }

        let affected = self
            .conn
            .execute(
                "DELETE FROM agent_sessions WHERE id = ?1 AND started_by = 'human'",
                rusqlite::params![id],
            )
            .ralph_err(codes::DB_WRITE, "Failed to delete human agent session")?;

        if affected == 0 {
            return ralph_err!(
                codes::TASK_OPS,
                "Session '{}' does not exist or is not human-started",
                id
            );
        }

        Ok(())
    }

    pub fn get_agent_session_by_id(&self, id: &str) -> RalphResult<Option<AgentSession>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, kind, started_by, task_id, agent, model, launch_command, \
             post_start_preamble, init_prompt, started, ended, exit_code, closing_verb, \
             status, prompt_hash, output_bytes, error_text, session_number \
             FROM agent_sessions WHERE id = ?1",
            )
            .ralph_err(codes::DB_READ, "Failed to prepare agent session query")?;

        stmt.query_row([id], Self::row_to_agent_session)
            .optional()
            .ralph_err(codes::DB_READ, "Failed to query agent session by id")
    }

    pub fn list_human_agent_sessions(&self) -> RalphResult<Vec<AgentSession>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, kind, started_by, task_id, agent, model, launch_command, \
                 post_start_preamble, init_prompt, started, ended, exit_code, closing_verb, \
                 status, prompt_hash, output_bytes, error_text, session_number \
                 FROM agent_sessions WHERE started_by = 'human' ORDER BY session_number DESC",
            )
            .ralph_err(
                codes::DB_READ,
                "Failed to prepare human agent sessions list query",
            )?;

        let rows = stmt
            .query_map([], Self::row_to_agent_session)
            .ralph_err(codes::DB_READ, "Failed to query human agent sessions list")?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row.ralph_err(codes::DB_READ, "Failed to decode human agent session row")?);
        }
        Ok(out)
    }

    fn row_to_agent_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<AgentSession> {
        let output_bytes_i64: Option<i64> = row.get(15)?;
        let session_number_i64: i64 = row.get(17)?;

        let output_bytes = match output_bytes_i64 {
            Some(v) => Some(
                u32::try_from(v).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(15, v))?,
            ),
            None => None,
        };
        let session_number = u32::try_from(session_number_i64)
            .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(17, session_number_i64))?;

        Ok(AgentSession {
            id: row.get(0)?,
            session_number,
            kind: row.get(1)?,
            started_by: row.get(2)?,
            task_id: row.get(3)?,
            agent: row.get(4)?,
            model: row.get(5)?,
            launch_command: row.get(6)?,
            post_start_preamble: row.get(7)?,
            init_prompt: row.get(8)?,
            started: row.get(9)?,
            ended: row.get(10)?,
            exit_code: row.get(11)?,
            closing_verb: row.get(12)?,
            status: row.get(13)?,
            prompt_hash: row.get(14)?,
            output_bytes,
            error_text: row.get(16)?,
        })
    }
}
