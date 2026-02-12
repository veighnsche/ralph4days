use ralph_macros::ipc_type;
use serde::{Deserialize, Serialize};

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalBridgeStartSessionArgs {
    pub session_id: String,
    pub mcp_mode: Option<String>,
    pub model: Option<String>,
    pub thinking: Option<bool>,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalBridgeStartTaskSessionArgs {
    pub session_id: String,
    pub task_id: u32,
    pub model: Option<String>,
    pub thinking: Option<bool>,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalBridgeSendInputArgs {
    pub session_id: String,
    pub data: Vec<u8>,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalBridgeResizeArgs {
    pub session_id: String,
    pub cols: u16,
    pub rows: u16,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalBridgeTerminateArgs {
    pub session_id: String,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalBridgeEmitSystemMessageArgs {
    pub session_id: String,
    pub text: String,
}
