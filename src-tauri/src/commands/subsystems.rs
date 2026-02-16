use super::executor::{
    dispatch_args, dispatch_args_async, dispatch_no_args, PlatformArg, PlatformOut,
};
use super::state::AppState;
use ralph_contracts::disciplines::{
    DisciplineConfig, DisciplinesCreateArgs, DisciplinesCroppedImageGetArgs, DisciplinesDeleteArgs,
    DisciplinesImageDataGetArgs, DisciplinesUpdateArgs,
};
use ralph_contracts::subsystems::{
    SubsystemData, SubsystemsCommentAddArgs, SubsystemsCommentDeleteArgs,
    SubsystemsCommentUpdateArgs, SubsystemsCreateArgs, SubsystemsDeleteArgs, SubsystemsUpdateArgs,
};
use ralph_errors::RalphResult;
use tauri::State;

#[tauri::command]
pub async fn disciplines_list(
    state: State<'_, AppState>,
) -> RalphResult<PlatformOut<Vec<DisciplineConfig>>> {
    dispatch_no_args(state.inner(), "disciplines_list", || {
        local::disciplines_list(&state)
    })
    .await
}

#[tauri::command]
pub async fn subsystems_list(
    state: State<'_, AppState>,
) -> RalphResult<PlatformOut<Vec<SubsystemData>>> {
    dispatch_no_args(state.inner(), "subsystems_list", || {
        local::subsystems_list(&state)
    })
    .await
}

#[tauri::command]
pub async fn subsystems_create(
    state: State<'_, AppState>,
    args: PlatformArg<SubsystemsCreateArgs>,
) -> RalphResult<PlatformOut<SubsystemData>> {
    dispatch_args(state.inner(), "subsystems_create", args, |args| {
        local::subsystems_create(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn subsystems_update(
    state: State<'_, AppState>,
    args: PlatformArg<SubsystemsUpdateArgs>,
) -> RalphResult<PlatformOut<SubsystemData>> {
    dispatch_args(state.inner(), "subsystems_update", args, |args| {
        local::subsystems_update(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn subsystems_comment_add(
    state: State<'_, AppState>,
    args: PlatformArg<SubsystemsCommentAddArgs>,
) -> RalphResult<PlatformOut<SubsystemData>> {
    dispatch_args_async(state.inner(), "subsystems_comment_add", args, |args| {
        local::subsystems_comment_add(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn subsystems_comment_update(
    state: State<'_, AppState>,
    args: PlatformArg<SubsystemsCommentUpdateArgs>,
) -> RalphResult<PlatformOut<SubsystemData>> {
    dispatch_args_async(state.inner(), "subsystems_comment_update", args, |args| {
        local::subsystems_comment_update(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn subsystems_comment_delete(
    state: State<'_, AppState>,
    args: PlatformArg<SubsystemsCommentDeleteArgs>,
) -> RalphResult<PlatformOut<SubsystemData>> {
    dispatch_args(state.inner(), "subsystems_comment_delete", args, |args| {
        local::subsystems_comment_delete(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn disciplines_create(
    state: State<'_, AppState>,
    args: PlatformArg<DisciplinesCreateArgs>,
) -> RalphResult<PlatformOut<DisciplineConfig>> {
    dispatch_args(state.inner(), "disciplines_create", args, |args| {
        local::disciplines_create(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn disciplines_update(
    state: State<'_, AppState>,
    args: PlatformArg<DisciplinesUpdateArgs>,
) -> RalphResult<PlatformOut<DisciplineConfig>> {
    dispatch_args(state.inner(), "disciplines_update", args, |args| {
        local::disciplines_update(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn subsystems_delete(
    state: State<'_, AppState>,
    args: PlatformArg<SubsystemsDeleteArgs>,
) -> RalphResult<()> {
    dispatch_args(state.inner(), "subsystems_delete", args, |args| {
        local::subsystems_delete(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn disciplines_delete(
    state: State<'_, AppState>,
    args: PlatformArg<DisciplinesDeleteArgs>,
) -> RalphResult<PlatformOut<String>> {
    dispatch_args(state.inner(), "disciplines_delete", args, |args| {
        local::disciplines_delete(&state, args)
    })
    .await
}

#[cfg_attr(not(mobile), ralph_macros::ipc_type)]
pub struct VisualIdentityData {
    pub style: String,
    pub theme: String,
    pub tone: String,
    pub references: String,
}

#[cfg_attr(not(mobile), ralph_macros::ipc_type)]
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
) -> RalphResult<PlatformOut<Vec<StackMetadataData>>> {
    dispatch_no_args(state.inner(), "stacks_metadata_list", || {
        #[cfg(not(mobile))]
        {
            Ok(predefined_disciplines::get_all_stack_metadata()
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
                .collect())
        }

        #[cfg(mobile)]
        {
            local::unreachable_local("stacks_metadata_list")
        }
    })
    .await
}

#[tauri::command]
pub async fn disciplines_image_data_get(
    state: State<'_, AppState>,
    args: PlatformArg<DisciplinesImageDataGetArgs>,
) -> RalphResult<PlatformOut<Option<String>>> {
    dispatch_args(state.inner(), "disciplines_image_data_get", args, |args| {
        local::disciplines_image_data_get(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn disciplines_cropped_image_get(
    state: State<'_, AppState>,
    args: PlatformArg<DisciplinesCroppedImageGetArgs>,
) -> RalphResult<PlatformOut<Option<String>>> {
    dispatch_args(
        state.inner(),
        "disciplines_cropped_image_get",
        args,
        |args| local::disciplines_cropped_image_get(&state, args),
    )
    .await
}

mod local {
    use super::*;

    pub(super) fn disciplines_list(
        state: &State<'_, AppState>,
    ) -> RalphResult<PlatformOut<Vec<DisciplineConfig>>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            use ralph_backend::disciplines_service;

            CommandContext::from_tauri_state(state).db(disciplines_service::disciplines_list)
        }

        #[cfg(mobile)]
        {
            let _ = state;
            unreachable_local("disciplines_list")
        }
    }

    pub(super) fn subsystems_list(
        state: &State<'_, AppState>,
    ) -> RalphResult<PlatformOut<Vec<SubsystemData>>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            use ralph_backend::subsystems_service;

            CommandContext::from_tauri_state(state).db(subsystems_service::subsystems_list)
        }

        #[cfg(mobile)]
        {
            let _ = state;
            unreachable_local("subsystems_list")
        }
    }

    pub(super) fn subsystems_create(
        state: &State<'_, AppState>,
        args: PlatformArg<SubsystemsCreateArgs>,
    ) -> RalphResult<PlatformOut<SubsystemData>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            use ralph_backend::subsystems_service;

            CommandContext::from_tauri_state(state)
                .db(|db| subsystems_service::subsystems_create(db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("subsystems_create")
        }
    }

    pub(super) fn subsystems_update(
        state: &State<'_, AppState>,
        args: PlatformArg<SubsystemsUpdateArgs>,
    ) -> RalphResult<PlatformOut<SubsystemData>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            use ralph_backend::subsystems_service;

            CommandContext::from_tauri_state(state)
                .db(|db| subsystems_service::subsystems_update(db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("subsystems_update")
        }
    }

    pub(super) async fn subsystems_comment_add(
        state: &State<'_, AppState>,
        args: PlatformArg<SubsystemsCommentAddArgs>,
    ) -> RalphResult<PlatformOut<SubsystemData>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            use ralph_backend::subsystems_service;

            let ctx = CommandContext::from_tauri_state(state);
            let project_path = ctx.locked_project_path()?;

            let (subsystem, embed_work) =
                ctx.db(|db| subsystems_service::subsystems_comment_add_prepare(db, args))?;
            subsystems_service::subsystems_comment_apply_embedding(&project_path, embed_work)
                .await?;

            Ok(subsystem)
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("subsystems_comment_add")
        }
    }

    pub(super) async fn subsystems_comment_update(
        state: &State<'_, AppState>,
        args: PlatformArg<SubsystemsCommentUpdateArgs>,
    ) -> RalphResult<PlatformOut<SubsystemData>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            use ralph_backend::subsystems_service;

            let ctx = CommandContext::from_tauri_state(state);
            let project_path = ctx.locked_project_path()?;

            let (subsystem, embed_work) =
                ctx.db(|db| subsystems_service::subsystems_comment_update_prepare(db, args))?;

            if let Some(work) = embed_work {
                subsystems_service::subsystems_comment_apply_embedding(&project_path, work).await?;
            }

            Ok(subsystem)
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("subsystems_comment_update")
        }
    }

    pub(super) fn subsystems_comment_delete(
        state: &State<'_, AppState>,
        args: PlatformArg<SubsystemsCommentDeleteArgs>,
    ) -> RalphResult<PlatformOut<SubsystemData>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            use ralph_backend::subsystems_service;

            CommandContext::from_tauri_state(state)
                .db(|db| subsystems_service::subsystems_comment_delete(db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("subsystems_comment_delete")
        }
    }

    pub(super) fn disciplines_create(
        state: &State<'_, AppState>,
        args: PlatformArg<DisciplinesCreateArgs>,
    ) -> RalphResult<PlatformOut<DisciplineConfig>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            use ralph_backend::disciplines_service;

            CommandContext::from_tauri_state(state)
                .db(|db| disciplines_service::disciplines_create(db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("disciplines_create")
        }
    }

    pub(super) fn disciplines_update(
        state: &State<'_, AppState>,
        args: PlatformArg<DisciplinesUpdateArgs>,
    ) -> RalphResult<PlatformOut<DisciplineConfig>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            use ralph_backend::disciplines_service;

            CommandContext::from_tauri_state(state)
                .db(|db| disciplines_service::disciplines_update(db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("disciplines_update")
        }
    }

    pub(super) fn subsystems_delete(
        state: &State<'_, AppState>,
        args: PlatformArg<SubsystemsDeleteArgs>,
    ) -> RalphResult<()> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            use ralph_backend::subsystems_service;

            CommandContext::from_tauri_state(state)
                .db(|db| subsystems_service::subsystems_delete(db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("subsystems_delete")
        }
    }

    pub(super) fn disciplines_delete(
        state: &State<'_, AppState>,
        args: PlatformArg<DisciplinesDeleteArgs>,
    ) -> RalphResult<PlatformOut<String>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            use ralph_backend::disciplines_service;

            CommandContext::from_tauri_state(state)
                .db(|db| disciplines_service::disciplines_delete(db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("disciplines_delete")
        }
    }

    pub(super) fn disciplines_image_data_get(
        state: &State<'_, AppState>,
        args: PlatformArg<DisciplinesImageDataGetArgs>,
    ) -> RalphResult<PlatformOut<Option<String>>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            use ralph_backend::disciplines_service;

            let ctx = CommandContext::from_tauri_state(state);
            let project_path = ctx.locked_project_path()?;
            ctx.db(|db| disciplines_service::disciplines_image_data_get(&project_path, db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("disciplines_image_data_get")
        }
    }

    pub(super) fn disciplines_cropped_image_get(
        state: &State<'_, AppState>,
        args: PlatformArg<DisciplinesCroppedImageGetArgs>,
    ) -> RalphResult<PlatformOut<Option<String>>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            use ralph_backend::disciplines_service;

            let ctx = CommandContext::from_tauri_state(state);
            let project_path = ctx.locked_project_path()?;
            ctx.db(|db| disciplines_service::disciplines_cropped_image_get(&project_path, db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("disciplines_cropped_image_get")
        }
    }

    #[cfg(mobile)]
    pub(super) fn unreachable_local<TResult>(command: &str) -> RalphResult<TResult> {
        use ralph_errors::{codes, ralph_err};
        ralph_err!(
            codes::INTERNAL,
            "Local execution path reached on mobile for '{}'",
            command
        )
    }
}
