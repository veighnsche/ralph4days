//! Terminal bridge subsystem - PTY-backed session transport for interactive agent sessions

pub use contract::{
    TerminalBridgeEmitSystemMessageArgs, TerminalBridgeListModelsResult, TerminalBridgeResizeArgs,
    TerminalBridgeSendInputArgs, TerminalBridgeStartHumanSessionArgs,
    TerminalBridgeStartHumanSessionResult, TerminalBridgeStartSessionArgs,
    TerminalBridgeStartTaskSessionArgs, TerminalBridgeTerminateArgs,
};
pub use events::{PtyOutputEvent, TERMINAL_BRIDGE_OUTPUT_EVENT};
pub use manager::PTYManager;
pub use session::{SessionConfig, SessionInitSettings};

mod contract;
mod events;
mod manager;
pub(crate) mod providers;
mod session;
