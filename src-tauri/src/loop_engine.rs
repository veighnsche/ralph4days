use crate::claude_client::{ClaudeClient, ClaudeOutput};
use crate::prompt_builder::{hash_content, PromptBuilder};
use crate::types::{LoopConfig, LoopState, LoopStatus, RalphError, RalphEvent};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub struct LoopEngine {
    status: Arc<Mutex<LoopStatus>>,
    config: Arc<Mutex<Option<LoopConfig>>>,
    pause_flag: Arc<Mutex<bool>>,
    stop_flag: Arc<Mutex<bool>>,
    current_pid: Arc<Mutex<Option<u32>>>,
}

impl Default for LoopEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LoopEngine {
    pub fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(LoopStatus::default())),
            config: Arc::new(Mutex::new(None)),
            pause_flag: Arc::new(Mutex::new(false)),
            stop_flag: Arc::new(Mutex::new(false)),
            current_pid: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_status(&self) -> LoopStatus {
        self.status.lock().unwrap().clone()
    }

    pub fn start(
        &self,
        app: AppHandle,
        project_path: PathBuf,
        max_iterations: u32,
    ) -> Result<(), RalphError> {
        {
            let status = self.status.lock().unwrap();
            if status.state == LoopState::Running {
                return Err(RalphError::AlreadyRunning);
            }
        }

        if !project_path.exists() {
            return Err(RalphError::ProjectNotFound(project_path));
        }

        if !project_path.join(".ralph").exists() {
            return Err(RalphError::MissingRalphDir);
        }

        let config = LoopConfig {
            project_path: project_path.clone(),
            max_iterations,
            ..Default::default()
        };

        *self.config.lock().unwrap() = Some(config.clone());
        *self.pause_flag.lock().unwrap() = false;
        *self.stop_flag.lock().unwrap() = false;

        {
            let mut status = self.status.lock().unwrap();
            status.state = LoopState::Running;
            status.current_iteration = 0;
            status.max_iterations = max_iterations;
            status.stagnant_count = 0;
            status.rate_limit_retries = 0;
            status.last_progress_hash = None;
            status.project_path = Some(project_path.clone());
        }

        self.emit_state_changed(&app);

        let status = Arc::clone(&self.status);
        let config_arc = Arc::clone(&self.config);
        let pause_flag = Arc::clone(&self.pause_flag);
        let stop_flag = Arc::clone(&self.stop_flag);
        let current_pid = Arc::clone(&self.current_pid);

        thread::spawn(move || {
            Self::run_loop(
                app,
                status,
                config_arc,
                pause_flag,
                stop_flag,
                current_pid,
                config,
            );
        });

        Ok(())
    }

    pub fn pause(&self, _app: &AppHandle) -> Result<(), RalphError> {
        let status = self.status.lock().unwrap();
        if status.state != LoopState::Running {
            return Err(RalphError::NotRunning);
        }
        drop(status);

        *self.pause_flag.lock().unwrap() = true;
        Ok(())
    }

    pub fn resume(&self, app: &AppHandle) -> Result<(), RalphError> {
        {
            let mut status = self.status.lock().unwrap();
            if status.state != LoopState::Paused {
                return Err(RalphError::NotRunning);
            }
            status.state = LoopState::Running;
        }

        *self.pause_flag.lock().unwrap() = false;
        self.emit_state_changed(app);
        Ok(())
    }

    pub fn stop(&self, app: &AppHandle) -> Result<(), RalphError> {
        *self.stop_flag.lock().unwrap() = true;

        // Kill the current subprocess if running
        if let Some(pid) = *self.current_pid.lock().unwrap() {
            #[cfg(unix)]
            {
                use std::process::Command;
                let _ = Command::new("kill")
                    .arg("-TERM")
                    .arg(pid.to_string())
                    .spawn();
            }
            *self.current_pid.lock().unwrap() = None;
        }

        {
            let mut status = self.status.lock().unwrap();
            status.state = LoopState::Aborted;
        }

        self.emit_state_changed(app);
        Ok(())
    }

    fn run_loop(
        app: AppHandle,
        status: Arc<Mutex<LoopStatus>>,
        _config_arc: Arc<Mutex<Option<LoopConfig>>>,
        pause_flag: Arc<Mutex<bool>>,
        stop_flag: Arc<Mutex<bool>>,
        current_pid: Arc<Mutex<Option<u32>>>,
        config: LoopConfig,
    ) {
        let mut iterations_since_opus = 0;

        loop {
            // Check stop flag
            if *stop_flag.lock().unwrap() {
                let mut s = status.lock().unwrap();
                s.state = LoopState::Aborted;
                Self::emit_event(
                    &app,
                    RalphEvent::StateChanged {
                        state: s.state,
                        iteration: s.current_iteration,
                    },
                );
                break;
            }

            // Check pause flag
            if *pause_flag.lock().unwrap() {
                let mut s = status.lock().unwrap();
                s.state = LoopState::Paused;
                Self::emit_event(
                    &app,
                    RalphEvent::StateChanged {
                        state: s.state,
                        iteration: s.current_iteration,
                    },
                );

                // Wait for unpause
                while *pause_flag.lock().unwrap() && !*stop_flag.lock().unwrap() {
                    thread::sleep(Duration::from_millis(100));
                }
                continue;
            }

            // Check iteration limit
            {
                let s = status.lock().unwrap();
                if s.current_iteration >= config.max_iterations {
                    drop(s);
                    let mut s = status.lock().unwrap();
                    s.state = LoopState::Complete;
                    Self::emit_event(
                        &app,
                        RalphEvent::StateChanged {
                            state: s.state,
                            iteration: s.current_iteration,
                        },
                    );
                    break;
                }
            }

            // Check stagnation
            {
                let s = status.lock().unwrap();
                if s.stagnant_count >= config.max_stagnant_iterations {
                    drop(s);
                    Self::emit_event(
                        &app,
                        RalphEvent::Error {
                            message: format!(
                                "Stagnation detected after {} iterations with no progress",
                                config.max_stagnant_iterations
                            ),
                        },
                    );
                    let mut s = status.lock().unwrap();
                    s.state = LoopState::Aborted;
                    Self::emit_event(
                        &app,
                        RalphEvent::StateChanged {
                            state: s.state,
                            iteration: s.current_iteration,
                        },
                    );
                    break;
                }
            }

            // Determine if this is an Opus review iteration
            let is_opus_review = iterations_since_opus >= config.haiku_iterations_before_opus;
            let model = if is_opus_review { "opus" } else { "haiku" };

            // Build prompt
            let prompt_result = if is_opus_review {
                PromptBuilder::build_opus_review_prompt(&config.project_path)
            } else {
                PromptBuilder::build_haiku_prompt(&config.project_path)
            };

            let prompt = match prompt_result {
                Ok(p) => p,
                Err(e) => {
                    Self::emit_event(
                        &app,
                        RalphEvent::Error {
                            message: format!("Failed to build prompt: {}", e),
                        },
                    );
                    let mut s = status.lock().unwrap();
                    s.state = LoopState::Aborted;
                    Self::emit_event(
                        &app,
                        RalphEvent::StateChanged {
                            state: s.state,
                            iteration: s.current_iteration,
                        },
                    );
                    break;
                }
            };

            // Run Claude iteration
            let (rx, handle) = ClaudeClient::run_iteration_async(
                &config.project_path,
                prompt,
                model.to_string(),
                config.iteration_timeout_secs,
                Arc::clone(&current_pid),
            );

            // Stream output to frontend
            let mut rate_limited = false;
            for output in rx {
                match output {
                    ClaudeOutput::Text(text) => {
                        Self::emit_event(&app, RalphEvent::OutputChunk { text });
                    }
                    ClaudeOutput::RateLimited => {
                        rate_limited = true;
                    }
                    ClaudeOutput::Error(msg) => {
                        Self::emit_event(&app, RalphEvent::Error { message: msg });
                    }
                    ClaudeOutput::Complete => {}
                }
            }

            // Wait for thread to finish
            let result = handle.join().unwrap_or(Err(RalphError::ClaudeProcessError(
                "Thread panicked".to_string(),
            )));

            // Handle rate limiting
            if rate_limited
                || matches!(result, Err(ref e) if e.to_string().contains("Rate limited"))
            {
                let mut s = status.lock().unwrap();
                s.rate_limit_retries += 1;

                if s.rate_limit_retries > config.max_rate_limit_retries {
                    Self::emit_event(
                        &app,
                        RalphEvent::Error {
                            message: format!(
                                "Rate limit exceeded after {} retries",
                                config.max_rate_limit_retries
                            ),
                        },
                    );
                    s.state = LoopState::Aborted;
                    Self::emit_event(
                        &app,
                        RalphEvent::StateChanged {
                            state: s.state,
                            iteration: s.current_iteration,
                        },
                    );
                    break;
                }

                s.state = LoopState::RateLimited;
                let attempt = s.rate_limit_retries;
                drop(s);

                Self::emit_event(
                    &app,
                    RalphEvent::RateLimited {
                        retry_in_secs: config.rate_limit_retry_secs,
                        attempt,
                        max_attempts: config.max_rate_limit_retries,
                    },
                );

                // Wait before retry
                thread::sleep(Duration::from_secs(config.rate_limit_retry_secs));

                {
                    let mut s = status.lock().unwrap();
                    s.state = LoopState::Running;
                }
                Self::emit_event(
                    &app,
                    RalphEvent::StateChanged {
                        state: LoopState::Running,
                        iteration: status.lock().unwrap().current_iteration,
                    },
                );
                continue;
            }

            // Process result
            match result {
                Ok(output) => {
                    // Reset rate limit counter on success
                    {
                        let mut s = status.lock().unwrap();
                        s.rate_limit_retries = 0;
                    }

                    // Check for completion marker
                    if PromptBuilder::check_completion(&output) {
                        let mut s = status.lock().unwrap();
                        s.state = LoopState::Complete;
                        Self::emit_event(
                            &app,
                            RalphEvent::IterationComplete {
                                iteration: s.current_iteration,
                                success: true,
                                message: Some("All tasks complete!".to_string()),
                            },
                        );
                        Self::emit_event(
                            &app,
                            RalphEvent::StateChanged {
                                state: s.state,
                                iteration: s.current_iteration,
                            },
                        );
                        break;
                    }

                    // Check for stagnation
                    let post_hash = Self::get_progress_hash(&config.project_path);
                    {
                        let mut s = status.lock().unwrap();
                        if Some(&post_hash) == s.last_progress_hash.as_ref() {
                            s.stagnant_count += 1;
                        } else {
                            s.stagnant_count = 0;
                            s.last_progress_hash = Some(post_hash);
                        }

                        s.current_iteration += 1;
                        Self::emit_event(
                            &app,
                            RalphEvent::IterationComplete {
                                iteration: s.current_iteration,
                                success: true,
                                message: None,
                            },
                        );
                    }

                    // Update Opus counter
                    if is_opus_review {
                        iterations_since_opus = 0;
                    } else {
                        iterations_since_opus += 1;
                    }
                }
                Err(e) => {
                    Self::emit_event(
                        &app,
                        RalphEvent::Error {
                            message: format!("Iteration failed: {}", e),
                        },
                    );

                    let mut s = status.lock().unwrap();
                    Self::emit_event(
                        &app,
                        RalphEvent::IterationComplete {
                            iteration: s.current_iteration,
                            success: false,
                            message: Some(e.to_string()),
                        },
                    );

                    // Continue to next iteration on non-fatal errors
                    s.current_iteration += 1;
                }
            }
        }
    }

    fn get_progress_hash(project_path: &PathBuf) -> String {
        let progress_path = project_path.join(".ralph/progress.txt");
        let prd_path = project_path.join(".ralph/prd.yaml");

        let progress = std::fs::read_to_string(&progress_path).unwrap_or_default();
        let prd = std::fs::read_to_string(&prd_path).unwrap_or_default();

        hash_content(&format!("{}{}", progress, prd))
    }

    fn emit_state_changed(&self, app: &AppHandle) {
        let status = self.status.lock().unwrap();
        Self::emit_event(
            app,
            RalphEvent::StateChanged {
                state: status.state,
                iteration: status.current_iteration,
            },
        );
    }

    fn emit_event(app: &AppHandle, event: RalphEvent) {
        let event_name = match &event {
            RalphEvent::StateChanged { .. } => "ralph://state_changed",
            RalphEvent::OutputChunk { .. } => "ralph://output_chunk",
            RalphEvent::IterationComplete { .. } => "ralph://iteration_complete",
            RalphEvent::RateLimited { .. } => "ralph://rate_limited",
            RalphEvent::Error { .. } => "ralph://error",
        };
        let _ = app.emit(event_name, event);
    }
}
