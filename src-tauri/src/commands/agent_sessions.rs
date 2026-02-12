use super::state::{get_db, AppState};
use tauri::State;

#[tauri::command]
pub fn create_human_agent_session(
    state: State<'_, AppState>,
    params: sqlite_db::AgentSessionCreateInput,
) -> Result<(), String> {
    let db = get_db(&state)?;
    db.create_human_agent_session(params)
}

#[tauri::command]
pub fn update_human_agent_session(
    state: State<'_, AppState>,
    params: sqlite_db::AgentSessionUpdateInput,
) -> Result<(), String> {
    let db = get_db(&state)?;
    db.update_human_agent_session(params)
}

#[tauri::command]
pub fn delete_human_agent_session(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let db = get_db(&state)?;
    db.delete_human_agent_session(&id)
}

#[tauri::command]
pub fn get_agent_session(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<sqlite_db::AgentSession>, String> {
    let db = get_db(&state)?;
    Ok(db.get_agent_session_by_id(&id))
}

#[tauri::command]
pub fn list_human_agent_sessions(
    state: State<'_, AppState>,
) -> Result<Vec<sqlite_db::AgentSession>, String> {
    let db = get_db(&state)?;
    Ok(db.list_human_agent_sessions())
}
