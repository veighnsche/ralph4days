use super::executor::{dispatch_args, dispatch_no_args, PlatformArg, PlatformOut};
use super::state::AppState;
use core_contracts::domain::{
    Task, TaskListItem, TaskSignalComment, TaskSignalCommentCreateInput, TaskSignalSummary,
};
use core_contracts::tasks::{
    TasksAskAnswerArgs, TasksCommentReplyAddArgs, TasksCreateArgs, TasksDeleteArgs, TasksGetArgs,
    TasksSetStatusArgs, TasksSignalAddArgs, TasksSignalCommentDeleteArgs,
    TasksSignalCommentUpdateArgs, TasksSignalCommentsListArgs, TasksSignalDeleteArgs,
    TasksSignalSummariesGetArgs, TasksSignalUpdateArgs, TasksUpdateArgs,
};
use core_errors::RalphResult;
use tauri::State;

#[tauri::command]
pub async fn tasks_create(
    state: State<'_, AppState>,
    args: PlatformArg<TasksCreateArgs>,
) -> RalphResult<PlatformOut<String>> {
    dispatch_args(state.inner(), "tasks_create", args, |args| {
        local::tasks_create(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn tasks_update(
    state: State<'_, AppState>,
    args: PlatformArg<TasksUpdateArgs>,
) -> RalphResult<PlatformOut<Task>> {
    dispatch_args(state.inner(), "tasks_update", args, |args| {
        local::tasks_update(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn tasks_set_status(
    state: State<'_, AppState>,
    args: PlatformArg<TasksSetStatusArgs>,
) -> RalphResult<PlatformOut<Task>> {
    dispatch_args(state.inner(), "tasks_set_status", args, |args| {
        local::tasks_set_status(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn tasks_delete(
    state: State<'_, AppState>,
    args: PlatformArg<TasksDeleteArgs>,
) -> RalphResult<()> {
    dispatch_args(state.inner(), "tasks_delete", args, |args| {
        local::tasks_delete(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn tasks_signal_add(
    state: State<'_, AppState>,
    args: PlatformArg<TasksSignalAddArgs>,
) -> RalphResult<PlatformOut<Task>> {
    dispatch_args(state.inner(), "tasks_signal_add", args, |args| {
        local::tasks_signal_add(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn tasks_signal_update(
    state: State<'_, AppState>,
    args: PlatformArg<TasksSignalUpdateArgs>,
) -> RalphResult<PlatformOut<Task>> {
    dispatch_args(state.inner(), "tasks_signal_update", args, |args| {
        local::tasks_signal_update(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn tasks_signal_delete(
    state: State<'_, AppState>,
    args: PlatformArg<TasksSignalDeleteArgs>,
) -> RalphResult<PlatformOut<Task>> {
    dispatch_args(state.inner(), "tasks_signal_delete", args, |args| {
        local::tasks_signal_delete(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn tasks_list(state: State<'_, AppState>) -> RalphResult<PlatformOut<Vec<Task>>> {
    dispatch_no_args(state.inner(), "tasks_list", || local::tasks_list(&state)).await
}

#[tauri::command]
pub async fn tasks_get(
    state: State<'_, AppState>,
    args: PlatformArg<TasksGetArgs>,
) -> RalphResult<PlatformOut<Task>> {
    dispatch_args(state.inner(), "tasks_get", args, |args| {
        local::tasks_get(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn tasks_list_items(
    state: State<'_, AppState>,
) -> RalphResult<PlatformOut<Vec<TaskListItem>>> {
    dispatch_no_args(state.inner(), "tasks_list_items", || {
        local::tasks_list_items(&state)
    })
    .await
}

#[tauri::command]
pub async fn tasks_signal_summaries_get(
    state: State<'_, AppState>,
    args: PlatformArg<TasksSignalSummariesGetArgs>,
) -> RalphResult<PlatformOut<std::collections::HashMap<u32, TaskSignalSummary>>> {
    dispatch_args(state.inner(), "tasks_signal_summaries_get", args, |args| {
        local::tasks_signal_summaries_get(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn tasks_ask_answer(
    state: State<'_, AppState>,
    args: PlatformArg<TasksAskAnswerArgs>,
) -> RalphResult<()> {
    dispatch_args(state.inner(), "tasks_ask_answer", args, |args| {
        local::tasks_ask_answer(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn tasks_comment_reply_add(
    state: State<'_, AppState>,
    args: PlatformArg<TasksCommentReplyAddArgs>,
) -> RalphResult<PlatformOut<Task>> {
    dispatch_args(state.inner(), "tasks_comment_reply_add", args, |args| {
        local::tasks_comment_reply_add(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn tasks_signal_comment_add(
    state: State<'_, AppState>,
    args: PlatformArg<TaskSignalCommentCreateInput>,
) -> RalphResult<PlatformOut<u32>> {
    dispatch_args(state.inner(), "tasks_signal_comment_add", args, |args| {
        local::tasks_signal_comment_add(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn tasks_signal_comment_update(
    state: State<'_, AppState>,
    args: PlatformArg<TasksSignalCommentUpdateArgs>,
) -> RalphResult<()> {
    dispatch_args(state.inner(), "tasks_signal_comment_update", args, |args| {
        local::tasks_signal_comment_update(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn tasks_signal_comment_delete(
    state: State<'_, AppState>,
    args: PlatformArg<TasksSignalCommentDeleteArgs>,
) -> RalphResult<()> {
    dispatch_args(state.inner(), "tasks_signal_comment_delete", args, |args| {
        local::tasks_signal_comment_delete(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn tasks_signal_comments_list(
    state: State<'_, AppState>,
    args: PlatformArg<TasksSignalCommentsListArgs>,
) -> RalphResult<PlatformOut<Vec<TaskSignalComment>>> {
    dispatch_args(state.inner(), "tasks_signal_comments_list", args, |args| {
        local::tasks_signal_comments_list(&state, args)
    })
    .await
}

mod local {
    use super::*;

    pub(super) fn tasks_create(
        state: &State<'_, AppState>,
        args: PlatformArg<TasksCreateArgs>,
    ) -> RalphResult<PlatformOut<String>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            CommandContext::from_tauri_state(state)
                .db(|db| service_tasks::tasks::tasks_create(db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("tasks_create")
        }
    }

    pub(super) fn tasks_update(
        state: &State<'_, AppState>,
        args: PlatformArg<TasksUpdateArgs>,
    ) -> RalphResult<PlatformOut<Task>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            CommandContext::from_tauri_state(state)
                .db(|db| service_tasks::tasks::tasks_update(db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("tasks_update")
        }
    }

    pub(super) fn tasks_set_status(
        state: &State<'_, AppState>,
        args: PlatformArg<TasksSetStatusArgs>,
    ) -> RalphResult<PlatformOut<Task>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            CommandContext::from_tauri_state(state)
                .db(|db| service_tasks::tasks::tasks_set_status(db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("tasks_set_status")
        }
    }

    pub(super) fn tasks_delete(
        state: &State<'_, AppState>,
        args: PlatformArg<TasksDeleteArgs>,
    ) -> RalphResult<()> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            CommandContext::from_tauri_state(state)
                .db(|db| service_tasks::tasks::tasks_delete(db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("tasks_delete")
        }
    }

    pub(super) fn tasks_signal_add(
        state: &State<'_, AppState>,
        args: PlatformArg<TasksSignalAddArgs>,
    ) -> RalphResult<PlatformOut<Task>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            CommandContext::from_tauri_state(state)
                .db(|db| service_tasks::tasks::tasks_signal_add(db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("tasks_signal_add")
        }
    }

    pub(super) fn tasks_signal_update(
        state: &State<'_, AppState>,
        args: PlatformArg<TasksSignalUpdateArgs>,
    ) -> RalphResult<PlatformOut<Task>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            CommandContext::from_tauri_state(state)
                .db(|db| service_tasks::tasks::tasks_signal_update(db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("tasks_signal_update")
        }
    }

    pub(super) fn tasks_signal_delete(
        state: &State<'_, AppState>,
        args: PlatformArg<TasksSignalDeleteArgs>,
    ) -> RalphResult<PlatformOut<Task>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            CommandContext::from_tauri_state(state)
                .db(|db| service_tasks::tasks::tasks_signal_delete(db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("tasks_signal_delete")
        }
    }

    pub(super) fn tasks_list(state: &State<'_, AppState>) -> RalphResult<PlatformOut<Vec<Task>>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            CommandContext::from_tauri_state(state).db(service_tasks::tasks::tasks_list)
        }

        #[cfg(mobile)]
        {
            let _ = state;
            unreachable_local("tasks_list")
        }
    }

    pub(super) fn tasks_get(
        state: &State<'_, AppState>,
        args: PlatformArg<TasksGetArgs>,
    ) -> RalphResult<PlatformOut<Task>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            CommandContext::from_tauri_state(state)
                .db(|db| service_tasks::tasks::tasks_get(db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("tasks_get")
        }
    }

    pub(super) fn tasks_list_items(
        state: &State<'_, AppState>,
    ) -> RalphResult<PlatformOut<Vec<TaskListItem>>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            CommandContext::from_tauri_state(state).db(service_tasks::tasks::tasks_list_items)
        }

        #[cfg(mobile)]
        {
            let _ = state;
            unreachable_local("tasks_list_items")
        }
    }

    pub(super) fn tasks_signal_summaries_get(
        state: &State<'_, AppState>,
        args: PlatformArg<TasksSignalSummariesGetArgs>,
    ) -> RalphResult<PlatformOut<std::collections::HashMap<u32, TaskSignalSummary>>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            CommandContext::from_tauri_state(state)
                .db(|db| service_tasks::tasks::tasks_signal_summaries_get(db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("tasks_signal_summaries_get")
        }
    }

    pub(super) fn tasks_ask_answer(
        state: &State<'_, AppState>,
        args: PlatformArg<TasksAskAnswerArgs>,
    ) -> RalphResult<()> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            CommandContext::from_tauri_state(state)
                .db(|db| service_tasks::tasks::tasks_ask_answer(db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("tasks_ask_answer")
        }
    }

    pub(super) fn tasks_comment_reply_add(
        state: &State<'_, AppState>,
        args: PlatformArg<TasksCommentReplyAddArgs>,
    ) -> RalphResult<PlatformOut<Task>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            CommandContext::from_tauri_state(state)
                .db(|db| service_tasks::tasks::tasks_comment_reply_add(db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("tasks_comment_reply_add")
        }
    }

    pub(super) fn tasks_signal_comment_add(
        state: &State<'_, AppState>,
        args: PlatformArg<TaskSignalCommentCreateInput>,
    ) -> RalphResult<PlatformOut<u32>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            CommandContext::from_tauri_state(state)
                .db(|db| service_tasks::tasks::tasks_signal_comment_add(db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("tasks_signal_comment_add")
        }
    }

    pub(super) fn tasks_signal_comment_update(
        state: &State<'_, AppState>,
        args: PlatformArg<TasksSignalCommentUpdateArgs>,
    ) -> RalphResult<()> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            CommandContext::from_tauri_state(state)
                .db(|db| service_tasks::tasks::tasks_signal_comment_update(db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("tasks_signal_comment_update")
        }
    }

    pub(super) fn tasks_signal_comment_delete(
        state: &State<'_, AppState>,
        args: PlatformArg<TasksSignalCommentDeleteArgs>,
    ) -> RalphResult<()> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            CommandContext::from_tauri_state(state)
                .db(|db| service_tasks::tasks::tasks_signal_comment_delete(db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("tasks_signal_comment_delete")
        }
    }

    pub(super) fn tasks_signal_comments_list(
        state: &State<'_, AppState>,
        args: PlatformArg<TasksSignalCommentsListArgs>,
    ) -> RalphResult<PlatformOut<Vec<TaskSignalComment>>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            CommandContext::from_tauri_state(state)
                .db(|db| service_tasks::tasks::tasks_signal_comments_list(db, args))
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("tasks_signal_comments_list")
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
