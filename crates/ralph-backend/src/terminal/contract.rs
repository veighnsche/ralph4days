use ralph_macros::ipc_type;

#[ipc_type]
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
pub struct TerminalBridgeModelOption {
    pub name: String,
    pub display: String,
    pub description: String,
    pub session_model: Option<String>,
    pub effort_options: Vec<String>,
}

#[ipc_type]
pub struct TerminalBridgeListModelsResult {
    pub agent: String,
    pub models: Vec<TerminalBridgeModelOption>,
}

#[ipc_type]
pub struct TerminalBridgeListModelFormTreeResult {
    pub providers: Vec<TerminalBridgeListModelsResult>,
}

#[ipc_type]
pub struct TerminalBridgeSendInputArgs {
    pub session_id: String,
    pub data: Vec<u8>,
}

#[ipc_type]
pub struct TerminalBridgeResizeArgs {
    pub session_id: String,
    pub cols: u16,
    pub rows: u16,
}

#[ipc_type]
pub struct TerminalBridgeTerminateArgs {
    pub session_id: String,
}

#[ipc_type]
pub struct TerminalBridgeSetStreamModeArgs {
    pub session_id: String,
    pub mode: String,
}

#[ipc_type]
pub struct TerminalBridgeReplayOutputArgs {
    pub session_id: String,
    #[serde(
        serialize_with = "ralph_contracts::json_safe::serialize_u64",
        deserialize_with = "ralph_contracts::json_safe::deserialize_u64"
    )]
    #[ts(type = "number")]
    pub after_seq: u64,
    pub limit: u32,
}

#[ipc_type]
pub struct TerminalBridgeReplayOutputChunk {
    #[serde(
        serialize_with = "ralph_contracts::json_safe::serialize_u64",
        deserialize_with = "ralph_contracts::json_safe::deserialize_u64"
    )]
    #[ts(type = "number")]
    pub seq: u64,
    pub data: String,
}

#[ipc_type]
pub struct TerminalBridgeReplayOutputResult {
    pub chunks: Vec<TerminalBridgeReplayOutputChunk>,
    pub has_more: bool,
    pub truncated: bool,
    #[serde(
        serialize_with = "ralph_contracts::json_safe::serialize_option_u64",
        deserialize_with = "ralph_contracts::json_safe::deserialize_option_u64"
    )]
    #[ts(type = "number")]
    pub truncated_until_seq: Option<u64>,
}

#[ipc_type]
pub struct TerminalBridgeEmitSystemMessageArgs {
    pub session_id: String,
    pub text: String,
}

#[ipc_type]
pub struct TerminalBridgeStartHumanSessionArgs {
    pub terminal_session_id: String,
    pub kind: String,
    pub task_id: Option<u32>,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub permission_level: Option<String>,
    pub post_start_preamble: Option<String>,
    pub init_prompt: Option<String>,
    pub mcp_mode: Option<String>,
    pub thinking: Option<bool>,
}

#[ipc_type]
pub struct TerminalBridgeStartHumanSessionResult {
    pub agent_session_id: String,
    pub agent_session_number: u32,
}

#[ipc_type]
pub enum TerminalBridgeLaunchSource {
    Task,
    Discipline,
    Default,
    Unset,
}

#[ipc_type]
pub struct TerminalBridgeLaunchDefaults {
    pub agent: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub thinking: Option<bool>,
    pub permission_level: Option<String>,
}

#[ipc_type]
pub struct TerminalBridgeResolveTaskLaunchArgs {
    pub task_id: u32,
    pub defaults: TerminalBridgeLaunchDefaults,
}

#[ipc_type]
pub struct TerminalBridgeResolvedLaunchConfig {
    pub agent: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub thinking: Option<bool>,
    pub permission_level: Option<String>,

    pub agent_source: TerminalBridgeLaunchSource,
    pub model_source: TerminalBridgeLaunchSource,
    pub effort_source: TerminalBridgeLaunchSource,
    pub thinking_source: TerminalBridgeLaunchSource,
    pub permission_level_source: TerminalBridgeLaunchSource,

    pub model_supports_effort: bool,
}
