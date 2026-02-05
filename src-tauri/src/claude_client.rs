use crate::types::{ClaudeStreamEvent, RalphError};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub struct ClaudeClient;

pub enum ClaudeOutput {
    Text(String),
    RateLimited,
    Complete,
    Error(String),
}

impl ClaudeClient {
    pub fn run_iteration(
        project_path: &Path,
        prompt: &str,
        model: &str,
        timeout_secs: u64,
        output_tx: mpsc::Sender<ClaudeOutput>,
    ) -> Result<String, RalphError> {
        let mut full_output = String::new();
        let mut rate_limited = false;

        let mut child = Command::new("timeout")
            .arg(format!("{}s", timeout_secs))
            .arg("claude")
            .arg("--model")
            .arg(model)
            .arg("--output-format")
            .arg("stream-json")
            .arg("--max-turns")
            .arg("50")
            .arg("--dangerously-skip-permissions")
            .arg("-p")
            .arg(prompt)
            .current_dir(project_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                RalphError::ClaudeProcessError(format!("Failed to spawn claude: {}", e))
            })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            RalphError::ClaudeProcessError("Failed to capture stdout".to_string())
        })?;

        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    let _ = output_tx.send(ClaudeOutput::Error(format!("Read error: {}", e)));
                    continue;
                }
            };

            if line.trim().is_empty() {
                continue;
            }

            if let Ok(event) = serde_json::from_str::<ClaudeStreamEvent>(&line) {
                match event.event_type.as_str() {
                    "assistant" => {
                        if let Some(content) = event.content {
                            full_output.push_str(&content);
                            let _ = output_tx.send(ClaudeOutput::Text(content));
                        }
                    }
                    "content_block_delta" => {
                        if let Some(content) = event.content {
                            full_output.push_str(&content);
                            let _ = output_tx.send(ClaudeOutput::Text(content));
                        }
                    }
                    "error" => {
                        if let Some(err) = event.error {
                            if err.error_type == "overloaded_error"
                                || err.error_type == "rate_limit_error"
                            {
                                rate_limited = true;
                                let _ = output_tx.send(ClaudeOutput::RateLimited);
                            } else {
                                let _ = output_tx.send(ClaudeOutput::Error(err.message));
                            }
                        }
                    }
                    "result" => {
                        // Final result event
                    }
                    _ => {}
                }
            } else if line.contains("rate") && line.contains("limit") {
                // Fallback text-based detection
                rate_limited = true;
                let _ = output_tx.send(ClaudeOutput::RateLimited);
            }
        }

        let status = child
            .wait()
            .map_err(|e| RalphError::ClaudeProcessError(format!("Wait failed: {}", e)))?;

        if rate_limited {
            return Err(RalphError::ClaudeProcessError("Rate limited".to_string()));
        }

        if !status.success() {
            let code = status.code().unwrap_or(-1);
            if code == 124 {
                return Err(RalphError::ClaudeProcessError(
                    "Iteration timed out".to_string(),
                ));
            }
            return Err(RalphError::ClaudeProcessError(format!(
                "Claude exited with code {}",
                code
            )));
        }

        let _ = output_tx.send(ClaudeOutput::Complete);
        Ok(full_output)
    }

    pub fn run_iteration_async(
        project_path: &Path,
        prompt: String,
        model: String,
        timeout_secs: u64,
        current_pid: Arc<Mutex<Option<u32>>>,
    ) -> (
        mpsc::Receiver<ClaudeOutput>,
        thread::JoinHandle<Result<String, RalphError>>,
    ) {
        let (tx, rx) = mpsc::channel();
        let path = project_path.to_path_buf();

        let handle = thread::spawn(move || {
            Self::run_iteration_with_pid(&path, &prompt, &model, timeout_secs, tx, current_pid)
        });

        (rx, handle)
    }

    fn run_iteration_with_pid(
        project_path: &Path,
        prompt: &str,
        model: &str,
        timeout_secs: u64,
        output_tx: mpsc::Sender<ClaudeOutput>,
        current_pid: Arc<Mutex<Option<u32>>>,
    ) -> Result<String, RalphError> {
        let mut full_output = String::new();
        let mut rate_limited = false;

        let mut child = Command::new("timeout")
            .arg(format!("{}s", timeout_secs))
            .arg("claude")
            .arg("--model")
            .arg(model)
            .arg("--output-format")
            .arg("stream-json")
            .arg("--max-turns")
            .arg("50")
            .arg("--dangerously-skip-permissions")
            .arg("-p")
            .arg(prompt)
            .current_dir(project_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                RalphError::ClaudeProcessError(format!("Failed to spawn claude: {}", e))
            })?;

        // Store the PID so it can be killed if needed
        *current_pid.lock().unwrap() = Some(child.id());

        let stdout = child.stdout.take().ok_or_else(|| {
            RalphError::ClaudeProcessError("Failed to capture stdout".to_string())
        })?;

        let reader = BufReader::new(stdout);

        for line in reader.lines().map_while(Result::ok) {
            if let Ok(event) = serde_json::from_str::<ClaudeStreamEvent>(&line) {
                match event.event_type.as_str() {
                    "text_delta" => {
                        if let Some(content) = &event.content {
                            full_output.push_str(content);
                            let _ = output_tx.send(ClaudeOutput::Text(content.to_string()));
                        }
                    }
                    "error" => {
                        if let Some(err) = &event.error {
                            if err.error_type == "overloaded_error"
                                || err.error_type == "rate_limit_error"
                            {
                                rate_limited = true;
                                let _ = output_tx.send(ClaudeOutput::RateLimited);
                            }
                        }
                    }
                    "result" => {}
                    _ => {}
                }
            } else if line.contains("rate") && line.contains("limit") {
                rate_limited = true;
                let _ = output_tx.send(ClaudeOutput::RateLimited);
            }
        }

        let status = child
            .wait()
            .map_err(|e| RalphError::ClaudeProcessError(format!("Wait failed: {}", e)))?;

        // Clear the stored PID
        *current_pid.lock().unwrap() = None;

        if rate_limited {
            return Err(RalphError::ClaudeProcessError("Rate limited".to_string()));
        }

        if !status.success() {
            let code = status.code().unwrap_or(-1);
            if code == 124 {
                return Err(RalphError::ClaudeProcessError(
                    "Iteration timed out".to_string(),
                ));
            }
            if code == 143 || code == -15 {
                // SIGTERM - process was killed
                return Err(RalphError::ClaudeProcessError(
                    "Iteration stopped".to_string(),
                ));
            }
            return Err(RalphError::ClaudeProcessError(format!(
                "Claude exited with code {}",
                code
            )));
        }

        let _ = output_tx.send(ClaudeOutput::Complete);
        Ok(full_output)
    }
}
