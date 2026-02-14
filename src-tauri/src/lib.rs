mod api_server;
mod commands;
mod diagnostics;
mod event_sink;
mod recent_projects;
mod remote;
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
            diagnostics::register_sink(std::sync::Arc::new(event_sink::TauriEventSink::new(
                app_handle.clone(),
            )));
            let state: tauri::State<AppState> = app.state();
            let mut skip_splash = false;

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
            if let Ok(matches) = app.cli().matches() {
                if let Some(no_splash) = matches.args.get("no-splash") {
                    if matches!(no_splash.value, serde_json::Value::Bool(true))
                        || matches!(&no_splash.value, serde_json::Value::String(value) if value == "true" || value.is_empty())
                    {
                        skip_splash = true;
                        tracing::info!("Skipping splash window via --no-splash");
                    }
                }
            }

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

            if !skip_splash {
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
            }

            if let Ok(matches) = app.cli().matches() {
                if let Some(project_path) = matches.args.get("project") {
                    if let serde_json::Value::String(path_str) = &project_path.value {
                        if let Err(e) = commands::project_validate_path(commands::project::ProjectValidatePathArgs {
                            path: path_str.clone(),
                        }) {
                            eprintln!("Failed to lock project: {e}");
                            std::process::exit(1);
                        }

                        let state: &AppState = app.state::<AppState>().inner();
                        if let Err(e) = commands::project_lock_validated(state, path_str.clone()) {
                            eprintln!("Error: {e}");
                            std::process::exit(1);
                        }
                    }
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::project::execution_start,
            commands::project::execution_pause,
            commands::project::execution_resume,
            commands::project::execution_stop,
            commands::project::execution_state_get,
            commands::project::project_scan,
            commands::project::system_home_dir_get,
            commands::project::project_validate_path,
            commands::project::project_initialize,
            commands::project::project_lock_set,
            commands::project::project_lock_get,
            commands::project::project_recent_list,
            commands::project::project_info_get,
            commands::project::window_splash_close,
            commands::project::window_open_new,
            commands::tasks::tasks_create,
            commands::tasks::tasks_update,
            commands::tasks::tasks_set_status,
            commands::tasks::tasks_delete,
            commands::tasks::tasks_signal_add,
            commands::tasks::tasks_signal_update,
            commands::tasks::tasks_signal_delete,
            commands::tasks::tasks_list,
            commands::tasks::tasks_get,
            commands::tasks::tasks_list_items,
            commands::tasks::tasks_signal_summaries_get,
            commands::tasks::tasks_ask_answer,
            commands::tasks::tasks_comment_reply_add,
            commands::tasks::tasks_signal_comment_add,
            commands::tasks::tasks_signal_comment_update,
            commands::tasks::tasks_signal_comment_delete,
            commands::tasks::tasks_signal_comments_list,
            commands::agent_sessions::agent_sessions_create_human,
            commands::agent_sessions::agent_sessions_update_human,
            commands::agent_sessions::agent_sessions_delete_human,
            commands::agent_sessions::agent_sessions_get,
            commands::agent_sessions::agent_sessions_list_human,
            commands::subsystems::disciplines_list,
            commands::subsystems::subsystems_list,
            commands::subsystems::subsystems_create,
            commands::subsystems::subsystems_update,
            commands::subsystems::subsystems_delete,
            commands::subsystems::subsystems_comment_add,
            commands::subsystems::subsystems_comment_update,
            commands::subsystems::subsystems_comment_delete,
            commands::subsystems::disciplines_create,
            commands::subsystems::disciplines_update,
            commands::subsystems::disciplines_delete,
            commands::subsystems::stacks_metadata_list,
            commands::subsystems::disciplines_image_data_get,
            commands::subsystems::disciplines_cropped_image_get,
            commands::prompts::prompt_builder_preview,
            commands::prompts::prompt_builder_config_list,
            commands::prompts::prompt_builder_config_get,
            commands::prompts::prompt_builder_config_save,
            commands::prompts::prompt_builder_config_delete,
            commands::protocol::protocol_version_get,
            commands::remote::remote_connect,
            commands::remote::remote_disconnect,
            commands::remote::remote_status_get,
            commands::terminal_bridge::terminal_start_session,
            commands::terminal_bridge::terminal_start_task_session,
            commands::terminal_bridge::terminal_send_input,
            commands::terminal_bridge::terminal_resize,
            commands::terminal_bridge::terminal_terminate,
            commands::terminal_bridge::terminal_set_stream_mode,
            commands::terminal_bridge::terminal_replay_output,
            commands::terminal_bridge::terminal_emit_system_message,
            commands::terminal_bridge::terminal_start_human_session,
            commands::terminal_bridge::terminal_list_model_form_tree,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
