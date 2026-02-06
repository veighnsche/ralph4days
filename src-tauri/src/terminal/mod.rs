//! Terminal subsystem - PTY session management for interactive Claude Code sessions

pub use manager::PTYManager;
pub use session::SessionConfig;

mod events;
mod manager;
mod session;
