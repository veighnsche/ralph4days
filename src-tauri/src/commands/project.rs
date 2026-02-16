use super::executor::{dispatch_args, dispatch_no_args, PlatformArg, PlatformOut};
use super::state::AppState;
use core_contracts::project::{
    ProjectInfo, ProjectInitializeArgs, ProjectScanArgs, ProjectValidatePathArgs, RalphProject,
    RecentProject,
};
use core_contracts::session::ProjectLockSetArgs;
use core_errors::RalphResult;
#[cfg(not(mobile))]
use tauri::Manager;
use tauri::State;

#[tauri::command]
#[tracing::instrument(skip(state))]
pub async fn project_validate_path(
    state: State<'_, AppState>,
    args: PlatformArg<ProjectValidatePathArgs>,
) -> RalphResult<()> {
    dispatch_args(state.inner(), "project_validate_path", args, |args| {
        local::project_validate_path(&state, args)
    })
    .await
}

#[tauri::command]
#[tracing::instrument(skip(state))]
pub async fn project_initialize(
    state: State<'_, AppState>,
    args: PlatformArg<ProjectInitializeArgs>,
) -> RalphResult<()> {
    dispatch_args(state.inner(), "project_initialize", args, |args| {
        local::project_initialize(&state, args)
    })
    .await
}

#[cfg(not(mobile))]
pub fn project_lock_validated(state: &AppState, path: String) -> RalphResult<()> {
    let data_dir = state.xdg.ensure_data()?;
    let _canonical_path = service_project::session::project_lock_set_and_record_recent(
        &state.locked_project,
        &state.db,
        data_dir,
        ProjectLockSetArgs { path },
    )?;

    Ok(())
}

#[tauri::command]
pub async fn project_lock_set(
    state: State<'_, AppState>,
    args: PlatformArg<ProjectLockSetArgs>,
) -> RalphResult<()> {
    dispatch_args(state.inner(), "project_lock_set", args, |args| {
        local::project_lock_set(&state, args)
    })
    .await
}

#[tauri::command]
pub async fn project_lock_get(
    state: State<'_, AppState>,
) -> RalphResult<PlatformOut<Option<String>>> {
    dispatch_no_args(state.inner(), "project_lock_get", || {
        local::project_lock_get(&state)
    })
    .await
}

#[tauri::command]
pub async fn project_recent_list(
    state: State<'_, AppState>,
) -> RalphResult<PlatformOut<Vec<RecentProject>>> {
    dispatch_no_args(state.inner(), "project_recent_list", || {
        local::project_recent_list(&state)
    })
    .await
}

#[tauri::command]
pub async fn execution_start(state: State<'_, AppState>) -> RalphResult<()> {
    dispatch_no_args(state.inner(), "execution_start", local::execution_start).await
}

#[tauri::command]
pub async fn execution_pause(state: State<'_, AppState>) -> RalphResult<()> {
    dispatch_no_args(state.inner(), "execution_pause", local::execution_pause).await
}

#[tauri::command]
pub async fn execution_resume(state: State<'_, AppState>) -> RalphResult<()> {
    dispatch_no_args(state.inner(), "execution_resume", local::execution_resume).await
}

#[tauri::command]
pub async fn execution_stop(state: State<'_, AppState>) -> RalphResult<()> {
    dispatch_no_args(state.inner(), "execution_stop", local::execution_stop).await
}

#[tauri::command]
pub async fn execution_state_get(state: State<'_, AppState>) -> RalphResult<()> {
    dispatch_no_args(
        state.inner(),
        "execution_state_get",
        local::execution_state_get,
    )
    .await
}

#[tauri::command]
pub async fn project_scan(
    state: State<'_, AppState>,
    args: PlatformArg<ProjectScanArgs>,
) -> RalphResult<PlatformOut<Vec<RalphProject>>> {
    dispatch_args(state.inner(), "project_scan", args, local::project_scan).await
}

#[tauri::command]
pub async fn system_home_dir_get(state: State<'_, AppState>) -> RalphResult<PlatformOut<String>> {
    dispatch_no_args(
        state.inner(),
        "system_home_dir_get",
        local::system_home_dir_get,
    )
    .await
}

#[tauri::command]
pub async fn project_info_get(state: State<'_, AppState>) -> RalphResult<PlatformOut<ProjectInfo>> {
    dispatch_no_args(state.inner(), "project_info_get", || {
        local::project_info_get(&state)
    })
    .await
}

#[cfg(not(mobile))]
#[tauri::command]
pub fn window_splash_close(app: tauri::AppHandle) {
    if let Some(splash) = app.get_webview_window("splash") {
        let _ = splash.close();
    }
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.set_focus();
    }
}

#[tauri::command]
#[cfg(mobile)]
pub fn window_splash_close(app: tauri::AppHandle) -> RalphResult<()> {
    let _ = app;
    core_errors::ralph_err!(
        core_errors::codes::INTERNAL,
        "window_splash_close is unsupported on mobile"
    )
}

#[cfg(not(mobile))]
#[tauri::command]
pub fn window_open_new() -> RalphResult<()> {
    use core_errors::{codes, RalphResultExt};

    let exe = std::env::current_exe()
        .ralph_err(codes::INTERNAL, "Failed to get current executable path")?;
    std::process::Command::new(exe)
        .spawn()
        .ralph_err(codes::INTERNAL, "Failed to spawn new window")?;
    Ok(())
}

#[cfg(mobile)]
#[tauri::command]
pub fn window_open_new() -> RalphResult<()> {
    core_errors::ralph_err!(
        core_errors::codes::INTERNAL,
        "window_open_new is unsupported on mobile"
    )
}

mod local {
    use super::*;

    pub(super) fn project_validate_path(
        state: &State<'_, AppState>,
        args: PlatformArg<ProjectValidatePathArgs>,
    ) -> RalphResult<()> {
        #[cfg(not(mobile))]
        {
            let _ = state;
            let path = std::path::PathBuf::from(&args.path);
            service_project::project::validate_project_path(&path)
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("project_validate_path")
        }
    }

    pub(super) fn project_initialize(
        state: &State<'_, AppState>,
        args: PlatformArg<ProjectInitializeArgs>,
    ) -> RalphResult<()> {
        #[cfg(not(mobile))]
        {
            let _ = state;
            service_project::project::project_initialize(args)
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("project_initialize")
        }
    }

    pub(super) fn project_lock_set(
        state: &State<'_, AppState>,
        args: PlatformArg<ProjectLockSetArgs>,
    ) -> RalphResult<()> {
        #[cfg(not(mobile))]
        {
            super::project_lock_validated(state.inner(), args.path)
        }

        #[cfg(mobile)]
        {
            let _ = state;
            let _ = args;
            unreachable_local("project_lock_set")
        }
    }

    pub(super) fn project_lock_get(
        state: &State<'_, AppState>,
    ) -> RalphResult<PlatformOut<Option<String>>> {
        #[cfg(not(mobile))]
        {
            service_project::session::project_lock_get(&state.locked_project)
        }

        #[cfg(mobile)]
        {
            let _ = state;
            unreachable_local("project_lock_get")
        }
    }

    pub(super) fn project_recent_list(
        state: &State<'_, AppState>,
    ) -> RalphResult<PlatformOut<Vec<RecentProject>>> {
        #[cfg(not(mobile))]
        {
            use service_project::project_scan;

            let data_dir = state.inner().xdg.ensure_data()?;
            project_scan::recents_load(data_dir)
        }

        #[cfg(mobile)]
        {
            let _ = state;
            unreachable_local("project_recent_list")
        }
    }

    pub(super) fn execution_start() -> RalphResult<()> {
        #[cfg(not(mobile))]
        {
            core_errors::ralph_err!(core_errors::codes::LOOP_ENGINE, "Not implemented")
        }

        #[cfg(mobile)]
        {
            unreachable_local("execution_start")
        }
    }

    pub(super) fn execution_pause() -> RalphResult<()> {
        #[cfg(not(mobile))]
        {
            core_errors::ralph_err!(core_errors::codes::LOOP_ENGINE, "Not implemented")
        }

        #[cfg(mobile)]
        {
            unreachable_local("execution_pause")
        }
    }

    pub(super) fn execution_resume() -> RalphResult<()> {
        #[cfg(not(mobile))]
        {
            core_errors::ralph_err!(core_errors::codes::LOOP_ENGINE, "Not implemented")
        }

        #[cfg(mobile)]
        {
            unreachable_local("execution_resume")
        }
    }

    pub(super) fn execution_stop() -> RalphResult<()> {
        #[cfg(not(mobile))]
        {
            core_errors::ralph_err!(core_errors::codes::LOOP_ENGINE, "Not implemented")
        }

        #[cfg(mobile)]
        {
            unreachable_local("execution_stop")
        }
    }

    pub(super) fn execution_state_get() -> RalphResult<()> {
        #[cfg(not(mobile))]
        {
            core_errors::ralph_err!(core_errors::codes::LOOP_ENGINE, "Not implemented")
        }

        #[cfg(mobile)]
        {
            unreachable_local("execution_state_get")
        }
    }

    pub(super) fn project_scan(
        args: PlatformArg<ProjectScanArgs>,
    ) -> RalphResult<PlatformOut<Vec<RalphProject>>> {
        #[cfg(not(mobile))]
        {
            service_project::project_scan::project_scan(args)
        }

        #[cfg(mobile)]
        {
            let _ = args;
            unreachable_local("project_scan")
        }
    }

    pub(super) fn system_home_dir_get() -> RalphResult<PlatformOut<String>> {
        #[cfg(not(mobile))]
        {
            let path = dirs::home_dir().ok_or_else(|| {
                core_errors::err_string(
                    core_errors::codes::FILESYSTEM,
                    "Failed to get home directory",
                )
            })?;
            Ok(path.to_string_lossy().to_string())
        }

        #[cfg(mobile)]
        {
            unreachable_local("system_home_dir_get")
        }
    }

    pub(super) fn project_info_get(
        state: &State<'_, AppState>,
    ) -> RalphResult<PlatformOut<ProjectInfo>> {
        #[cfg(not(mobile))]
        {
            use super::super::state::CommandContext;
            use service_project::project_scan;

            CommandContext::from_tauri_state(state).db(project_scan::project_info_get)
        }

        #[cfg(mobile)]
        {
            let _ = state;
            unreachable_local("project_info_get")
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
