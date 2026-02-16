use super::remote_proxy::{remote_invoke_args, remote_invoke_no_args};
use super::state::AppState;
use ralph_backend::prompt_builder_configs_contract::{
    PromptBuilderConfigDeleteArgs, PromptBuilderConfigGetArgs, PromptBuilderConfigSaveArgs,
};
use ralph_backend::prompt_builder_preview::{PromptBuilderPreviewArgs, PromptPreview};
use ralph_errors::RalphResult;
use sqlite_db::PromptBuilderConfigData;
use tauri::State;

#[tauri::command]
pub async fn prompt_builder_preview(
    state: State<'_, AppState>,
    args: PromptBuilderPreviewArgs,
) -> RalphResult<PromptPreview> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "prompt_builder_preview", args).await
}

#[tauri::command]
pub async fn prompt_builder_config_list(state: State<'_, AppState>) -> RalphResult<Vec<String>> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_no_args(&rpc, "prompt_builder_config_list").await
}

#[tauri::command]
pub async fn prompt_builder_config_get(
    state: State<'_, AppState>,
    args: PromptBuilderConfigGetArgs,
) -> RalphResult<Option<PromptBuilderConfigData>> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "prompt_builder_config_get", args).await
}

#[tauri::command]
pub async fn prompt_builder_config_save(
    state: State<'_, AppState>,
    args: PromptBuilderConfigSaveArgs,
) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "prompt_builder_config_save", args).await
}

#[tauri::command]
pub async fn prompt_builder_config_delete(
    state: State<'_, AppState>,
    args: PromptBuilderConfigDeleteArgs,
) -> RalphResult<()> {
    let rpc = state.inner().remote_rpc_client_required().await?;
    remote_invoke_args(&rpc, "prompt_builder_config_delete", args).await
}
