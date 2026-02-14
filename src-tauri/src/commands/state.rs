use crate::diagnostics;
use crate::terminal::PTYManager;
use crate::xdg::XdgDirs;
use prompt_builder::{CodebaseSnapshot, PromptContext};
use ralph_errors::{codes, RalphResultExt, ToStringErr};
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

    pub(super) fn maybe_locked_project_path(&self) -> Result<Option<PathBuf>, String> {
        let locked = self
            .app_state
            .locked_project
            .lock()
            .err_str(codes::INTERNAL)?;
        Ok(locked.as_ref().cloned())
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

    pub(super) fn maybe_locked_project_path(&self) -> Result<Option<PathBuf>, String> {
        self.session.maybe_locked_project_path()
    }
}

impl AppState {
    pub(super) fn build_prompt_context(
        &self,
        project_path: &std::path::Path,
        user_input: Option<String>,
        instruction_overrides: std::collections::HashMap<String, String>,
        target_task_id: Option<u32>,
    ) -> Result<PromptContext, String> {
        let ralph_dir = project_path.join(".ralph");
        let db_path = ralph_dir.join("db").join("ralph.db");

        let db_guard = self.db.lock().err_str(codes::INTERNAL)?;
        let db = db_guard.as_ref().ok_or_else(|| {
            ralph_errors::err_string(codes::PROJECT_LOCK, "No project locked (database not open)")
        })?;

        let snapshot = {
            let mut snap_guard = self.codebase_snapshot.lock().err_str(codes::INTERNAL)?;
            if snap_guard.is_none() {
                *snap_guard = Some(prompt_builder::snapshot::analyze(project_path));
            }
            snap_guard.clone()
        };

        let api_port = *self.api_server_port.lock().err_str(codes::INTERNAL)?;

        Ok(PromptContext {
            features: db.get_subsystems(),
            tasks: db.get_tasks(),
            disciplines: db.get_disciplines(),
            metadata: db.get_project_info(),
            file_contents: std::collections::HashMap::new(),
            progress_txt: None,
            learnings_txt: None,
            claude_ralph_md: None,
            project_path: project_path.to_string_lossy().to_string(),
            db_path: db_path.to_string_lossy().to_string(),
            script_dir: self.mcp_dir.to_string_lossy().to_string(),
            api_server_port: api_port,
            user_input,
            target_task_id,
            target_feature: None,
            codebase_snapshot: snapshot,
            instruction_overrides,
            relevant_comments: None,
        })
    }

    pub(super) fn generate_mcp_config(
        &self,
        mode: &str,
        project_path: &std::path::Path,
    ) -> Result<PathBuf, String> {
        let prompt_type = match mode {
            "task_creation" => prompt_builder::PromptType::Braindump,
            _ => prompt_builder::PromptType::Discuss,
        };

        let mut overrides = std::collections::HashMap::new();
        let override_path = project_path
            .join(".ralph")
            .join("prompts")
            .join(format!("{mode}_instructions.md"));
        if let Ok(text) = std::fs::read_to_string(&override_path) {
            let section_name = format!("{mode}_instructions");
            overrides.insert(section_name, text);
        }

        let recipe = prompt_builder::recipes::get(prompt_type);
        let ctx = self.build_prompt_context(project_path, None, overrides, None)?;

        let (scripts, config_json) =
            prompt_builder::mcp::generate(&ctx, recipe.mcp_mode, &recipe.mcp_tools);

        self.write_mcp_artifacts(&scripts, &config_json, format!("mcp-{mode}.json"))
    }

    pub(super) fn generate_mcp_config_for_task(
        &self,
        task_id: u32,
        project_path: &std::path::Path,
    ) -> Result<PathBuf, String> {
        let ctx = self.build_prompt_context(
            project_path,
            None,
            std::collections::HashMap::new(),
            Some(task_id),
        )?;

        let recipe = prompt_builder::recipes::get(prompt_builder::PromptType::TaskExecution);
        let (scripts, config_json) =
            prompt_builder::mcp::generate(&ctx, recipe.mcp_mode, &recipe.mcp_tools);

        self.write_mcp_artifacts(&scripts, &config_json, format!("mcp-task-{task_id}.json"))
    }

    fn write_mcp_artifacts(
        &self,
        scripts: &[prompt_builder::McpScript],
        config_json: &str,
        config_filename: String,
    ) -> Result<PathBuf, String> {
        std::fs::create_dir_all(&self.mcp_dir)
            .ralph_err(codes::FILESYSTEM, "Failed to create MCP dir")?;

        for script in scripts {
            let script_path = self.mcp_dir.join(&script.filename);
            std::fs::write(&script_path, &script.content)
                .ralph_err(codes::FILESYSTEM, "Failed to write MCP script")?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755))
                    .ralph_err(codes::FILESYSTEM, "Failed to chmod MCP script")?;
            }
        }

        let config_path = self.mcp_dir.join(config_filename);
        std::fs::write(&config_path, config_json)
            .ralph_err(codes::FILESYSTEM, "Failed to write MCP config")?;

        Ok(config_path)
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
