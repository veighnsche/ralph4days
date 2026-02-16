use super::state::AppState;
use core_contracts::remote::{RemoteConnectArgs, RemoteConnectResult, RemoteStatus};
use core_errors::{codes, err_string, RalphResult};
use std::sync::Arc;
use tauri::{AppHandle, State};

fn remote_status_disconnected() -> RemoteStatus {
    RemoteStatus {
        connected: false,
        ws_url: None,
        protocol: None,
    }
}

fn remote_status_connected(conn: &crate::remote::RemoteWireFrameConnection) -> RemoteStatus {
    RemoteStatus {
        connected: conn.is_connected(),
        ws_url: Some(conn.ws_url().to_owned()),
        protocol: Some(conn.remote_protocol()),
    }
}

async fn shutdown_stale_connection(conn: crate::remote::RemoteWireFrameConnection) {
    if let Err(err) = conn.shutdown().await {
        tracing::warn!(error = %err, "Failed to shutdown stale remote connection");
    }
}

#[tauri::command]
pub async fn remote_connect(
    app: AppHandle,
    state: State<'_, AppState>,
    args: RemoteConnectArgs,
) -> RalphResult<RemoteConnectResult> {
    let mut guard = state.remote.lock().await;

    if let Some(existing) = guard.as_ref() {
        if existing.is_connected() {
            return Err(err_string(
                codes::INTERNAL,
                format!(
                    "Remote already connected (wsUrl='{}'). Disconnect first.",
                    existing.ws_url()
                ),
            ));
        }

        // Stale/disconnected connection: close it before reconnecting.
        if let Some(stale) = guard.take() {
            shutdown_stale_connection(stale).await;
        }
    }

    let sink = Arc::new(crate::event_sink::TauriEventSink::new(app));
    let conn = crate::remote::RemoteWireFrameConnection::connect(args.ws_url.clone(), sink).await?;
    let protocol = conn.remote_protocol();

    *guard = Some(conn);

    Ok(RemoteConnectResult {
        ws_url: args.ws_url,
        protocol,
    })
}

#[tauri::command]
pub async fn remote_disconnect(state: State<'_, AppState>) -> RalphResult<()> {
    let conn = { state.remote.lock().await.take() };
    if let Some(conn) = conn {
        conn.shutdown().await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn remote_status_get(state: State<'_, AppState>) -> RalphResult<RemoteStatus> {
    let guard = state.remote.lock().await;
    let status = guard
        .as_ref()
        .map_or_else(remote_status_disconnected, remote_status_connected);

    Ok(status)
}
