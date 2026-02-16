use core_contracts::transport::EventSink;
use data_sqlite::SqliteDb;
use prompt_builder::CodebaseSnapshot;
use service_runtime::xdg::XdgDirs;
use service_terminal::terminal::PTYManager;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

pub struct AppState {
    pub locked_project: Mutex<Option<PathBuf>>,
    pub db: Mutex<Option<SqliteDb>>,
    pub codebase_snapshot: Mutex<Option<CodebaseSnapshot>>,
    pub pty_manager: PTYManager,
    pub mcp_dir: PathBuf,
    pub api_server_port: Mutex<Option<u16>>,
    pub xdg: XdgDirs,
    pub event_tx: broadcast::Sender<String>,
    pub event_sink: Arc<dyn EventSink>,
}

impl Default for AppState {
    fn default() -> Self {
        let xdg = XdgDirs::resolve().unwrap_or_else(|error| {
            panic!("Failed to resolve XDG directories: {error}");
        });

        let (event_tx, _) = broadcast::channel::<String>(1024);
        let sink: Arc<dyn EventSink> =
            Arc::new(crate::event_sink::RalphdEventSink::new(event_tx.clone()));

        Self {
            locked_project: Mutex::new(None),
            db: Mutex::new(None),
            codebase_snapshot: Mutex::new(None),
            pty_manager: PTYManager::new(),
            mcp_dir: std::env::temp_dir().join(format!("ralphd-mcp-{}", std::process::id())),
            api_server_port: Mutex::new(None),
            xdg,
            event_tx,
            event_sink: sink,
        }
    }
}

impl Drop for AppState {
    fn drop(&mut self) {
        match std::fs::remove_dir_all(&self.mcp_dir) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                tracing::warn!(error = %error, path = %self.mcp_dir.display(), "Failed to remove mcp dir");
            }
        }
    }
}
