use super::remote_proxy::{remote_invoke_args, remote_invoke_no_args};
use super::state::{AppState, CommandContext};
use ralph_macros::ipc_type;
use serde::{Deserialize, Serialize};
use tauri::State;

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentSessionsByIdArgs {
    pub id: String,
}

#[tauri::command]
pub async fn agent_sessions_create_human(
    state: State<'_, AppState>,
    args: sqlite_db::AgentSessionCreateInput,
) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "agent_sessions_create_human", args).await;
    }

    CommandContext::from_tauri_state(&state).db(|db| db.create_human_agent_session(args))
}

#[tauri::command]
pub async fn agent_sessions_update_human(
    state: State<'_, AppState>,
    args: sqlite_db::AgentSessionUpdateInput,
) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "agent_sessions_update_human", args).await;
    }

    CommandContext::from_tauri_state(&state).db(|db| db.update_human_agent_session(args))
}

#[tauri::command]
pub async fn agent_sessions_delete_human(
    state: State<'_, AppState>,
    args: AgentSessionsByIdArgs,
) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "agent_sessions_delete_human", args).await;
    }

    CommandContext::from_tauri_state(&state).db(|db| db.delete_human_agent_session(&args.id))
}

#[tauri::command]
pub async fn agent_sessions_get(
    state: State<'_, AppState>,
    args: AgentSessionsByIdArgs,
) -> Result<Option<sqlite_db::AgentSession>, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "agent_sessions_get", args).await;
    }

    CommandContext::from_tauri_state(&state).db(|db| Ok(db.get_agent_session_by_id(&args.id)))
}

#[tauri::command]
pub async fn agent_sessions_list_human(
    state: State<'_, AppState>,
) -> Result<Vec<sqlite_db::AgentSession>, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "agent_sessions_list_human").await;
    }

    CommandContext::from_tauri_state(&state).db(|db| Ok(db.list_human_agent_sessions()))
}
