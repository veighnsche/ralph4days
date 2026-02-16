use super::{
    remote_rpc_client_from_transport, remote_rpc_client_required, RemoteTransport,
    SshHostKeyChallengeManager, SshTunnelManager,
};
use core_errors::RalphResult;

pub struct AppState {
    pub remote: RemoteTransport,
    pub ssh_tunnel: SshTunnelManager,
    pub ssh_host_key_challenges: SshHostKeyChallengeManager,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            remote: tokio::sync::Mutex::new(None),
            ssh_tunnel: tokio::sync::Mutex::new(None),
            ssh_host_key_challenges: tokio::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

impl AppState {
    pub async fn remote_rpc_client(&self) -> RalphResult<Option<crate::remote::RemoteRpcClient>> {
        remote_rpc_client_from_transport(&self.remote, true).await
    }

    pub async fn remote_rpc_client_required(&self) -> RalphResult<crate::remote::RemoteRpcClient> {
        remote_rpc_client_required(self.remote_rpc_client().await?)
    }
}
