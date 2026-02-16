use super::{remote_rpc_client_from_transport, remote_rpc_client_required, RemoteTransport};
use core_errors::{codes, err_string, RalphResult, RalphResultExt};
use data_sqlite::SqliteDb;
use prompt_builder::CodebaseSnapshot;
use service_runtime::xdg::XdgDirs;
use service_terminal::terminal::PTYManager;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub locked_project: Mutex<Option<PathBuf>>,
    pub db: Mutex<Option<SqliteDb>>,
    pub codebase_snapshot: Mutex<Option<CodebaseSnapshot>>,
    pub pty_manager: PTYManager,
    pub remote: RemoteTransport,
    pub(crate) mcp_dir: PathBuf,
    pub xdg: XdgDirs,
    pub api_server_port: Mutex<Option<u16>>,
}

impl Default for AppState {
    fn default() -> Self {
        let xdg = XdgDirs::resolve().unwrap_or_else(|error| {
            panic!("Failed to resolve XDG directories: {error}");
        });

        Self {
            locked_project: Mutex::new(None),
            db: Mutex::new(None),
            codebase_snapshot: Mutex::new(None),
            pty_manager: PTYManager::new(),
            remote: tokio::sync::Mutex::new(None),
            mcp_dir: std::env::temp_dir().join(format!("ralph-mcp-{}", std::process::id())),
            xdg,
            api_server_port: Mutex::new(None),
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

pub(super) struct ProjectSessionService<'a> {
    app_state: &'a AppState,
}

impl<'a> ProjectSessionService<'a> {
    pub(super) fn new(app_state: &'a AppState) -> Self {
        Self { app_state }
    }

    pub(super) fn with_db<T, F>(&self, f: F) -> RalphResult<T>
    where
        F: FnOnce(&SqliteDb) -> RalphResult<T>,
    {
        let guard = self
            .app_state
            .db
            .lock()
            .ralph_err(codes::INTERNAL, "Database mutex poisoned")?;
        let db = guard.as_ref().ok_or_else(|| {
            err_string(codes::PROJECT_LOCK, "No project locked (database not open)")
        })?;
        f(db)
    }

    pub(super) fn with_db_tx<T, F>(&self, f: F) -> RalphResult<T>
    where
        F: FnOnce(&SqliteDb) -> RalphResult<T>,
    {
        self.with_db(|db| TransactionService::new(db).run(f))
    }

    pub(super) fn locked_project_path(&self) -> RalphResult<PathBuf> {
        let locked = self
            .app_state
            .locked_project
            .lock()
            .ralph_err(codes::INTERNAL, "Locked project mutex poisoned")?;
        locked
            .as_ref()
            .cloned()
            .ok_or_else(|| err_string(codes::PROJECT_LOCK, "No project locked"))
    }
}

pub(super) struct TransactionService<'a> {
    db: &'a SqliteDb,
}

impl<'a> TransactionService<'a> {
    pub(super) fn new(db: &'a SqliteDb) -> Self {
        Self { db }
    }

    pub(super) fn run<T, F>(&self, f: F) -> RalphResult<T>
    where
        F: FnOnce(&SqliteDb) -> RalphResult<T>,
    {
        self.db.with_transaction(f)
    }
}

pub(crate) struct CommandContext<'a> {
    session: ProjectSessionService<'a>,
}

impl<'a> CommandContext<'a> {
    pub(crate) fn new(app_state: &'a AppState) -> Self {
        Self {
            session: ProjectSessionService::new(app_state),
        }
    }

    pub(crate) fn from_tauri_state(state: &'a State<'_, AppState>) -> Self {
        Self::new(state.inner())
    }

    pub(crate) fn db<T, F>(&self, f: F) -> RalphResult<T>
    where
        F: FnOnce(&SqliteDb) -> RalphResult<T>,
    {
        self.session.with_db(f)
    }

    pub(crate) fn db_tx<T, F>(&self, f: F) -> RalphResult<T>
    where
        F: FnOnce(&SqliteDb) -> RalphResult<T>,
    {
        self.session.with_db_tx(f)
    }

    pub(crate) fn locked_project_path(&self) -> RalphResult<PathBuf> {
        self.session.locked_project_path()
    }
}

impl AppState {
    pub async fn remote_rpc_client(&self) -> RalphResult<Option<crate::remote::RemoteRpcClient>> {
        remote_rpc_client_from_transport(&self.remote, false).await
    }

    #[allow(dead_code)]
    pub async fn remote_rpc_client_required(&self) -> RalphResult<crate::remote::RemoteRpcClient> {
        remote_rpc_client_required(self.remote_rpc_client().await?)
    }
}

#[allow(dead_code)]
pub(crate) fn with_db<T, F>(state: &State<'_, AppState>, f: F) -> RalphResult<T>
where
    F: FnOnce(&SqliteDb) -> RalphResult<T>,
{
    CommandContext::from_tauri_state(state).db(f)
}

#[allow(dead_code)]
pub(crate) fn with_db_tx<T, F>(state: &State<'_, AppState>, f: F) -> RalphResult<T>
where
    F: FnOnce(&SqliteDb) -> RalphResult<T>,
{
    CommandContext::from_tauri_state(state).db_tx(f)
}

#[allow(dead_code)]
pub(crate) fn get_locked_project_path(state: &State<'_, AppState>) -> RalphResult<PathBuf> {
    CommandContext::from_tauri_state(state).locked_project_path()
}
