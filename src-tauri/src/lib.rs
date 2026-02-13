mod api_server;
mod commands;
mod diagnostics;
mod recent_projects;
mod terminal;
mod xdg;

use commands::AppState;
use tauri::Manager;
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

pub fn list_provider_models(agent: Option<&str>) -> Vec<String> {
    terminal::providers::list_models_for_agent(agent)
}

pub fn list_provider_model_entries(agent: Option<&str>) -> Vec<terminal::providers::ModelEntry> {
    terminal::providers::list_model_entries_for_agent(agent)
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
            // Start API server for MCP signal communication
            let app_handle = app.handle().clone();
            diagnostics::register_app_handle(&app_handle);
            let state: tauri::State<AppState> = app.state();

            tauri::async_runtime::block_on(async {
                match api_server::start_api_server(app_handle).await {
                    Ok(port) => {
                        *state.api_server_port.lock().unwrap() = Some(port);
                        tracing::info!("API server started on port {}", port);
                    }
                    Err(e) => {
                        tracing::error!("Failed to start API server: {}", e);
                    }
                }
            });

            // WHY: tao#1046 / tauri#11856 — on Wayland, a window created with
            // visible(false) then shown via .show() has a stale CSD input region,
            // making decoration buttons unclickable. Both windows are created here
            // in order: main first (born visible), then splash on top.
            let _main = tauri::WebviewWindowBuilder::new(
                app,
                "main",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .title("Ralph4days")
            .inner_size(1400.0, 900.0)
            .min_inner_size(900.0, 600.0)
            .center()
            .resizable(true)
            .maximizable(true)
            .decorations(true)
            .visible(true)
            .focused(false)
            .build()?;

            let _splash = tauri::WebviewWindowBuilder::new(
                app,
                "splash",
                tauri::WebviewUrl::App("splash.html".into()),
            )
            .inner_size(400.0, 250.0)
            .center()
            .decorations(false)
            .skip_taskbar(true)
            .resizable(false)
            .always_on_top(true)
            .focused(true)
            .build()?;

            if let Ok(matches) = app.cli().matches() {
                if let Some(project_path) = matches.args.get("project") {
                    if let serde_json::Value::String(path_str) = &project_path.value {
                        if let Err(e) = commands::validate_project_path(path_str.clone()) {
                            eprintln!("Failed to lock project: {e}");
                            std::process::exit(1);
                        }

                        let state: &AppState = app.state::<AppState>().inner();
                        if let Err(e) = commands::lock_project_validated(state, path_str.clone()) {
                            eprintln!("Error: {e}");
                            std::process::exit(1);
                        }
                    }
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::project::start_execution_sequence,
            commands::project::pause_execution_sequence,
            commands::project::resume_execution_sequence,
            commands::project::stop_execution_sequence,
            commands::project::get_execution_sequence_state,
            commands::project::scan_for_ralph_projects,
            commands::project::get_current_dir,
            commands::project::validate_project_path,
            commands::project::initialize_ralph_project,
            commands::project::set_locked_project,
            commands::project::get_locked_project,
            commands::project::get_recent_projects,
            commands::project::get_project_info,
            commands::project::close_splash,
            commands::project::open_new_window,
            commands::tasks::create_task,
            commands::tasks::update_task,
            commands::tasks::set_task_status,
            commands::tasks::delete_task,
            commands::tasks::add_task_signal,
            commands::tasks::update_task_signal,
            commands::tasks::delete_task_signal,
            commands::tasks::get_tasks,
            commands::tasks::get_signal_summaries,
            commands::tasks::answer_ask,
            commands::tasks::add_reply_to_comment,
            commands::tasks::add_task_signal_comment,
            commands::tasks::update_task_signal_comment,
            commands::tasks::delete_task_signal_comment,
            commands::tasks::get_task_signal_comments,
            commands::agent_sessions::create_human_agent_session,
            commands::agent_sessions::update_human_agent_session,
            commands::agent_sessions::delete_human_agent_session,
            commands::agent_sessions::get_agent_session,
            commands::agent_sessions::list_human_agent_sessions,
            commands::subsystems::get_disciplines_config,
            commands::subsystems::get_subsystems,
            commands::subsystems::create_subsystem,
            commands::subsystems::update_subsystem,
            commands::subsystems::delete_subsystem,
            commands::subsystems::add_subsystem_comment,
            commands::subsystems::update_subsystem_comment,
            commands::subsystems::delete_subsystem_comment,
            commands::subsystems::create_discipline,
            commands::subsystems::update_discipline,
            commands::subsystems::delete_discipline,
            commands::subsystems::get_stack_metadata,
            commands::subsystems::get_discipline_image_data,
            commands::subsystems::get_cropped_image,
            commands::prompts::preview_custom_prompt_builder,
            commands::prompts::list_prompt_builder_configs,
            commands::prompts::get_prompt_builder_config,
            commands::prompts::save_prompt_builder_config,
            commands::prompts::delete_prompt_builder_config,
            commands::terminal_bridge::terminal_bridge_start_session,
            commands::terminal_bridge::terminal_bridge_start_task_session,
            commands::terminal_bridge::terminal_bridge_send_input,
            commands::terminal_bridge::terminal_bridge_resize,
            commands::terminal_bridge::terminal_bridge_terminate,
            commands::terminal_bridge::terminal_bridge_set_stream_mode,
            commands::terminal_bridge::terminal_bridge_replay_output,
            commands::terminal_bridge::terminal_bridge_emit_system_message,
            commands::terminal_bridge::terminal_bridge_start_human_session,
            commands::terminal_bridge::terminal_bridge_list_model_form_tree,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
