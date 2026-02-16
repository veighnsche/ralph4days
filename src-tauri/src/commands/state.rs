#[cfg(not(mobile))]
#[path = "state_desktop.rs"]
mod imp;

#[cfg(mobile)]
#[path = "state_mobile.rs"]
mod imp;

pub use imp::AppState;
#[cfg(not(mobile))]
pub(crate) use imp::{with_db, CommandContext};

use core_contracts::remote::{
    RemoteSshAuthMode, RemoteSshHostKeyChallenge, RemoteSshTransportKind,
};
use core_errors::{codes, err_string, RalphResult};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

pub(crate) type RemoteTransport =
    tokio::sync::Mutex<Option<crate::remote::RemoteWireFrameConnection>>;

pub(crate) struct SshTunnelSession {
    pub session_id: u32,
    pub connection: crate::ssh_tunnel::EmbeddedSshConnection,
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

impl SshTunnelSession {
    pub fn ws_url(&self) -> String {
        format!("ws://127.0.0.1:{}", self.remote_port)
    }

    pub fn is_closed(&self) -> bool {
        self.connection.is_closed()
    }
}

pub(crate) type SshTunnelManager = tokio::sync::Mutex<Option<SshTunnelSession>>;

pub(crate) struct PendingSshHostKeyChallenge {
    pub challenge: RemoteSshHostKeyChallenge,
    pub known_hosts_path: PathBuf,
    pub server_public_key: russh::keys::ssh_key::PublicKey,
    pub expires_at: Instant,
}

pub(crate) type SshHostKeyChallengeManager =
    tokio::sync::Mutex<HashMap<String, PendingSshHostKeyChallenge>>;

pub(crate) async fn remote_rpc_client_from_transport(
    remote: &RemoteTransport,
    required_on_absent: bool,
) -> RalphResult<Option<crate::remote::RemoteRpcClient>> {
    let guard = remote.lock().await;
    let Some(conn) = guard.as_ref() else {
        return if required_on_absent {
            Err(err_string(
                codes::INTERNAL,
                "Remote transport is required on mobile. Call remote_connect first.",
            ))
        } else {
            Ok(None)
        };
    };

    if conn.is_connected() {
        Ok(Some(conn.rpc_client()))
    } else {
        Err(err_string(
            codes::INTERNAL,
            format!(
                "Remote transport disconnected (wsUrl='{}'). Reconnect.",
                conn.ws_url()
            ),
        ))
    }
}

pub(crate) fn remote_rpc_client_required(
    rpc: Option<crate::remote::RemoteRpcClient>,
) -> RalphResult<crate::remote::RemoteRpcClient> {
    rpc.ok_or_else(|| {
        err_string(
            codes::INTERNAL,
            "Remote transport is not connected. Call remote_connect first.",
        )
    })
}
