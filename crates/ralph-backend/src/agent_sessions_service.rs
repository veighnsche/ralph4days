use sqlite_db::{AgentSession, AgentSessionCreateInput, AgentSessionUpdateInput, SqliteDb};

pub fn agent_sessions_create_human(
    db: &SqliteDb,
    args: AgentSessionCreateInput,
) -> Result<(), String> {
    db.create_human_agent_session(args)
}

pub fn agent_sessions_update_human(
    db: &SqliteDb,
    args: AgentSessionUpdateInput,
) -> Result<(), String> {
    db.update_human_agent_session(args)
}

pub fn agent_sessions_delete_human(db: &SqliteDb, id: &str) -> Result<(), String> {
    db.delete_human_agent_session(id)
}

pub fn agent_sessions_get(db: &SqliteDb, id: &str) -> Result<Option<AgentSession>, String> {
    Ok(db.get_agent_session_by_id(id))
}

pub fn agent_sessions_list_human(db: &SqliteDb) -> Result<Vec<AgentSession>, String> {
    Ok(db.list_human_agent_sessions())
}
