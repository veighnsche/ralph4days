use ralph_contracts::domain::{AgentSession, AgentSessionCreateInput, AgentSessionUpdateInput};
use ralph_errors::RalphResult;
use sqlite_db::SqliteDb;

pub fn agent_sessions_create_human(
    db: &SqliteDb,
    args: AgentSessionCreateInput,
) -> RalphResult<()> {
    db.create_human_agent_session(args)
}

pub fn agent_sessions_update_human(
    db: &SqliteDb,
    args: AgentSessionUpdateInput,
) -> RalphResult<()> {
    db.update_human_agent_session(args)
}

pub fn agent_sessions_delete_human(db: &SqliteDb, id: &str) -> RalphResult<()> {
    db.delete_human_agent_session(id)
}

pub fn agent_sessions_get(db: &SqliteDb, id: &str) -> RalphResult<Option<AgentSession>> {
    db.get_agent_session_by_id(id)
}

pub fn agent_sessions_list_human(db: &SqliteDb) -> RalphResult<Vec<AgentSession>> {
    db.list_human_agent_sessions()
}
