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
pub use manager::{PTYManager, SessionStreamMode};
pub use ralph_contracts::terminal::PtyOutputEvent;
pub use session::{SessionConfig, SessionInitSettings};

mod contract;
mod manager;
mod mappers;
pub(crate) mod providers;
mod session;
