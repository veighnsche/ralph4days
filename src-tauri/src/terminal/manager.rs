use base64::{engine::general_purpose::STANDARD, Engine};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

use super::events::{PtyClosedEvent, PtyOutputEvent};
use super::session::{build_settings_json, PTYSession, SessionConfig};

use crate::errors::codes;

trait ToStringErr<T> {
    fn err_str(self) -> Result<T, String>;
}

impl<T, E: std::fmt::Display> ToStringErr<T> for Result<T, E> {
    fn err_str(self) -> Result<T, String> {
        self.map_err(|e| {
            crate::errors::RalphError {
                code: codes::INTERNAL,
                message: e.to_string(),
            }
            .to_string()
        })
    }
}

pub struct PTYManager {
    sessions: Arc<Mutex<HashMap<String, PTYSession>>>,
}

impl Default for PTYManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PTYManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[tracing::instrument(skip(self, app), fields(session_id = %session_id))]
    pub fn create_session(
        &self,
        app: AppHandle,
        session_id: String,
        working_dir: &Path,
        mcp_config: Option<PathBuf>,
        config: SessionConfig,
    ) -> Result<(), String> {
        tracing::info!(
            working_dir = %working_dir.display(),
            model = ?config.model,
            has_mcp = mcp_config.is_some(),
            "Creating PTY session"
        );

        {
            let sessions = self.sessions.lock().err_str()?;
            if sessions.contains_key(&session_id) {
                tracing::error!("PTY session already exists");
                return Err(crate::errors::RalphError {
                    code: codes::TERMINAL,
                    message: format!("PTY session already exists: {session_id}"),
                }
                .to_string());
            }
        }

        let pty_system = native_pty_system();

        // Start with standard terminal size - will be immediately resized by frontend after terminal fits
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to open PTY");
                crate::errors::RalphError {
                    code: codes::TERMINAL,
                    message: format!("Failed to open PTY: {e}"),
                }
                .to_string()
            })?;

        tracing::debug!("PTY opened successfully");

        let mut cmd = CommandBuilder::new("claude");
        cmd.cwd(working_dir);

        cmd.args(["--permission-mode", "bypassPermissions"]);
        cmd.arg("--verbose");
        cmd.arg("--no-chrome");

        if let Some(model) = &config.model {
            cmd.args(["--model", model]);
        }

        let settings_json = build_settings_json(&config);
        cmd.args(["--settings", &settings_json]);

        if let Some(mcp_config) = mcp_config {
            cmd.args(["--mcp-config", &mcp_config.to_string_lossy()]);
        }

        tracing::debug!(
            working_dir = %working_dir.display(),
            model = ?config.model,
            "Spawning Claude CLI subprocess"
        );

        let child = pair.slave.spawn_command(cmd).map_err(|e| {
            tracing::error!(error = %e, "Failed to spawn Claude CLI");
            crate::errors::RalphError {
                code: codes::TERMINAL,
                message: format!("Failed to spawn claude: {e}"),
            }
            .to_string()
        })?;

        tracing::info!("Claude CLI subprocess spawned successfully");

        let child = Arc::new(Mutex::new(child));

        let writer: Box<dyn Write + Send> = pair.master.take_writer().map_err(|e| {
            crate::errors::RalphError {
                code: codes::TERMINAL,
                message: format!("Failed to take PTY writer: {e}"),
            }
            .to_string()
        })?;
        let writer = Arc::new(Mutex::new(writer));

        let mut reader = pair.master.try_clone_reader().map_err(|e| {
            crate::errors::RalphError {
                code: codes::TERMINAL,
                message: format!("Failed to clone PTY reader: {e}"),
            }
            .to_string()
        })?;

        let sid = session_id.clone();
        let app_clone = app;
        let child_clone = Arc::clone(&child);
        let sessions_ref = Arc::clone(&self.sessions);
        let reader_handle = std::thread::spawn(move || {
            tracing::debug!(session_id = %sid, "PTY reader thread started");
            let mut buf = [0u8; 4096];
            let mut total_bytes = 0u64;
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        tracing::debug!(session_id = %sid, "PTY reader reached EOF");
                        break;
                    }
                    Err(e) => {
                        tracing::warn!(session_id = %sid, error = %e, "PTY read error");
                        break;
                    }
                    Ok(n) => {
                        total_bytes += n as u64;
                        tracing::trace!(session_id = %sid, bytes = n, total_bytes, "PTY output");
                        let _ = app_clone.emit(
                            "ralph://pty_output",
                            PtyOutputEvent {
                                session_id: sid.clone(),
                                data: STANDARD.encode(&buf[..n]),
                            },
                        );
                    }
                }
            }

            let exit_code = child_clone
                .lock()
                .ok()
                .and_then(|mut c| c.wait().ok())
                .map_or(1, |s| s.exit_code());

            tracing::info!(
                session_id = %sid,
                exit_code,
                total_bytes,
                "PTY session closed"
            );

            let _ = app_clone.emit(
                "ralph://pty_closed",
                PtyClosedEvent {
                    session_id: sid.clone(),
                    exit_code,
                },
            );

            if let Ok(mut sessions) = sessions_ref.lock() {
                sessions.remove(&sid);
            }
        });

        let session = PTYSession {
            writer,
            master: pair.master,
            child,
            _reader_handle: Some(reader_handle),
        };

        self.sessions
            .lock()
            .err_str()?
            .insert(session_id.clone(), session);

        tracing::info!(session_id, "PTY session created successfully");

        Ok(())
    }

    #[tracing::instrument(skip(self, data), fields(session_id, bytes = data.len()))]
    pub fn send_input(&self, session_id: &str, data: &[u8]) -> Result<(), String> {
        let writer = {
            let sessions = self.sessions.lock().err_str()?;
            let session = sessions.get(session_id).ok_or_else(|| {
                crate::errors::RalphError {
                    code: codes::TERMINAL,
                    message: format!("No PTY session: {session_id}"),
                }
                .to_string()
            })?;
            Arc::clone(&session.writer)
        };
        let mut guard = writer.lock().err_str()?;
        guard.write_all(data).map_err(|e| {
            tracing::error!(session_id, error = %e, "Failed to write to PTY");
            crate::errors::RalphError {
                code: codes::TERMINAL,
                message: format!("Failed to write to PTY: {e}"),
            }
            .to_string()
        })?;

        tracing::trace!(session_id, bytes = data.len(), "Sent input to PTY");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn resize(&self, session_id: &str, cols: u16, rows: u16) -> Result<(), String> {
        let sessions = self.sessions.lock().err_str()?;
        let session = sessions.get(session_id).ok_or_else(|| {
            crate::errors::RalphError {
                code: codes::TERMINAL,
                message: format!("No PTY session: {session_id}"),
            }
            .to_string()
        })?;
        session
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| {
                tracing::error!(session_id, error = %e, cols, rows, "Failed to resize PTY");
                crate::errors::RalphError {
                    code: codes::TERMINAL,
                    message: format!("Failed to resize PTY: {e}"),
                }
                .to_string()
            })?;

        tracing::debug!(session_id, cols, rows, "PTY resized");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn terminate(&self, session_id: &str) -> Result<(), String> {
        let session = {
            let mut sessions = self.sessions.lock().err_str()?;
            sessions.remove(session_id)
        };

        if let Some(session) = session {
            if let Ok(mut child) = session.child.lock() {
                let _ = child.kill();
                tracing::info!(session_id, "PTY session terminated (killed)");
            }
        } else {
            tracing::warn!(
                session_id,
                "Attempted to terminate non-existent PTY session"
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pty_manager_new() {
        let manager = PTYManager::new();
        let sessions = manager.sessions.lock().unwrap();
        assert_eq!(sessions.len(), 0);
    }

    #[test]
    fn test_pty_manager_default() {
        let manager = PTYManager::default();
        let sessions = manager.sessions.lock().unwrap();
        assert_eq!(sessions.len(), 0);
    }
}
