use super::executor::{dispatch_args, dispatch_no_args, PlatformArg, PlatformOut};
use super::state::AppState;
use ralph_contracts::agent_sessions::AgentSessionsByIdArgs;
use ralph_contracts::domain::{AgentSession, AgentSessionCreateInput, AgentSessionUpdateInput};
use ralph_errors::RalphResult;
use tauri::State;

#[tauri::command]
pub async fn agent_sessions_create_human(
    state: State<'_, AppState>,
    args: PlatformArg<AgentSessionCreateInput>,
) -> RalphResult<()> {
    dispatch_args(state.inner(), "agent_sessions_create_human", args, |args| {
        local::agent_sessions_create_human(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn agent_sessions_update_human(
    state: State<'_, AppState>,
    args: PlatformArg<AgentSessionUpdateInput>,
) -> RalphResult<()> {
    dispatch_args(state.inner(), "agent_sessions_update_human", args, |args| {
        local::agent_sessions_update_human(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn agent_sessions_delete_human(
    state: State<'_, AppState>,
    args: PlatformArg<AgentSessionsByIdArgs>,
) -> RalphResult<()> {
    dispatch_args(state.inner(), "agent_sessions_delete_human", args, |args| {
        local::agent_sessions_delete_human(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn agent_sessions_get(
    state: State<'_, AppState>,
    args: PlatformArg<AgentSessionsByIdArgs>,
) -> RalphResult<PlatformOut<Option<AgentSession>>> {
    dispatch_args(state.inner(), "agent_sessions_get", args, |args| {
        local::agent_sessions_get(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn agent_sessions_list_human(
    state: State<'_, AppState>,
) -> RalphResult<PlatformOut<Vec<AgentSession>>> {
    dispatch_no_args(state.inner(), "agent_sessions_list_human", || {
        local::agent_sessions_list_human(&state)
    })
    .await
}

mod local {
    use super::*;

    pub(super) fn agent_sessions_create_human(
        state: &State<'_, AppState>,
        args: PlatformArg<AgentSessionCreateInput>,
    ) -> RalphResult<()> {
        #[cfg(not(mobile))]
        {
            use super::super::state::with_db;
            use ralph_backend::agent_sessions_service;

            with_db(state, |db| {
                agent_sessions_service::agent_sessions_create_human(db, args)
            })
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("agent_sessions_create_human")
        }
    }

    pub(super) fn agent_sessions_update_human(
        state: &State<'_, AppState>,
        args: PlatformArg<AgentSessionUpdateInput>,
    ) -> RalphResult<()> {
        #[cfg(not(mobile))]
        {
            use super::super::state::with_db;
            use ralph_backend::agent_sessions_service;

            with_db(state, |db| {
                agent_sessions_service::agent_sessions_update_human(db, args)
            })
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("agent_sessions_update_human")
        }
    }

    pub(super) fn agent_sessions_delete_human(
        state: &State<'_, AppState>,
        args: PlatformArg<AgentSessionsByIdArgs>,
    ) -> RalphResult<()> {
        #[cfg(not(mobile))]
        {
            use super::super::state::with_db;
            use ralph_backend::agent_sessions_service;

            with_db(state, |db| {
                agent_sessions_service::agent_sessions_delete_human(db, &args.id)
            })
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("agent_sessions_delete_human")
        }
    }

    pub(super) fn agent_sessions_get(
        state: &State<'_, AppState>,
        args: PlatformArg<AgentSessionsByIdArgs>,
    ) -> RalphResult<PlatformOut<Option<AgentSession>>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::with_db;
            use ralph_backend::agent_sessions_service;

            with_db(state, |db| {
                agent_sessions_service::agent_sessions_get(db, &args.id)
            })
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("agent_sessions_get")
        }
    }

    pub(super) fn agent_sessions_list_human(
        state: &State<'_, AppState>,
    ) -> RalphResult<PlatformOut<Vec<AgentSession>>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::with_db;
            use ralph_backend::agent_sessions_service;

            with_db(state, agent_sessions_service::agent_sessions_list_human)
        }

        #[cfg(mobile)]
        {
            let _ = state;
            unreachable_local("agent_sessions_list_human")
        }
    }

    #[cfg(mobile)]
    fn unreachable_local<TResult>(command: &str) -> RalphResult<TResult> {
        use ralph_errors::{codes, ralph_err};
        ralph_err!(
            codes::INTERNAL,
            "Local execution path reached on mobile for '{}'",
            command
        )
    }
}
