use super::executor::{dispatch_args, dispatch_no_args, PlatformArg, PlatformOut};
use super::state::AppState;
use core_contracts::prompt_builder::{
    PromptBuilderConfigData, PromptBuilderConfigDeleteArgs, PromptBuilderConfigGetArgs,
    PromptBuilderConfigSaveArgs, PromptBuilderPreviewArgs, PromptPreview,
};
use core_errors::RalphResult;
use tauri::State;

#[tauri::command]
pub async fn prompt_builder_preview(
    state: State<'_, AppState>,
    args: PlatformArg<PromptBuilderPreviewArgs>,
) -> RalphResult<PlatformOut<PromptPreview>> {
    dispatch_args(state.inner(), "prompt_builder_preview", args, |args| {
        local::prompt_builder_preview(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn prompt_builder_config_list(
    state: State<'_, AppState>,
) -> RalphResult<PlatformOut<Vec<String>>> {
    dispatch_no_args(state.inner(), "prompt_builder_config_list", || {
        local::prompt_builder_config_list(&state)
    })
    .await
}

#[tauri::command]
pub async fn prompt_builder_config_get(
    state: State<'_, AppState>,
    args: PlatformArg<PromptBuilderConfigGetArgs>,
) -> RalphResult<PlatformOut<Option<PromptBuilderConfigData>>> {
    dispatch_args(state.inner(), "prompt_builder_config_get", args, |args| {
        local::prompt_builder_config_get(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn prompt_builder_config_save(
    state: State<'_, AppState>,
    args: PlatformArg<PromptBuilderConfigSaveArgs>,
) -> RalphResult<()> {
    dispatch_args(state.inner(), "prompt_builder_config_save", args, |args| {
        local::prompt_builder_config_save(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn prompt_builder_config_delete(
    state: State<'_, AppState>,
    args: PlatformArg<PromptBuilderConfigDeleteArgs>,
) -> RalphResult<()> {
    dispatch_args(
        state.inner(),
        "prompt_builder_config_delete",
        args,
        |args| local::prompt_builder_config_delete(&state, args),
    )
    .await
}

mod local {
    use super::*;

    pub(super) fn prompt_builder_preview(
        state: &State<'_, AppState>,
        args: PlatformArg<PromptBuilderPreviewArgs>,
    ) -> RalphResult<PlatformOut<PromptPreview>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            use core_errors::{codes, RalphResultExt};
            use service_prompts::prompt_builder_preview::PromptBuilderPreviewDeps;

            let ctx = CommandContext::from_tauri_state(state);
            let project_path = ctx.locked_project_path()?;

            let api_port = *state
                .inner()
                .api_server_port
                .lock()
                .ralph_err(codes::INTERNAL, "API server port mutex poisoned")?;

            ctx.db(|db| {
                service_prompts::prompt_builder_preview::prompt_builder_preview(
                    PromptBuilderPreviewDeps {
                        db,
                        project_path: &project_path,
                        mcp_dir: &state.inner().mcp_dir,
                        codebase_snapshot: &state.inner().codebase_snapshot,
                        api_server_port: api_port,
                    },
                    args,
                )
            })
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("prompt_builder_preview")
        }
    }

    pub(super) fn prompt_builder_config_list(
        state: &State<'_, AppState>,
    ) -> RalphResult<PlatformOut<Vec<String>>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            use service_prompts::prompt_builder_configs_service;

            CommandContext::from_tauri_state(state)
                .db(prompt_builder_configs_service::prompt_builder_config_list)
        }

        #[cfg(mobile)]
        {
            let _ = state;
            unreachable_local("prompt_builder_config_list")
        }
    }

    pub(super) fn prompt_builder_config_get(
        state: &State<'_, AppState>,
        args: PlatformArg<PromptBuilderConfigGetArgs>,
    ) -> RalphResult<PlatformOut<Option<PromptBuilderConfigData>>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            use service_prompts::prompt_builder_configs_service;

            CommandContext::from_tauri_state(state)
                .db(|db| prompt_builder_configs_service::prompt_builder_config_get(db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("prompt_builder_config_get")
        }
    }

    pub(super) fn prompt_builder_config_save(
        state: &State<'_, AppState>,
        args: PlatformArg<PromptBuilderConfigSaveArgs>,
    ) -> RalphResult<()> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            use service_prompts::prompt_builder_configs_service;

            CommandContext::from_tauri_state(state)
                .db(|db| prompt_builder_configs_service::prompt_builder_config_save(db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("prompt_builder_config_save")
        }
    }

    pub(super) fn prompt_builder_config_delete(
        state: &State<'_, AppState>,
        args: PlatformArg<PromptBuilderConfigDeleteArgs>,
    ) -> RalphResult<()> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            use service_prompts::prompt_builder_configs_service;

            CommandContext::from_tauri_state(state)
                .db(|db| prompt_builder_configs_service::prompt_builder_config_delete(db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("prompt_builder_config_delete")
        }
    }

    #[cfg(mobile)]
    fn unreachable_local<TResult>(command: &str) -> RalphResult<TResult> {
        use core_errors::{codes, ralph_err};
        ralph_err!(
            codes::INTERNAL,
            "Local execution path reached on mobile for '{}'",
            command
        )
    }
}
