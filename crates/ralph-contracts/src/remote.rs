use crate::protocol::ProtocolVersionInfo;
use ralph_macros::ipc_type;

#[ipc_type]
pub struct RemoteConnectArgs {
    pub ws_url: String,
}

#[ipc_type]
pub struct RemoteConnectResult {
    pub ws_url: String,
    pub protocol: ProtocolVersionInfo,
}

#[ipc_type]
pub struct RemoteStatus {
    pub connected: bool,
    pub ws_url: Option<String>,
    pub protocol: Option<ProtocolVersionInfo>,
}
