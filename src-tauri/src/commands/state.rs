#[cfg(not(mobile))]
#[path = "state_desktop.rs"]
mod imp;

#[cfg(mobile)]
#[path = "state_mobile.rs"]
mod imp;

pub use imp::AppState;
#[cfg(not(mobile))]
pub(crate) use imp::{with_db, CommandContext};

use core_errors::{codes, err_string, RalphResult};

pub(crate) type RemoteTransport =
    tokio::sync::Mutex<Option<crate::remote::RemoteWireFrameConnection>>;

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
