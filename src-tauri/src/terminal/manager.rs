use base64::{engine::general_purpose::STANDARD, Engine};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

use super::events::{PtyClosedEvent, PtyOutputEvent};
use super::session::{build_settings_json, PTYSession, SessionConfig};

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

    pub fn create_session(
        &self,
        app: AppHandle,
        session_id: String,
        working_dir: &Path,
        mcp_config: Option<PathBuf>,
        config: SessionConfig,
    ) -> Result<(), String> {
        // Reject duplicate session IDs to prevent leaking the old PTY process
        {
            let sessions = self.sessions.lock().map_err(|e| e.to_string())?;
            if sessions.contains_key(&session_id) {
                return Err(format!("PTY session already exists: {}", session_id));
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
            .map_err(|e| format!("Failed to open PTY: {}", e))?;

        let mut cmd = CommandBuilder::new("claude");
        cmd.cwd(working_dir);
        // Interactive sessions — no -p, no --output-format stream-json

        // Fixed CLI flags
        cmd.args(["--permission-mode", "bypassPermissions"]);
        cmd.arg("--verbose");
        cmd.arg("--no-chrome");

        // Per-task model override (omit to use Claude Code's default)
        if let Some(model) = &config.model {
            cmd.args(["--model", model]);
        }

        // Settings JSON (fixed + per-task overrides)
        let settings_json = build_settings_json(&config);
        cmd.args(["--settings", &settings_json]);

        if let Some(mcp_config) = mcp_config {
            cmd.args(["--mcp-config", &mcp_config.to_string_lossy()]);
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn claude: {}", e))?;

        let child = Arc::new(Mutex::new(child));

        let writer: Box<dyn Write + Send> = pair
            .master
            .take_writer()
            .map_err(|e| format!("Failed to take PTY writer: {}", e))?;
        let writer = Arc::new(Mutex::new(writer));

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("Failed to clone PTY reader: {}", e))?;

        // Spawn reader thread to forward PTY output to frontend
        let sid = session_id.clone();
        let app_clone = app.clone();
        let child_clone = Arc::clone(&child);
        let sessions_ref = Arc::clone(&self.sessions);
        let reader_handle = std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let _ = app_clone.emit(
                            "ralph://pty_output",
                            PtyOutputEvent {
                                session_id: sid.clone(),
                                data: STANDARD.encode(&buf[..n]),
                            },
                        );
                    }
                    Err(_) => break,
                }
            }

            // Process ended — get exit code
            let exit_code = child_clone
                .lock()
                .ok()
                .and_then(|mut c| c.wait().ok())
                .map(|s| s.exit_code())
                .unwrap_or(1);

            let _ = app_clone.emit(
                "ralph://pty_closed",
                PtyClosedEvent {
                    session_id: sid.clone(),
                    exit_code,
                },
            );

            // Cleanup session from map
            if let Ok(mut sessions) = sessions_ref.lock() {
                sessions.remove(&sid);
            }
        });

        let session = PTYSession {
            writer,
            master: pair.master,
            child,
            reader_handle: Some(reader_handle),
        };

        self.sessions
            .lock()
            .map_err(|e| e.to_string())?
            .insert(session_id, session);

        Ok(())
    }

    pub fn send_input(&self, session_id: &str, data: &[u8]) -> Result<(), String> {
        let writer = {
            let sessions = self.sessions.lock().map_err(|e| e.to_string())?;
            let session = sessions
                .get(session_id)
                .ok_or_else(|| format!("No PTY session: {}", session_id))?;
            Arc::clone(&session.writer)
        };
        let mut guard = writer.lock().map_err(|e| e.to_string())?;
        guard
            .write_all(data)
            .map_err(|e| format!("Failed to write to PTY: {}", e))
    }

    pub fn resize(&self, session_id: &str, cols: u16, rows: u16) -> Result<(), String> {
        let sessions = self.sessions.lock().map_err(|e| e.to_string())?;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| format!("No PTY session: {}", session_id))?;
        session
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to resize PTY: {}", e))
    }

    pub fn terminate(&self, session_id: &str) -> Result<(), String> {
        // Remove session from map first, then kill outside the lock
        let session = {
            let mut sessions = self.sessions.lock().map_err(|e| e.to_string())?;
            sessions.remove(session_id)
        };

        if let Some(session) = session {
            if let Ok(mut child) = session.child.lock() {
                let _ = child.kill();
            }
            // reader thread will exit on EOF from killed process and clean up
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

    // Note: Full integration tests for create_session, send_input, resize, and terminate
    // require mocking Tauri AppHandle and spawning actual PTY processes.
    // These should be covered by Tauri WebDriver E2E tests (see .docs/007_TESTING_STRATEGY.md)
}
