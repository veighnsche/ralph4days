use super::{remote_rpc_client_from_transport, remote_rpc_client_required, RemoteTransport};
use core_errors::RalphResult;

pub struct AppState {
    pub remote: RemoteTransport,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            remote: tokio::sync::Mutex::new(None),
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
