use crate::diagnostics;
use crate::terminal::PTYManager;
use crate::xdg::XdgDirs;
use prompt_builder::{CodebaseSnapshot, PromptContext};
use ralph_errors::{codes, err_string, ToStringErr};
use sqlite_db::SqliteDb;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub locked_project: Mutex<Option<PathBuf>>,
    pub db: Mutex<Option<SqliteDb>>,
    pub codebase_snapshot: Mutex<Option<CodebaseSnapshot>>,
    pub pty_manager: PTYManager,
    pub remote: tokio::sync::Mutex<Option<crate::remote::RemoteWireFrameConnection>>,
    pub(super) mcp_dir: PathBuf,
    pub xdg: XdgDirs,
    pub api_server_port: Mutex<Option<u16>>,
}

impl Default for AppState {
    fn default() -> Self {
        let xdg = match XdgDirs::resolve() {
            Ok(xdg) => xdg,
            Err(error) => {
                let message = format!(
                    "Failed to resolve XDG directories: {error}. Using fallback temp directories."
                );
                diagnostics::emit_warning("app-state", "xdg-resolve-fallback", &message);
                tracing::warn!("{message}");
                XdgDirs::fallback()
            }
        };

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
        let _ = std::fs::remove_dir_all(&self.mcp_dir);
    }
}

pub(super) struct ProjectSessionService<'a> {
    app_state: &'a AppState,
}

impl<'a> ProjectSessionService<'a> {
    pub(super) fn new(app_state: &'a AppState) -> Self {
        Self { app_state }
    }

    pub(super) fn with_db<T, F>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&SqliteDb) -> Result<T, String>,
    {
        let guard = self.app_state.db.lock().err_str(codes::INTERNAL)?;
        let db = guard.as_ref().ok_or_else(|| {
            ralph_errors::err_string(codes::PROJECT_LOCK, "No project locked (database not open)")
        })?;
        f(db)
    }

    pub(super) fn with_db_tx<T, F>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&SqliteDb) -> Result<T, String>,
    {
        self.with_db(|db| TransactionService::new(db).run(f))
    }

    pub(super) fn locked_project_path(&self) -> Result<PathBuf, String> {
        let locked = self
            .app_state
            .locked_project
            .lock()
            .err_str(codes::INTERNAL)?;
        locked
            .as_ref()
            .cloned()
            .ok_or_else(|| ralph_errors::err_string(codes::PROJECT_LOCK, "No project locked"))
    }
}

pub(super) struct TransactionService<'a> {
    db: &'a SqliteDb,
}

impl<'a> TransactionService<'a> {
    pub(super) fn new(db: &'a SqliteDb) -> Self {
        Self { db }
    }

    pub(super) fn run<T, F>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&SqliteDb) -> Result<T, String>,
    {
        self.db.with_transaction(f)
    }
}

pub(super) struct CommandContext<'a> {
    session: ProjectSessionService<'a>,
}

impl<'a> CommandContext<'a> {
    pub(super) fn new(app_state: &'a AppState) -> Self {
        Self {
            session: ProjectSessionService::new(app_state),
        }
    }

    pub(super) fn from_tauri_state(state: &'a State<'_, AppState>) -> Self {
        Self::new(state.inner())
    }

    pub(super) fn db<T, F>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&SqliteDb) -> Result<T, String>,
    {
        self.session.with_db(f)
    }

    pub(super) fn db_tx<T, F>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&SqliteDb) -> Result<T, String>,
    {
        self.session.with_db_tx(f)
    }

    pub(super) fn locked_project_path(&self) -> Result<PathBuf, String> {
        self.session.locked_project_path()
    }
}

impl AppState {
    pub async fn remote_rpc_client(
        &self,
    ) -> Result<Option<crate::remote::RemoteRpcClient>, String> {
        let guard = self.remote.lock().await;

        guard.as_ref().map_or(Ok(None), |conn| {
            if conn.is_connected() {
                Ok(Some(conn.rpc_client()))
            } else {
                Err(err_string(
                    codes::INTERNAL,
                    format!(
                        "Remote transport disconnected (wsUrl='{}'). Reconnect.",
                        conn.ws_url()
                    ),
                ))
            }
        })
    }

    pub(super) fn build_prompt_context(
        &self,
        project_path: &std::path::Path,
        user_input: Option<String>,
        instruction_overrides: std::collections::HashMap<String, String>,
        target_task_id: Option<u32>,
    ) -> Result<PromptContext, String> {
        let db_guard = self.db.lock().err_str(codes::INTERNAL)?;
        let db = db_guard.as_ref().ok_or_else(|| {
            ralph_errors::err_string(codes::PROJECT_LOCK, "No project locked (database not open)")
        })?;

        let api_port = *self.api_server_port.lock().err_str(codes::INTERNAL)?;

        ralph_backend::prompt_context::build_prompt_context(
            ralph_backend::prompt_context::PromptContextArgs {
                db,
                project_path,
                mcp_dir: &self.mcp_dir,
                codebase_snapshot: &self.codebase_snapshot,
                api_server_port: api_port,
                user_input,
                instruction_overrides,
                target_task_id,
            },
        )
    }

    pub(super) fn generate_mcp_config(
        &self,
        mode: &str,
        project_path: &std::path::Path,
    ) -> Result<PathBuf, String> {
        let api_port = *self.api_server_port.lock().err_str(codes::INTERNAL)?;
        ralph_backend::mcp::generate_mcp_config(
            &self.db,
            &self.codebase_snapshot,
            &self.mcp_dir,
            api_port,
            mode,
            project_path,
        )
    }

    pub(super) fn generate_mcp_config_for_task(
        &self,
        task_id: u32,
        project_path: &std::path::Path,
    ) -> Result<PathBuf, String> {
        let api_port = *self.api_server_port.lock().err_str(codes::INTERNAL)?;
        ralph_backend::mcp::generate_mcp_config_for_task(
            &self.db,
            &self.codebase_snapshot,
            &self.mcp_dir,
            api_port,
            task_id,
            project_path,
        )
    }
}

#[allow(dead_code)]
pub(super) fn with_db<T, F>(state: &State<'_, AppState>, f: F) -> Result<T, String>
where
    F: FnOnce(&SqliteDb) -> Result<T, String>,
{
    CommandContext::from_tauri_state(state).db(f)
}

#[allow(dead_code)]
pub(super) fn with_db_tx<T, F>(state: &State<'_, AppState>, f: F) -> Result<T, String>
where
    F: FnOnce(&SqliteDb) -> Result<T, String>,
{
    CommandContext::from_tauri_state(state).db_tx(f)
}

#[allow(dead_code)]
pub(super) fn get_locked_project_path(state: &State<'_, AppState>) -> Result<PathBuf, String> {
    CommandContext::from_tauri_state(state).locked_project_path()
}
