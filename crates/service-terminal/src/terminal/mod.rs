//! Terminal bridge subsystem - PTY-backed session transport for interactive agent sessions

pub use contract::{
    TerminalAgent, TerminalBridgeEmitSystemMessageArgs, TerminalBridgeLaunchDefaults,
    TerminalBridgeLaunchSource, TerminalBridgeListModelFormTreeResult,
    TerminalBridgeListModelsResult, TerminalBridgeModelOption, TerminalBridgeReplayOutputArgs,
    TerminalBridgeReplayOutputChunk, TerminalBridgeReplayOutputResult, TerminalBridgeResizeArgs,
    TerminalBridgeResolveTaskLaunchArgs, TerminalBridgeResolvedLaunchConfig,
    TerminalBridgeSendInputArgs, TerminalBridgeSetStreamModeArgs,
    TerminalBridgeStartHumanSessionArgs, TerminalBridgeStartHumanSessionResult,
    TerminalBridgeStartSessionArgs, TerminalBridgeStartTaskSessionArgs,
    TerminalBridgeTerminateArgs, TerminalMcpMode,
};
pub use core_contracts::terminal::PtyOutputEvent;
pub use manager::{PTYManager, SessionStreamMode};
pub use session::{resolve_task_launch_config, SessionConfig, SessionInitSettings};

mod contract;
mod manager;
mod mappers;
pub mod providers;
mod session;
