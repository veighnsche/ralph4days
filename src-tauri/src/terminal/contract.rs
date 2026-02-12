use ralph_macros::ipc_type;
use serde::{Deserialize, Serialize};

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalBridgeStartSessionArgs {
    pub session_id: String,
    pub agent: Option<String>,
    pub mcp_mode: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub permission_level: Option<String>,
    pub thinking: Option<bool>,
    pub post_start_preamble: Option<String>,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalBridgeStartTaskSessionArgs {
    pub session_id: String,
    pub task_id: u32,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub permission_level: Option<String>,
    pub thinking: Option<bool>,
    pub post_start_preamble: Option<String>,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalBridgeModelOption {
    pub name: String,
    pub display: String,
    pub description: String,
    pub session_model: Option<String>,
    pub effort_options: Vec<String>,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalBridgeListModelsResult {
    pub agent: String,
    pub models: Vec<TerminalBridgeModelOption>,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalBridgeListModelFormTreeResult {
    pub providers: Vec<TerminalBridgeListModelsResult>,
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

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalBridgeStartHumanSessionArgs {
    pub terminal_session_id: String,
    pub kind: String,
    pub task_id: Option<u32>,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub permission_level: Option<String>,
    pub launch_command: Option<String>,
    pub post_start_preamble: Option<String>,
    pub init_prompt: Option<String>,
    pub mcp_mode: Option<String>,
    pub thinking: Option<bool>,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalBridgeStartHumanSessionResult {
    pub agent_session_id: String,
    pub agent_session_number: u32,
}
