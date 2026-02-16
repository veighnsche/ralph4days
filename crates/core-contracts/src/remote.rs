use crate::protocol::ProtocolVersionInfo;
use core_macros::ipc_type;

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

#[ipc_type(rename_all = "snake_case")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemoteSshAuthMode {
    Key,
    Password,
}

#[ipc_type(rename_all = "snake_case")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemoteSshTransportKind {
    Stream,
    TcpLoopback,
}

#[ipc_type]
pub struct RemoteSshConnectArgs {
    pub host: String,
    pub username: String,
    pub ssh_port: u16,
    pub remote_port: u16,
    pub auth_mode: RemoteSshAuthMode,
    pub identity_file: Option<String>,
    pub password: Option<String>,
    pub key_passphrase: Option<String>,
    pub known_hosts_file: Option<String>,
}

#[ipc_type]
pub struct RemoteSshConnectResult {
    pub ws_url: String,
    pub protocol: ProtocolVersionInfo,
    pub ssh_session_id: u32,
    pub host: String,
    pub username: String,
    pub ssh_port: u16,
    pub remote_port: u16,
    pub auth_mode: RemoteSshAuthMode,
    pub transport_kind: RemoteSshTransportKind,
    pub active_profile_id: Option<String>,
    pub identity_file: Option<String>,
    pub known_hosts_file: Option<String>,
}

#[ipc_type]
pub struct RemoteSshStatus {
    pub active: bool,
    pub ws_url: Option<String>,
    pub ssh_session_id: Option<u32>,
    pub active_profile_id: Option<String>,
    pub host: Option<String>,
    pub username: Option<String>,
    pub ssh_port: Option<u16>,
    pub remote_port: Option<u16>,
    pub auth_mode: Option<RemoteSshAuthMode>,
    pub transport_kind: Option<RemoteSshTransportKind>,
    pub identity_file: Option<String>,
    pub known_hosts_file: Option<String>,
}

#[ipc_type]
pub struct RemoteSshProfile {
    pub id: String,
    pub name: String,
    pub host: String,
    pub username: String,
    pub ssh_port: u16,
    pub remote_port: u16,
    pub auth_mode: RemoteSshAuthMode,
    pub identity_file: Option<String>,
    pub identity_ref: Option<String>,
    pub known_hosts_file: Option<String>,
    pub auto_reconnect_enabled: bool,
    pub last_used_at: Option<String>,
}

#[ipc_type]
pub struct RemoteSshProfileUpsertArgs {
    pub id: Option<String>,
    pub name: String,
    pub host: String,
    pub username: String,
    pub ssh_port: u16,
    pub remote_port: u16,
    pub auth_mode: RemoteSshAuthMode,
    pub identity_file: Option<String>,
    pub identity_ref: Option<String>,
    pub known_hosts_file: Option<String>,
    pub auto_reconnect_enabled: bool,
    pub password: Option<String>,
    pub key_passphrase: Option<String>,
    pub save_password: bool,
    pub save_key_passphrase: bool,
}

#[ipc_type]
pub struct RemoteSshProfileDeleteArgs {
    pub profile_id: String,
}

#[ipc_type]
pub struct RemoteSshProfileSetLastUsedArgs {
    pub profile_id: String,
}

#[ipc_type]
pub struct RemoteSshProfileConnectArgs {
    pub profile_id: String,
    pub password: Option<String>,
    pub key_passphrase: Option<String>,
}

#[ipc_type]
pub struct RemoteSshIdentityImportArgs {
    pub profile_id: String,
    pub file_name: String,
    pub bytes_base64: String,
    pub passphrase: Option<String>,
    pub save_passphrase: bool,
}

#[ipc_type]
pub struct RemoteSshIdentityImportResult {
    pub identity_ref: String,
}

#[ipc_type]
pub struct RemoteSshHostKeyChallenge {
    pub challenge_id: String,
    pub host: String,
    pub ssh_port: u16,
    pub algorithm: String,
    pub fingerprint_sha256: String,
    pub known_hosts_target_path: String,
    pub expires_at: String,
}

#[ipc_type]
pub struct RemoteSshHostKeyChallengeApproveArgs {
    pub challenge_id: String,
}

#[ipc_type]
pub struct RemoteSshHostKeyChallengeRejectArgs {
    pub challenge_id: String,
}
