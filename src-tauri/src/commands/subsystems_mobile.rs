use super::remote_proxy::{remote_invoke_args, remote_invoke_no_args};
use super::state::AppState;
use ralph_backend::disciplines_contract::{
    DisciplineConfig, DisciplinesCreateArgs, DisciplinesCroppedImageGetArgs, DisciplinesDeleteArgs,
    DisciplinesImageDataGetArgs, DisciplinesUpdateArgs,
};
use ralph_backend::subsystems_contract::{
    SubsystemData, SubsystemsCommentAddArgs, SubsystemsCommentDeleteArgs,
    SubsystemsCommentUpdateArgs, SubsystemsCreateArgs, SubsystemsDeleteArgs, SubsystemsUpdateArgs,
};
use ralph_errors::RalphResult;
use ralph_macros::ipc_type;
use tauri::State;

#[tauri::command]
pub async fn disciplines_list(state: State<'_, AppState>) -> RalphResult<Vec<DisciplineConfig>> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_no_args(&rpc, "disciplines_list").await
}

#[tauri::command]
pub async fn subsystems_list(state: State<'_, AppState>) -> RalphResult<Vec<SubsystemData>> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_no_args(&rpc, "subsystems_list").await
}

#[tauri::command]
pub async fn subsystems_create(
    state: State<'_, AppState>,
    args: SubsystemsCreateArgs,
) -> RalphResult<SubsystemData> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "subsystems_create", args).await
}

#[tauri::command]
pub async fn subsystems_update(
    state: State<'_, AppState>,
    args: SubsystemsUpdateArgs,
) -> RalphResult<SubsystemData> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "subsystems_update", args).await
}

#[tauri::command]
pub async fn subsystems_comment_add(
    state: State<'_, AppState>,
    args: SubsystemsCommentAddArgs,
) -> RalphResult<SubsystemData> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "subsystems_comment_add", args).await
}

#[tauri::command]
pub async fn subsystems_comment_update(
    state: State<'_, AppState>,
    args: SubsystemsCommentUpdateArgs,
) -> RalphResult<SubsystemData> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "subsystems_comment_update", args).await
}

#[tauri::command]
pub async fn subsystems_comment_delete(
    state: State<'_, AppState>,
    args: SubsystemsCommentDeleteArgs,
) -> RalphResult<SubsystemData> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "subsystems_comment_delete", args).await
}

#[tauri::command]
pub async fn disciplines_create(
    state: State<'_, AppState>,
    args: DisciplinesCreateArgs,
) -> RalphResult<DisciplineConfig> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "disciplines_create", args).await
}

#[tauri::command]
pub async fn disciplines_update(
    state: State<'_, AppState>,
    args: DisciplinesUpdateArgs,
) -> RalphResult<DisciplineConfig> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "disciplines_update", args).await
}

#[tauri::command]
pub async fn subsystems_delete(
    state: State<'_, AppState>,
    args: SubsystemsDeleteArgs,
) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "subsystems_delete", args).await
}

#[tauri::command]
pub async fn disciplines_delete(
    state: State<'_, AppState>,
    args: DisciplinesDeleteArgs,
) -> RalphResult<String> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "disciplines_delete", args).await
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
pub async fn stacks_metadata_list(
    state: State<'_, AppState>,
) -> RalphResult<Vec<StackMetadataData>> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_no_args(&rpc, "stacks_metadata_list").await
}

#[tauri::command]
pub async fn disciplines_image_data_get(
    state: State<'_, AppState>,
    args: DisciplinesImageDataGetArgs,
) -> RalphResult<Option<String>> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "disciplines_image_data_get", args).await
}

#[tauri::command]
pub async fn disciplines_cropped_image_get(
    state: State<'_, AppState>,
    args: DisciplinesCroppedImageGetArgs,
) -> RalphResult<Option<String>> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "disciplines_cropped_image_get", args).await
}
