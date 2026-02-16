use core_contracts::project::{
    ProjectInfo, ProjectInitializeArgs, ProjectScanArgs, ProjectValidatePathArgs, RalphProject,
    RecentProject,
};
use core_contracts::session::ProjectLockSetArgs;
use core_errors::{codes, ralph_err, RalphResult};
use service_project::project_scan;
use std::path::PathBuf;

use crate::rpc_codec::{decode_args, encode_result, require_null_payload};
use crate::state::AppState;

pub fn project_validate_path(payload: serde_json::Value) -> RalphResult<serde_json::Value> {
    let args: ProjectValidatePathArgs = decode_args("project_validate_path", payload)?;
    let path = PathBuf::from(args.path);
    service_project::project::validate_project_path(&path)?;
    Ok(serde_json::Value::Null)
}

pub fn project_initialize(payload: serde_json::Value) -> RalphResult<serde_json::Value> {
    let args: ProjectInitializeArgs = decode_args("project_initialize", payload)?;
    service_project::project::project_initialize(args)?;
    Ok(serde_json::Value::Null)
}

pub fn project_lock_set(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: ProjectLockSetArgs = decode_args("project_lock_set", payload)?;
    let data_dir = state.xdg.ensure_data()?;
    let _canonical = service_project::session::project_lock_set_and_record_recent(
        &state.locked_project,
        &state.db,
        data_dir,
        args,
    )?;
    Ok(serde_json::Value::Null)
}

pub fn project_lock_get(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    require_null_payload("project_lock_get", payload)?;
    let locked = service_project::session::project_lock_get(&state.locked_project)?;
    encode_result("project_lock_get", locked)
}

pub fn project_recent_list(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    require_null_payload("project_recent_list", payload)?;
    let data_dir = state.xdg.ensure_data()?;
    let projects: Vec<RecentProject> = project_scan::recents_load(data_dir)?;
    encode_result("project_recent_list", projects)
}

pub fn project_scan(payload: serde_json::Value) -> RalphResult<serde_json::Value> {
    let args: ProjectScanArgs = decode_args("project_scan", payload)?;
    let projects: Vec<RalphProject> = project_scan::project_scan(args)?;
    encode_result("project_scan", projects)
}

pub fn project_info_get(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    require_null_payload("project_info_get", payload)?;
    let info: ProjectInfo =
        service_project::session::with_db(&state.db, project_scan::project_info_get)?;
    encode_result("project_info_get", info)
}

pub fn system_home_dir_get(payload: serde_json::Value) -> RalphResult<serde_json::Value> {
    require_null_payload("system_home_dir_get", payload)?;
    let path = dirs::home_dir().ok_or_else(|| {
        core_errors::err_string(codes::FILESYSTEM, "Failed to get home directory")
    })?;
    encode_result("system_home_dir_get", path.to_string_lossy().to_string())
}

pub fn execution_start(payload: serde_json::Value) -> RalphResult<serde_json::Value> {
    require_null_payload("execution_start", payload)?;
    ralph_err!(codes::LOOP_ENGINE, "Not implemented")
}

pub fn execution_pause(payload: serde_json::Value) -> RalphResult<serde_json::Value> {
    require_null_payload("execution_pause", payload)?;
    ralph_err!(codes::LOOP_ENGINE, "Not implemented")
}

pub fn execution_resume(payload: serde_json::Value) -> RalphResult<serde_json::Value> {
    require_null_payload("execution_resume", payload)?;
    ralph_err!(codes::LOOP_ENGINE, "Not implemented")
}

pub fn execution_stop(payload: serde_json::Value) -> RalphResult<serde_json::Value> {
    require_null_payload("execution_stop", payload)?;
    ralph_err!(codes::LOOP_ENGINE, "Not implemented")
}

pub fn execution_state_get(payload: serde_json::Value) -> RalphResult<serde_json::Value> {
    require_null_payload("execution_state_get", payload)?;
    ralph_err!(codes::LOOP_ENGINE, "Not implemented")
}
