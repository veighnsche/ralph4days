use crate::events::BackendDiagnosticEvent;
use crate::terminal::{PtyClosedEvent, PtyOutputEvent};
use std::future::Future;
use std::pin::Pin;

pub trait EventSink: Send + Sync {
    fn emit_backend_diagnostic(&self, payload: BackendDiagnosticEvent) -> Result<(), String>;

    fn emit_terminal_output(&self, payload: PtyOutputEvent) -> Result<(), String>;

    fn emit_terminal_closed(&self, payload: PtyClosedEvent) -> Result<(), String>;
}

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Transport-agnostic invoke-style RPC.
///
/// Intended uses:
/// - Tauri backend "remote mode" can forward `#[tauri::command]` calls to a remote `ralphd`.
/// - `ralphd` unit tests can swap in an in-memory fake transport.
pub trait RpcClient: Send + Sync {
    fn invoke(
        &self,
        command: String,
        args: serde_json::Value,
    ) -> BoxFuture<'_, Result<serde_json::Value, String>>;
}
