use super::remote_proxy::{remote_invoke_args, remote_invoke_no_args};
use super::state::AppState;
use core_errors::RalphResult;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::future::Future;

#[cfg(mobile)]
pub(crate) trait PlatformArgAlias<T> {
    type Type;
}

#[cfg(mobile)]
impl<T> PlatformArgAlias<T> for () {
    type Type = serde_json::Value;
}

#[cfg(mobile)]
pub(crate) type PlatformArg<T> = <() as PlatformArgAlias<T>>::Type;

#[cfg(not(mobile))]
pub(crate) type PlatformArg<T> = T;

#[cfg(mobile)]
pub(crate) trait PlatformOutAlias<T> {
    type Type;
}

#[cfg(mobile)]
impl<T> PlatformOutAlias<T> for () {
    type Type = serde_json::Value;
}

#[cfg(mobile)]
pub(crate) type PlatformOut<T> = <() as PlatformOutAlias<T>>::Type;

#[cfg(not(mobile))]
pub(crate) type PlatformOut<T> = T;

pub(crate) trait CommandExecutor {
    async fn remote_client(
        &self,
        state: &AppState,
    ) -> RalphResult<Option<crate::remote::RemoteRpcClient>>;
}

pub(crate) struct PlatformExecutor;

impl PlatformExecutor {
    const fn new() -> Self {
        Self
    }
}

#[cfg(not(mobile))]
impl CommandExecutor for PlatformExecutor {
    async fn remote_client(
        &self,
        state: &AppState,
    ) -> RalphResult<Option<crate::remote::RemoteRpcClient>> {
        state.remote_rpc_client().await
    }
}

#[cfg(mobile)]
impl CommandExecutor for PlatformExecutor {
    async fn remote_client(
        &self,
        state: &AppState,
    ) -> RalphResult<Option<crate::remote::RemoteRpcClient>> {
        let rpc = state.remote_rpc_client_required().await?;
        Ok(Some(rpc))
    }
}

pub(crate) async fn dispatch_args<TArgs, TResult, FLocal>(
    state: &AppState,
    command: &str,
    args: TArgs,
    local: FLocal,
) -> RalphResult<TResult>
where
    TArgs: Serialize,
    TResult: DeserializeOwned,
    FLocal: FnOnce(TArgs) -> RalphResult<TResult>,
{
    let executor = PlatformExecutor::new();
    if let Some(rpc) = executor.remote_client(state).await? {
        return remote_invoke_args(&rpc, command, args).await;
    }

    local(args)
}

pub(crate) async fn dispatch_no_args<TResult, FLocal>(
    state: &AppState,
    command: &str,
    local: FLocal,
) -> RalphResult<TResult>
where
    TResult: DeserializeOwned,
    FLocal: FnOnce() -> RalphResult<TResult>,
{
    let executor = PlatformExecutor::new();
    if let Some(rpc) = executor.remote_client(state).await? {
        return remote_invoke_no_args(&rpc, command).await;
    }

    local()
}

pub(crate) async fn dispatch_args_async<TArgs, TResult, FLocal, FLocalFuture>(
    state: &AppState,
    command: &str,
    args: TArgs,
    local: FLocal,
) -> RalphResult<TResult>
where
    TArgs: Serialize,
    TResult: DeserializeOwned,
    FLocal: FnOnce(TArgs) -> FLocalFuture,
    FLocalFuture: Future<Output = RalphResult<TResult>>,
{
    let executor = PlatformExecutor::new();
    if let Some(rpc) = executor.remote_client(state).await? {
        return remote_invoke_args(&rpc, command, args).await;
    }

    local(args).await
}
