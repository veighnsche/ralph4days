use service_subsystems::disciplines_service;
use service_subsystems::subsystems_service;
use core_contracts::disciplines::{
    DisciplinesCreateArgs, DisciplinesCroppedImageGetArgs, DisciplinesDeleteArgs,
    DisciplinesImageDataGetArgs, DisciplinesUpdateArgs,
};
use core_contracts::subsystems::{
    SubsystemsCommentAddArgs, SubsystemsCommentDeleteArgs, SubsystemsCommentUpdateArgs,
    SubsystemsCreateArgs, SubsystemsDeleteArgs, SubsystemsUpdateArgs,
};
use core_errors::RalphResult;

use crate::rpc_codec::{decode_args, encode_result, require_null_payload};
use crate::state::AppState;

#[derive(Debug, Clone, serde::Serialize)]
struct VisualIdentityData {
    style: String,
    theme: String,
    tone: String,
    references: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct StackMetadataData {
    stack_id: u8,
    name: String,
    description: String,
    philosophy: String,
    visual_identity: VisualIdentityData,
    when_to_use: Vec<String>,
    discipline_count: u8,
    characteristics: Vec<String>,
}

pub fn subsystems_list(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    require_null_payload("subsystems_list", payload)?;
    let subsystems =
        service_project::session::with_db(&state.db, subsystems_service::subsystems_list)?;
    encode_result("subsystems_list", subsystems)
}

pub fn subsystems_create(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: SubsystemsCreateArgs = decode_args("subsystems_create", payload)?;
    let created = service_project::session::with_db(&state.db, |db| {
        subsystems_service::subsystems_create(db, args)
    })?;
    encode_result("subsystems_create", created)
}

pub fn subsystems_update(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: SubsystemsUpdateArgs = decode_args("subsystems_update", payload)?;
    let updated = service_project::session::with_db(&state.db, |db| {
        subsystems_service::subsystems_update(db, args)
    })?;
    encode_result("subsystems_update", updated)
}

pub fn subsystems_delete(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: SubsystemsDeleteArgs = decode_args("subsystems_delete", payload)?;
    service_project::session::with_db(&state.db, |db| {
        subsystems_service::subsystems_delete(db, args)
    })?;
    Ok(serde_json::Value::Null)
}

pub async fn subsystems_comment_add(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: SubsystemsCommentAddArgs = decode_args("subsystems_comment_add", payload)?;
    let project_path = service_project::session::locked_project_path(&state.locked_project)?;

    let (subsystem, embed_work) = service_project::session::with_db(&state.db, |db| {
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
    let project_path = service_project::session::locked_project_path(&state.locked_project)?;

    let (subsystem, embed_work) = service_project::session::with_db(&state.db, |db| {
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
    let updated = service_project::session::with_db(&state.db, |db| {
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
        service_project::session::with_db(&state.db, disciplines_service::disciplines_list)?;
    encode_result("disciplines_list", disciplines)
}

pub fn disciplines_create(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: DisciplinesCreateArgs = decode_args("disciplines_create", payload)?;
    let created = service_project::session::with_db(&state.db, |db| {
        disciplines_service::disciplines_create(db, args)
    })?;
    encode_result("disciplines_create", created)
}

pub fn disciplines_update(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: DisciplinesUpdateArgs = decode_args("disciplines_update", payload)?;
    let updated = service_project::session::with_db(&state.db, |db| {
        disciplines_service::disciplines_update(db, args)
    })?;
    encode_result("disciplines_update", updated)
}

pub fn disciplines_delete(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: DisciplinesDeleteArgs = decode_args("disciplines_delete", payload)?;
    let deleted = service_project::session::with_db(&state.db, |db| {
        disciplines_service::disciplines_delete(db, args)
    })?;
    encode_result("disciplines_delete", deleted)
}

pub fn stacks_metadata_list(payload: serde_json::Value) -> RalphResult<serde_json::Value> {
    require_null_payload("stacks_metadata_list", payload)?;
    let metadata: Vec<StackMetadataData> = catalog_disciplines::get_all_stack_metadata()
        .iter()
        .map(|m| StackMetadataData {
            stack_id: m.stack_id,
            name: m.name.clone(),
            description: m.description.clone(),
            philosophy: m.philosophy.clone(),
            visual_identity: VisualIdentityData {
                style: m.visual_identity.style.clone(),
                theme: m.visual_identity.theme.clone(),
                tone: m.visual_identity.tone.clone(),
                references: m.visual_identity.references.clone(),
            },
            when_to_use: m.when_to_use.clone(),
            discipline_count: m.discipline_count,
            characteristics: m.characteristics.clone(),
        })
        .collect();
    encode_result("stacks_metadata_list", metadata)
}

pub fn disciplines_image_data_get(
    state: &AppState,
    payload: serde_json::Value,
) -> RalphResult<serde_json::Value> {
    let args: DisciplinesImageDataGetArgs = decode_args("disciplines_image_data_get", payload)?;
    let project_path = service_project::session::locked_project_path(&state.locked_project)?;
    let result = service_project::session::with_db(&state.db, |db| {
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
    let project_path = service_project::session::locked_project_path(&state.locked_project)?;
    let result = service_project::session::with_db(&state.db, |db| {
        disciplines_service::disciplines_cropped_image_get(&project_path, db, args)
    })?;
    encode_result("disciplines_cropped_image_get", result)
}
