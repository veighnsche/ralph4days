use core_contracts::agent_sessions::AgentSessionsByIdArgs;
use core_contracts::domain::{AgentSessionCreateInput, AgentSessionUpdateInput};
use core_errors::RalphResult;
use service_tasks::agent_sessions_service;

use crate::rpc_codec::{decode_args, encode_result, require_null_payload};
use crate::state::AppState;

pub fn agent_sessions_create_human(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: AgentSessionCreateInput = decode_args("agent_sessions_create_human", payload)?;
    service_project::session::with_db(&state.db, |db| {
        agent_sessions_service::agent_sessions_create_human(db, args)
    })?;
    Ok(serde_json::Value::Null)
}

pub fn agent_sessions_update_human(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: AgentSessionUpdateInput = decode_args("agent_sessions_update_human", payload)?;
    service_project::session::with_db(&state.db, |db| {
        agent_sessions_service::agent_sessions_update_human(db, args)
    })?;
    Ok(serde_json::Value::Null)
}

pub fn agent_sessions_delete_human(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: AgentSessionsByIdArgs = decode_args("agent_sessions_delete_human", payload)?;
    service_project::session::with_db(&state.db, |db| {
        agent_sessions_service::agent_sessions_delete_human(db, &args.id)
    })?;
    Ok(serde_json::Value::Null)
}

pub fn agent_sessions_get(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: AgentSessionsByIdArgs = decode_args("agent_sessions_get", payload)?;
    let session = service_project::session::with_db(&state.db, |db| {
        agent_sessions_service::agent_sessions_get(db, &args.id)
    })?;
    encode_result("agent_sessions_get", session)
}

pub fn agent_sessions_list_human(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    require_null_payload("agent_sessions_list_human", payload)?;
    let sessions = service_project::session::with_db(
        &state.db,
        agent_sessions_service::agent_sessions_list_human,
    )?;
    encode_result("agent_sessions_list_human", sessions)
}
