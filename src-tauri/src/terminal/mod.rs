//! Terminal bridge subsystem - PTY-backed session transport for interactive agent sessions

pub use contract::{
    TerminalBridgeEmitSystemMessageArgs, TerminalBridgeListModelFormTreeResult,
    TerminalBridgeListModelsResult, TerminalBridgeModelOption, TerminalBridgeReplayOutputArgs,
    TerminalBridgeReplayOutputChunk, TerminalBridgeReplayOutputResult, TerminalBridgeResizeArgs,
    TerminalBridgeSendInputArgs, TerminalBridgeSetStreamModeArgs,
    TerminalBridgeStartHumanSessionArgs, TerminalBridgeStartHumanSessionResult,
    TerminalBridgeStartSessionArgs, TerminalBridgeStartTaskSessionArgs,
    TerminalBridgeTerminateArgs,
};
pub use events::{PtyOutputEvent, TERMINAL_OUTPUT_EVENT};
pub use manager::{PTYManager, SessionStreamMode};
pub use session::{SessionConfig, SessionInitSettings};

mod contract;
mod events;
mod manager;
mod mappers;
pub(crate) mod providers;
mod session;
