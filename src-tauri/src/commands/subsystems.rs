use super::remote_proxy::{remote_invoke_args, remote_invoke_no_args};
use super::state::{AppState, CommandContext};
use ralph_backend::disciplines_contract::{
    DisciplineConfig, DisciplinesCreateArgs, DisciplinesCroppedImageGetArgs, DisciplinesDeleteArgs,
    DisciplinesImageDataGetArgs, DisciplinesUpdateArgs,
};
use ralph_backend::disciplines_service;
use ralph_backend::subsystems_contract::{
    SubsystemData, SubsystemsCommentAddArgs, SubsystemsCommentDeleteArgs,
    SubsystemsCommentUpdateArgs, SubsystemsCreateArgs, SubsystemsDeleteArgs, SubsystemsUpdateArgs,
};
use ralph_backend::subsystems_service;
use ralph_macros::ipc_type;
use tauri::State;

#[tauri::command]
pub async fn disciplines_list(state: State<'_, AppState>) -> Result<Vec<DisciplineConfig>, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "disciplines_list").await;
    }

    CommandContext::from_tauri_state(&state).db(disciplines_service::disciplines_list)
}

#[tauri::command]
pub async fn subsystems_list(state: State<'_, AppState>) -> Result<Vec<SubsystemData>, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_no_args(&rpc, "subsystems_list").await;
    }

    CommandContext::from_tauri_state(&state).db(subsystems_service::subsystems_list)
}

#[tauri::command]
pub async fn subsystems_create(
    state: State<'_, AppState>,
    args: SubsystemsCreateArgs,
) -> Result<SubsystemData, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "subsystems_create", args).await;
    }

    CommandContext::from_tauri_state(&state)
        .db(|db| subsystems_service::subsystems_create(db, args))
}

#[tauri::command]
pub async fn subsystems_update(
    state: State<'_, AppState>,
    args: SubsystemsUpdateArgs,
) -> Result<SubsystemData, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "subsystems_update", args).await;
    }

    CommandContext::from_tauri_state(&state)
        .db(|db| subsystems_service::subsystems_update(db, args))
}

#[tauri::command]
pub async fn subsystems_comment_add(
    state: State<'_, AppState>,
    args: SubsystemsCommentAddArgs,
) -> Result<SubsystemData, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "subsystems_comment_add", args).await;
    }

    let ctx = CommandContext::from_tauri_state(&state);
    let project_path = ctx.locked_project_path()?;

    let (subsystem, embed_work) =
        ctx.db(|db| subsystems_service::subsystems_comment_add_prepare(db, args))?;
    subsystems_service::subsystems_comment_apply_embedding(&project_path, embed_work).await?;

    Ok(subsystem)
}

#[tauri::command]
pub async fn subsystems_comment_update(
    state: State<'_, AppState>,
    args: SubsystemsCommentUpdateArgs,
) -> Result<SubsystemData, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "subsystems_comment_update", args).await;
    }

    let ctx = CommandContext::from_tauri_state(&state);
    let project_path = ctx.locked_project_path()?;

    let (subsystem, embed_work) =
        ctx.db(|db| subsystems_service::subsystems_comment_update_prepare(db, args))?;

    if let Some(work) = embed_work {
        subsystems_service::subsystems_comment_apply_embedding(&project_path, work).await?;
    }

    Ok(subsystem)
}

#[tauri::command]
pub async fn subsystems_comment_delete(
    state: State<'_, AppState>,
    args: SubsystemsCommentDeleteArgs,
) -> Result<SubsystemData, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "subsystems_comment_delete", args).await;
    }

    CommandContext::from_tauri_state(&state)
        .db(|db| subsystems_service::subsystems_comment_delete(db, args))
}

#[tauri::command]
pub async fn disciplines_create(
    state: State<'_, AppState>,
    args: DisciplinesCreateArgs,
) -> Result<DisciplineConfig, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "disciplines_create", args).await;
    }

    CommandContext::from_tauri_state(&state)
        .db(|db| disciplines_service::disciplines_create(db, args))
}

#[tauri::command]
pub async fn disciplines_update(
    state: State<'_, AppState>,
    args: DisciplinesUpdateArgs,
) -> Result<DisciplineConfig, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "disciplines_update", args).await;
    }

    CommandContext::from_tauri_state(&state)
        .db(|db| disciplines_service::disciplines_update(db, args))
}

#[tauri::command]
pub async fn subsystems_delete(
    state: State<'_, AppState>,
    args: SubsystemsDeleteArgs,
) -> Result<(), String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "subsystems_delete", args).await;
    }

    CommandContext::from_tauri_state(&state)
        .db(|db| subsystems_service::subsystems_delete(db, args))
}

#[tauri::command]
pub async fn disciplines_delete(
    state: State<'_, AppState>,
    args: DisciplinesDeleteArgs,
) -> Result<String, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "disciplines_delete", args).await;
    }

    CommandContext::from_tauri_state(&state)
        .db(|db| disciplines_service::disciplines_delete(db, args))
}

#[ipc_type]
pub struct VisualIdentityData {
    pub style: String,
    pub theme: String,
    pub tone: String,
    pub references: String,
}

#[ipc_type]
pub struct StackMetadataData {
    pub stack_id: u8,
    pub name: String,
    pub description: String,
    pub philosophy: String,
    pub visual_identity: VisualIdentityData,
    pub when_to_use: Vec<String>,
    pub discipline_count: u8,
    pub characteristics: Vec<String>,
}

#[tauri::command]
pub fn stacks_metadata_list() -> Vec<StackMetadataData> {
    predefined_disciplines::get_all_stack_metadata()
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
        .collect()
}

#[tauri::command]
pub async fn disciplines_image_data_get(
    state: State<'_, AppState>,
    args: DisciplinesImageDataGetArgs,
) -> Result<Option<String>, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "disciplines_image_data_get", args).await;
    }

    let ctx = CommandContext::from_tauri_state(&state);
    let project_path = ctx.locked_project_path()?;
    ctx.db(|db| disciplines_service::disciplines_image_data_get(&project_path, db, args))
}

#[tauri::command]
pub async fn disciplines_cropped_image_get(
    state: State<'_, AppState>,
    args: DisciplinesCroppedImageGetArgs,
) -> Result<Option<String>, String> {
    if let Some(rpc) = state.inner().remote_rpc_client().await? {
        return remote_invoke_args(&rpc, "disciplines_cropped_image_get", args).await;
    }

    let ctx = CommandContext::from_tauri_state(&state);
    let project_path = ctx.locked_project_path()?;
    ctx.db(|db| disciplines_service::disciplines_cropped_image_get(&project_path, db, args))
}
