//! Terminal bridge subsystem - PTY-backed session transport for interactive agent sessions

pub use contract::{
    TerminalBridgeEmitSystemMessageArgs, TerminalBridgeResizeArgs, TerminalBridgeSendInputArgs,
    TerminalBridgeStartHumanSessionArgs, TerminalBridgeStartHumanSessionResult,
    TerminalBridgeStartSessionArgs, TerminalBridgeStartTaskSessionArgs,
    TerminalBridgeTerminateArgs,
};
pub use events::{PtyOutputEvent, TERMINAL_BRIDGE_OUTPUT_EVENT};
pub use manager::PTYManager;
pub use session::SessionConfig;

mod contract;
mod events;
mod manager;
mod session;
