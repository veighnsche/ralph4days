use super::remote_proxy::{remote_invoke_args, remote_invoke_no_args};
use super::state::{with_db, AppState};
use ralph_backend::agent_sessions_contract::AgentSessionsByIdArgs;
use ralph_backend::agent_sessions_service;
use ralph_errors::RalphResult;
use tauri::State;

#[tauri::command]
pub async fn agent_sessions_create_human(
    state: State<'_, AppState>,
    args: sqlite_db::AgentSessionCreateInput,
) -> RalphResult<()> {
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
) -> RalphResult<()> {
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
) -> RalphResult<()> {
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
) -> RalphResult<Option<sqlite_db::AgentSession>> {
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
) -> RalphResult<Vec<sqlite_db::AgentSession>> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "agent_sessions_list_human").await;
    }

    with_db(&state, agent_sessions_service::agent_sessions_list_human)
}
