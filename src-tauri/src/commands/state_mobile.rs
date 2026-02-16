use ralph_errors::{codes, err_string, RalphResult};

pub struct AppState {
    pub remote: tokio::sync::Mutex<Option<crate::remote::RemoteWireFrameConnection>>,
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
        let guard = self.remote.lock().await;
        let conn = guard.as_ref().ok_or_else(|| {
            err_string(
                codes::INTERNAL,
                "Remote transport is required on mobile. Call remote_connect first.",
            )
        })?;

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

    pub async fn remote_rpc_client_required(&self) -> RalphResult<crate::remote::RemoteRpcClient> {
        self.remote_rpc_client().await?.ok_or_else(|| {
            err_string(
                codes::INTERNAL,
                "Remote transport is not connected. Call remote_connect first.",
            )
        })
    }
}
