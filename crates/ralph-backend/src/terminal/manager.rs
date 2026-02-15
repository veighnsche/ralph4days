use base64::{engine::general_purpose::STANDARD, Engine};
use portable_pty::{native_pty_system, PtySize};
use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::providers::resolve_agent_provider;
use super::session::{PTYSession, SessionConfig};
use super::{TerminalBridgeReplayOutputChunk, TerminalBridgeReplayOutputResult};
use ralph_contracts::terminal::{PtyClosedEvent, PtyOutputEvent};
use ralph_contracts::transport::EventSink;

use ralph_errors::{codes, RalphResultExt, ToStringErr};

const DEFAULT_REPLAY_BUFFER_BYTES: usize = 8 * 1024 * 1024;

fn preview_text(bytes: &[u8], max_chars: usize) -> String {
    let escaped = String::from_utf8_lossy(bytes).escape_debug().to_string();
    if escaped.chars().count() <= max_chars {
        return escaped;
    }
    let preview: String = escaped.chars().take(max_chars).collect();
    format!("{preview}…")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStreamMode {
    Live,
    Buffered,
}

impl SessionStreamMode {
    pub fn parse(mode: &str) -> Result<Self, String> {
        match mode.trim().to_ascii_lowercase().as_str() {
            "live" => Ok(Self::Live),
            "buffered" => Ok(Self::Buffered),
            _ => Err(ralph_errors::err_string(
                codes::TERMINAL,
                format!("Invalid stream mode: {mode}. Expected 'live' or 'buffered'"),
            )),
        }
    }
}

#[derive(Debug, Clone)]
struct BufferedOutputChunk {
    seq: u64,
    data: String,
    byte_len: usize,
}

#[derive(Debug)]
struct SessionStreamState {
    mode: SessionStreamMode,
    next_seq: u64,
    dropped_until_seq: u64,
    buffered_bytes: usize,
    chunks: VecDeque<BufferedOutputChunk>,
}

impl SessionStreamState {
    fn new() -> Self {
        Self {
            mode: SessionStreamMode::Live,
            next_seq: 1,
            dropped_until_seq: 0,
            buffered_bytes: 0,
            chunks: VecDeque::new(),
        }
    }

    fn append_chunk(&mut self, data: String, byte_len: usize, max_buffer_bytes: usize) -> u64 {
        let seq = self.next_seq;
        self.next_seq += 1;

        self.chunks.push_back(BufferedOutputChunk {
            seq,
            data,
            byte_len,
        });
        self.buffered_bytes += byte_len;

        while self.buffered_bytes > max_buffer_bytes {
            if let Some(removed) = self.chunks.pop_front() {
                self.buffered_bytes = self.buffered_bytes.saturating_sub(removed.byte_len);
                self.dropped_until_seq = self.dropped_until_seq.max(removed.seq);
            } else {
                break;
            }
        }

        seq
    }

    fn replay_after(&self, after_seq: u64, limit: usize) -> TerminalBridgeReplayOutputResult {
        let capped_limit = limit.max(1);
        let truncated = after_seq < self.dropped_until_seq;
        let cursor = after_seq.max(self.dropped_until_seq);

        let chunks: Vec<TerminalBridgeReplayOutputChunk> = self
            .chunks
            .iter()
            .filter(|chunk| chunk.seq > cursor)
            .take(capped_limit)
            .map(|chunk| TerminalBridgeReplayOutputChunk {
                seq: chunk.seq,
                data: chunk.data.clone(),
            })
            .collect();

        let last_seq = chunks.last().map_or(cursor, |chunk| chunk.seq);
        let has_more = self.chunks.iter().any(|chunk| chunk.seq > last_seq);

        TerminalBridgeReplayOutputResult {
            chunks,
            has_more,
            truncated,
            truncated_until_seq: truncated.then_some(self.dropped_until_seq),
        }
    }
}

struct ManagedSession {
    pty: PTYSession,
    stream: SessionStreamState,
}

pub struct PTYManager {
    sessions: Arc<Mutex<HashMap<String, ManagedSession>>>,
    replay_buffer_bytes: usize,
}

impl Default for PTYManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PTYManager {
    pub fn new() -> Self {
        Self::new_with_replay_buffer_bytes(DEFAULT_REPLAY_BUFFER_BYTES)
    }

    fn new_with_replay_buffer_bytes(replay_buffer_bytes: usize) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            replay_buffer_bytes,
        }
    }

    #[tracing::instrument(skip(self, sink), fields(session_id = %session_id))]
    pub fn create_session(
        &self,
        sink: Arc<dyn EventSink>,
        session_id: String,
        working_dir: &Path,
        mcp_config: Option<PathBuf>,
        config: SessionConfig,
    ) -> Result<(), String> {
        tracing::info!(
            working_dir = %working_dir.display(),
            agent = ?config.agent,
            model = ?config.model,
            has_mcp = mcp_config.is_some(),
            "Creating PTY session"
        );

        {
            let sessions = self.sessions.lock().err_str(codes::INTERNAL)?;
            if sessions.contains_key(&session_id) {
                tracing::error!("PTY session already exists");
                return ralph_errors::ralph_err!(
                    codes::TERMINAL,
                    "PTY session already exists: {session_id}"
                );
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
            .ralph_err(codes::TERMINAL, "Failed to open PTY")?;

        tracing::debug!("PTY opened successfully");

        let provider = resolve_agent_provider(config.agent.as_deref());
        let cmd = provider.build_command(working_dir, mcp_config.as_deref(), &config);

        tracing::debug!(
            working_dir = %working_dir.display(),
            agent = provider.id(),
            model = ?config.model,
            "Spawning agent subprocess"
        );

        let child = pair
            .slave
            .spawn_command(cmd)
            .ralph_err(codes::TERMINAL, "Failed to spawn agent subprocess")?;

        tracing::info!(
            agent = provider.id(),
            "Agent subprocess spawned successfully"
        );

        let child = Arc::new(Mutex::new(child));

        let writer: Box<dyn Write + Send> = pair
            .master
            .take_writer()
            .ralph_err(codes::TERMINAL, "Failed to take PTY writer")?;
        let writer = Arc::new(Mutex::new(writer));

        let mut reader = pair
            .master
            .try_clone_reader()
            .ralph_err(codes::TERMINAL, "Failed to clone PTY reader")?;

        self.sessions.lock().err_str(codes::INTERNAL)?.insert(
            session_id.clone(),
            ManagedSession {
                pty: PTYSession {
                    writer: Arc::clone(&writer),
                    master: pair.master,
                    child: Arc::clone(&child),
                    reader_handle: None,
                },
                stream: SessionStreamState::new(),
            },
        );

        let sid = session_id.clone();
        let sink_clone = sink;
        let child_clone = Arc::clone(&child);
        let sessions_ref = Arc::clone(&self.sessions);
        let replay_buffer_bytes = self.replay_buffer_bytes;
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
                        tracing::debug!(
                            session_id = %sid,
                            bytes = n,
                            preview = %preview_text(&buf[..n], 220),
                            "terminal_bridge_output_chunk"
                        );

                        let encoded_data = STANDARD.encode(&buf[..n]);
                        let mut output_event: Option<PtyOutputEvent> = None;

                        if let Ok(mut sessions) = sessions_ref.lock() {
                            if let Some(session) = sessions.get_mut(&sid) {
                                let seq = session.stream.append_chunk(
                                    encoded_data.clone(),
                                    n,
                                    replay_buffer_bytes,
                                );

                                if session.stream.mode == SessionStreamMode::Live {
                                    output_event = Some(PtyOutputEvent {
                                        session_id: sid.clone(),
                                        seq,
                                        data: encoded_data,
                                    });
                                }
                            }
                        }

                        if let Some(event) = output_event {
                            if let Err(error) = sink_clone.emit_terminal_output(event) {
                                tracing::warn!(
                                    session_id = %sid,
                                    error = %error,
                                    "Failed to emit terminal output event"
                                );
                            }
                        }
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

            if let Err(error) = sink_clone.emit_terminal_closed(PtyClosedEvent {
                session_id: sid.clone(),
                exit_code,
            }) {
                tracing::warn!(
                    session_id = %sid,
                    error = %error,
                    "Failed to emit terminal closed event"
                );
            }

            if let Ok(mut sessions) = sessions_ref.lock() {
                sessions.remove(&sid);
            }
        });

        if let Ok(mut sessions) = self.sessions.lock().err_str(codes::INTERNAL) {
            if let Some(session) = sessions.get_mut(&session_id) {
                session.pty.reader_handle = Some(reader_handle);
            }
        }

        if let Some(preamble) = &config.post_start_preamble {
            tracing::debug!(
                session_id = %session_id,
                line_count = preamble.lines().filter(|line| !line.trim().is_empty()).count(),
                "Applying post-start preamble"
            );
            let mut guard = writer.lock().err_str(codes::INTERNAL)?;
            for line in preamble.lines().filter(|line| !line.trim().is_empty()) {
                guard
                    .write_all(line.as_bytes())
                    .ralph_err(codes::TERMINAL, "Failed to write preamble line to PTY")?;
                guard
                    .write_all(b"\r")
                    .ralph_err(codes::TERMINAL, "Failed to send preamble line terminator")?;
            }
        }

        tracing::info!(session_id, "PTY session created successfully");

        Ok(())
    }

    #[tracing::instrument(skip(self, data), fields(session_id, bytes = data.len()))]
    pub fn send_input(&self, session_id: &str, data: &[u8]) -> Result<(), String> {
        tracing::debug!(
            session_id,
            bytes = data.len(),
            preview = %preview_text(data, 140),
            "terminal_bridge_input_chunk"
        );
        let writer = {
            let sessions = self.sessions.lock().err_str(codes::INTERNAL)?;
            let session = sessions.get(session_id).ok_or_else(|| {
                ralph_errors::err_string(
                    codes::TERMINAL,
                    format!("No terminal bridge session: {session_id}"),
                )
            })?;
            Arc::clone(&session.pty.writer)
        };
        let mut guard = writer.lock().err_str(codes::INTERNAL)?;
        guard
            .write_all(data)
            .ralph_err(codes::TERMINAL, "Failed to write to PTY")?;

        tracing::trace!(session_id, bytes = data.len(), "Sent input to PTY");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn resize(&self, session_id: &str, cols: u16, rows: u16) -> Result<(), String> {
        let sessions = self.sessions.lock().err_str(codes::INTERNAL)?;
        let session = sessions.get(session_id).ok_or_else(|| {
            ralph_errors::err_string(
                codes::TERMINAL,
                format!("No terminal bridge session: {session_id}"),
            )
        })?;
        session
            .pty
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .ralph_err(codes::TERMINAL, "Failed to resize PTY")?;

        tracing::debug!(session_id, cols, rows, "PTY resized");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub fn terminate(&self, session_id: &str) -> Result<(), String> {
        let session = {
            let mut sessions = self.sessions.lock().err_str(codes::INTERNAL)?;
            sessions.remove(session_id)
        };

        if let Some(session) = session {
            if let Ok(mut child) = session.pty.child.lock() {
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

    pub fn set_stream_mode(&self, session_id: &str, mode: SessionStreamMode) -> Result<(), String> {
        let mut sessions = self.sessions.lock().err_str(codes::INTERNAL)?;
        let session = sessions.get_mut(session_id).ok_or_else(|| {
            ralph_errors::err_string(
                codes::TERMINAL,
                format!("No terminal bridge session: {session_id}"),
            )
        })?;
        session.stream.mode = mode;
        Ok(())
    }

    pub fn replay_output(
        &self,
        session_id: &str,
        after_seq: u64,
        limit: usize,
    ) -> Result<TerminalBridgeReplayOutputResult, String> {
        let sessions = self.sessions.lock().err_str(codes::INTERNAL)?;
        let session = sessions.get(session_id).ok_or_else(|| {
            ralph_errors::err_string(
                codes::TERMINAL,
                format!("No terminal bridge session: {session_id}"),
            )
        })?;
        Ok(session.stream.replay_after(after_seq, limit))
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

    #[test]
    fn test_send_input_fails_for_missing_session() {
        let manager = PTYManager::new();
        let err = manager.send_input("missing-session", b"hello").unwrap_err();
        assert!(err.contains("No terminal bridge session: missing-session"));
    }

    #[test]
    fn test_resize_fails_for_missing_session() {
        let manager = PTYManager::new();
        let err = manager.resize("missing-session", 80, 24).unwrap_err();
        assert!(err.contains("No terminal bridge session: missing-session"));
    }

    #[test]
    fn test_terminate_missing_session_is_ok() {
        let manager = PTYManager::new();
        assert!(manager.terminate("missing-session").is_ok());
    }

    #[test]
    fn test_parse_stream_mode() {
        assert_eq!(
            SessionStreamMode::parse("live").unwrap(),
            SessionStreamMode::Live
        );
        assert_eq!(
            SessionStreamMode::parse("buffered").unwrap(),
            SessionStreamMode::Buffered
        );
        assert!(SessionStreamMode::parse("wat").is_err());
    }

    #[test]
    fn test_stream_buffer_eviction_marks_truncation() {
        let mut stream = SessionStreamState::new();
        stream.append_chunk("QQ==".to_owned(), 1, 4);
        stream.append_chunk("Qg==".to_owned(), 1, 4);
        stream.append_chunk("Qw==".to_owned(), 1, 4);
        stream.append_chunk("RA==".to_owned(), 1, 4);
        stream.append_chunk("RQ==".to_owned(), 1, 4);

        let replay = stream.replay_after(0, 50);
        assert!(replay.truncated);
        assert_eq!(replay.truncated_until_seq, Some(1));
        assert_eq!(replay.chunks.first().map(|chunk| chunk.seq), Some(2));
    }

    #[test]
    fn test_stream_replay_limit_and_has_more() {
        let mut stream = SessionStreamState::new();
        stream.append_chunk("QQ==".to_owned(), 1, 1024);
        stream.append_chunk("Qg==".to_owned(), 1, 1024);
        stream.append_chunk("Qw==".to_owned(), 1, 1024);

        let replay = stream.replay_after(0, 2);
        assert_eq!(replay.chunks.len(), 2);
        assert_eq!(replay.chunks[0].seq, 1);
        assert_eq!(replay.chunks[1].seq, 2);
        assert!(replay.has_more);
        assert!(!replay.truncated);
    }
}
