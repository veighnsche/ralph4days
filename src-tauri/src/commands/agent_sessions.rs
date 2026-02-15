use super::remote_proxy::{remote_invoke_args, remote_invoke_no_args};
use super::state::{with_db, AppState};
use ralph_backend::agent_sessions_contract::AgentSessionsByIdArgs;
use ralph_backend::agent_sessions_service;
use tauri::State;

#[tauri::command]
pub async fn agent_sessions_create_human(
    state: State<'_, AppState>,
    args: sqlite_db::AgentSessionCreateInput,
) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "agent_sessions_create_human", args).await;
    }

    with_db(&state, |db| {
        agent_sessions_service::agent_sessions_create_human(db, args)
    })
}

#[tauri::command]
pub async fn agent_sessions_update_human(
    state: State<'_, AppState>,
    args: sqlite_db::AgentSessionUpdateInput,
) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "agent_sessions_update_human", args).await;
    }

    with_db(&state, |db| {
        agent_sessions_service::agent_sessions_update_human(db, args)
    })
}

#[tauri::command]
pub async fn agent_sessions_delete_human(
    state: State<'_, AppState>,
    args: AgentSessionsByIdArgs,
) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "agent_sessions_delete_human", args).await;
    }

    with_db(&state, |db| {
        agent_sessions_service::agent_sessions_delete_human(db, &args.id)
    })
}

#[tauri::command]
pub async fn agent_sessions_get(
    state: State<'_, AppState>,
    args: AgentSessionsByIdArgs,
) -> Result<Option<sqlite_db::AgentSession>, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "agent_sessions_get", args).await;
    }

    with_db(&state, |db| {
        agent_sessions_service::agent_sessions_get(db, &args.id)
    })
}

#[tauri::command]
pub async fn agent_sessions_list_human(
    state: State<'_, AppState>,
) -> Result<Vec<sqlite_db::AgentSession>, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "agent_sessions_list_human").await;
    }

    with_db(&state, agent_sessions_service::agent_sessions_list_human)
}
