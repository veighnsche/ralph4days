use ralph_backend::disciplines_contract::{
    DisciplinesCreateArgs, DisciplinesCroppedImageGetArgs, DisciplinesDeleteArgs,
    DisciplinesImageDataGetArgs, DisciplinesUpdateArgs,
};
use ralph_backend::disciplines_service;
use ralph_backend::subsystems_contract::{
    SubsystemsCommentAddArgs, SubsystemsCommentDeleteArgs, SubsystemsCommentUpdateArgs,
    SubsystemsCreateArgs, SubsystemsDeleteArgs, SubsystemsUpdateArgs,
};
use ralph_backend::subsystems_service;
use ralph_errors::RalphResult;

use crate::rpc_codec::{decode_args, encode_result, require_null_payload};
use crate::state::AppState;

pub fn subsystems_list(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    require_null_payload("subsystems_list", payload)?;
    let subsystems =
        ralph_backend::session::with_db(&state.db, subsystems_service::subsystems_list)?;
    encode_result("subsystems_list", subsystems)
}

pub fn subsystems_create(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: SubsystemsCreateArgs = decode_args("subsystems_create", payload)?;
    let created = ralph_backend::session::with_db(&state.db, |db| {
        subsystems_service::subsystems_create(db, args)
    })?;
    encode_result("subsystems_create", created)
}

pub fn subsystems_update(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: SubsystemsUpdateArgs = decode_args("subsystems_update", payload)?;
    let updated = ralph_backend::session::with_db(&state.db, |db| {
        subsystems_service::subsystems_update(db, args)
    })?;
    encode_result("subsystems_update", updated)
}

pub fn subsystems_delete(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: SubsystemsDeleteArgs = decode_args("subsystems_delete", payload)?;
    ralph_backend::session::with_db(&state.db, |db| {
        subsystems_service::subsystems_delete(db, args)
    })?;
    Ok(serde_json::Value::Null)
}

pub async fn subsystems_comment_add(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: SubsystemsCommentAddArgs = decode_args("subsystems_comment_add", payload)?;
    let project_path = ralph_backend::session::locked_project_path(&state.locked_project)?;

    let (subsystem, embed_work) = ralph_backend::session::with_db(&state.db, |db| {
        subsystems_service::subsystems_comment_add_prepare(db, args)
    })?;
    subsystems_service::subsystems_comment_apply_embedding(&project_path, embed_work).await?;

    encode_result("subsystems_comment_add", subsystem)
}

pub async fn subsystems_comment_update(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: SubsystemsCommentUpdateArgs = decode_args("subsystems_comment_update", payload)?;
    let project_path = ralph_backend::session::locked_project_path(&state.locked_project)?;

    let (subsystem, embed_work) = ralph_backend::session::with_db(&state.db, |db| {
        subsystems_service::subsystems_comment_update_prepare(db, args)
    })?;

    if let Some(work) = embed_work {
        subsystems_service::subsystems_comment_apply_embedding(&project_path, work).await?;
    }

    encode_result("subsystems_comment_update", subsystem)
}

pub fn subsystems_comment_delete(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: SubsystemsCommentDeleteArgs = decode_args("subsystems_comment_delete", payload)?;
    let updated = ralph_backend::session::with_db(&state.db, |db| {
        subsystems_service::subsystems_comment_delete(db, args)
    })?;
    encode_result("subsystems_comment_delete", updated)
}

pub fn disciplines_list(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    require_null_payload("disciplines_list", payload)?;
    let disciplines =
        ralph_backend::session::with_db(&state.db, disciplines_service::disciplines_list)?;
    encode_result("disciplines_list", disciplines)
}

pub fn disciplines_create(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: DisciplinesCreateArgs = decode_args("disciplines_create", payload)?;
    let created = ralph_backend::session::with_db(&state.db, |db| {
        disciplines_service::disciplines_create(db, args)
    })?;
    encode_result("disciplines_create", created)
}

pub fn disciplines_update(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: DisciplinesUpdateArgs = decode_args("disciplines_update", payload)?;
    let updated = ralph_backend::session::with_db(&state.db, |db| {
        disciplines_service::disciplines_update(db, args)
    })?;
    encode_result("disciplines_update", updated)
}

pub fn disciplines_delete(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: DisciplinesDeleteArgs = decode_args("disciplines_delete", payload)?;
    let deleted = ralph_backend::session::with_db(&state.db, |db| {
        disciplines_service::disciplines_delete(db, args)
    })?;
    encode_result("disciplines_delete", deleted)
}

pub fn disciplines_image_data_get(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: DisciplinesImageDataGetArgs = decode_args("disciplines_image_data_get", payload)?;
    let project_path = ralph_backend::session::locked_project_path(&state.locked_project)?;
    let result = ralph_backend::session::with_db(&state.db, |db| {
        disciplines_service::disciplines_image_data_get(&project_path, db, args)
    })?;
    encode_result("disciplines_image_data_get", result)
}

pub fn disciplines_cropped_image_get(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: DisciplinesCroppedImageGetArgs =
        decode_args("disciplines_cropped_image_get", payload)?;
    let project_path = ralph_backend::session::locked_project_path(&state.locked_project)?;
    let result = ralph_backend::session::with_db(&state.db, |db| {
        disciplines_service::disciplines_cropped_image_get(&project_path, db, args)
    })?;
    encode_result("disciplines_cropped_image_get", result)
}
