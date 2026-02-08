mod commands;
mod terminal;

use commands::AppState;
use tauri::{Emitter, Manager};
use tauri_plugin_cli::CliExt;

fn init_tracing() {
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| {
            EnvFilter::try_new(if cfg!(debug_assertions) {
                "ralph4days=debug,sqlite_db=debug,prompt_builder=debug"
            } else {
                "ralph4days=info,sqlite_db=info,prompt_builder=info"
            })
        })
        .unwrap();

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(true).with_line_number(true))
        .init();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _start = std::time::Instant::now();
    init_tracing();

    // WORKAROUND: WebKitGTK + NVIDIA + Wayland crash prevention
    // See: https://github.com/tauri-apps/tauri/issues/9394
    // __NV_DISABLE_EXPLICIT_SYNC=1 targets the specific NVIDIA sync issue
    // without disabling GPU acceleration entirely (unlike WEBKIT_DISABLE_DMABUF_RENDERER=1
    // which forces software rendering and causes massive input lag).
    #[cfg(target_os = "linux")]
    {
        std::env::set_var("__NV_DISABLE_EXPLICIT_SYNC", "1");
    };

    tracing::info!("Starting Ralph4days");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_cli::init())
        .manage(AppState::default())
        .setup(|app| {
            let setup_start = std::time::Instant::now();

            if let Some(splash) = app.get_webview_window("splash") {
                let _ = splash.emit("loading", "Initializing...");
                tracing::debug!("Setup phase started at {:?}", setup_start.elapsed());
            }

            if let Ok(matches) = app.cli().matches() {
                if let Some(project_path) = matches.args.get("project") {
                    if let serde_json::Value::String(path_str) = &project_path.value {
                        if let Some(splash) = app.get_webview_window("splash") {
                            let _ = splash.emit("loading", "Validating project...");
                        }

                        if let Err(e) = commands::validate_project_path(path_str.clone()) {
                            eprintln!("Failed to lock project: {e}");
                            std::process::exit(1);
                        }

                        if let Some(splash) = app.get_webview_window("splash") {
                            let _ = splash.emit("loading", "Locking project...");
                        }

                        let state: &AppState = app.state::<AppState>().inner();
                        if let Err(e) = commands::lock_project_validated(state, path_str.clone()) {
                            eprintln!("Error: {e}");
                            std::process::exit(1);
                        }
                    }
                }
            }

            tracing::debug!("Setup completed in {:?}", setup_start.elapsed());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::project::start_loop,
            commands::project::pause_loop,
            commands::project::resume_loop,
            commands::project::stop_loop,
            commands::project::get_loop_state,
            commands::project::scan_for_ralph_projects,
            commands::project::get_current_dir,
            commands::project::validate_project_path,
            commands::project::initialize_ralph_project,
            commands::project::set_locked_project,
            commands::project::get_locked_project,
            commands::project::get_project_info,
            commands::project::close_splash,
            commands::tasks::create_task,
            commands::tasks::update_task,
            commands::tasks::set_task_status,
            commands::tasks::delete_task,
            commands::tasks::add_task_comment,
            commands::tasks::update_task_comment,
            commands::tasks::delete_task_comment,
            commands::tasks::get_tasks,
            commands::features::get_disciplines_config,
            commands::features::get_features,
            commands::features::create_feature,
            commands::features::update_feature,
            commands::features::delete_feature,
            commands::features::append_feature_learning,
            commands::features::remove_feature_learning,
            commands::features::add_feature_context_file,
            commands::features::create_discipline,
            commands::features::update_discipline,
            commands::features::delete_discipline,
            commands::features::get_stack_metadata,
            commands::image_gen::get_image_prompt_config,
            commands::image_gen::build_image_prompt,
            commands::image_gen::get_stack_disciplines,
            commands::prompts::preview_custom_recipe,
            commands::prompts::list_recipe_configs,
            commands::prompts::get_recipe_config,
            commands::prompts::save_recipe_config,
            commands::prompts::delete_recipe_config,
            commands::terminal::create_pty_session,
            commands::terminal::create_pty_session_for_task,
            commands::terminal::send_terminal_input,
            commands::terminal::resize_pty,
            commands::terminal::terminate_pty_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
