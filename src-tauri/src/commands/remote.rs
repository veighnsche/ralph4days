use super::state::{AppState, PendingSshHostKeyChallenge, SshTunnelSession};
use core_contracts::protocol::ProtocolVersionInfo;
use core_contracts::remote::{
    RemoteConnectArgs, RemoteConnectResult, RemoteSshAuthMode, RemoteSshConnectArgs,
    RemoteSshConnectResult, RemoteSshHostKeyChallenge, RemoteSshHostKeyChallengeApproveArgs,
    RemoteSshHostKeyChallengeRejectArgs, RemoteSshIdentityImportArgs,
    RemoteSshIdentityImportResult, RemoteSshProfile, RemoteSshProfileConnectArgs,
    RemoteSshProfileDeleteArgs, RemoteSshProfileSetLastUsedArgs, RemoteSshProfileUpsertArgs,
    RemoteSshStatus, RemoteSshTransportKind, RemoteStatus,
};
use core_contracts::transport::EventSink;
use core_errors::{codes, err_string, RalphResult};
use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
#[cfg(mobile)]
use tauri::Manager;
use tauri::{AppHandle, State};
use tokio::io::{AsyncRead, AsyncWrite};
use url::Url;

const SSH_HOSTKEY_CHALLENGE_TTL_SECONDS: u64 = 120;

static NEXT_PROFILE_ID_SEQUENCE: AtomicU64 = AtomicU64::new(1);
static NEXT_HOSTKEY_CHALLENGE_ID_SEQUENCE: AtomicU64 = AtomicU64::new(1);

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

fn remote_ssh_status_disconnected() -> RemoteSshStatus {
    RemoteSshStatus {
        active: false,
        ws_url: None,
        ssh_session_id: None,
        active_profile_id: None,
        host: None,
        username: None,
        ssh_port: None,
        remote_port: None,
        auth_mode: None,
        transport_kind: None,
        identity_file: None,
        known_hosts_file: None,
    }
}

fn remote_ssh_status_active(tunnel: &SshTunnelSession) -> RemoteSshStatus {
    RemoteSshStatus {
        active: true,
        ws_url: Some(tunnel.ws_url()),
        ssh_session_id: Some(tunnel.session_id),
        active_profile_id: tunnel.active_profile_id.clone(),
        host: Some(tunnel.host.clone()),
        username: Some(tunnel.username.clone()),
        ssh_port: Some(tunnel.ssh_port),
        remote_port: Some(tunnel.remote_port),
        auth_mode: Some(tunnel.auth_mode),
        transport_kind: Some(tunnel.transport_kind),
        identity_file: tunnel.identity_file.clone(),
        known_hosts_file: tunnel.known_hosts_file.clone(),
    }
}

fn validate_loopback_ws_url(ws_url: &str) -> RalphResult<()> {
    let parsed = Url::parse(ws_url).map_err(|error| {
        err_string(
            codes::INTERNAL,
            format!("Invalid remote wsUrl '{ws_url}': {error}"),
        )
    })?;

    if parsed.scheme() != "ws" {
        return Err(err_string(
            codes::INTERNAL,
            format!(
                "Refusing remote wsUrl '{ws_url}': SSH tunnel mode only allows plain ws:// loopback endpoints."
            ),
        ));
    }

    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(err_string(
            codes::INTERNAL,
            format!("Refusing remote wsUrl '{ws_url}': credentials in URL are forbidden."),
        ));
    }

    if parsed.query().is_some() || parsed.fragment().is_some() {
        return Err(err_string(
            codes::INTERNAL,
            format!("Refusing remote wsUrl '{ws_url}': query strings and fragments are forbidden."),
        ));
    }

    if parsed.path() != "/" {
        return Err(err_string(
            codes::INTERNAL,
            format!(
                "Refusing remote wsUrl '{ws_url}': path components are forbidden (expected root '/')."
            ),
        ));
    }

    let Some(host) = parsed.host() else {
        return Err(err_string(
            codes::INTERNAL,
            format!("Refusing remote wsUrl '{ws_url}': missing host."),
        ));
    };

    let is_loopback_host = match host {
        url::Host::Ipv4(ip) => ip.is_loopback(),
        url::Host::Ipv6(ip) => ip.is_loopback(),
        url::Host::Domain(_) => false,
    };

    if !is_loopback_host {
        return Err(err_string(
            codes::INTERNAL,
            format!(
                "Refusing remote wsUrl '{ws_url}': only ws://127.0.0.1:<port> or ws://[::1]:<port> is allowed. Create an SSH tunnel first."
            ),
        ));
    }

    if parsed.port().is_none() {
        return Err(err_string(
            codes::INTERNAL,
            format!("Refusing remote wsUrl '{ws_url}': explicit port is required."),
        ));
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NormalizedSshConnectArgs {
    host: String,
    username: String,
    ssh_port: u16,
    remote_port: u16,
    auth_mode: RemoteSshAuthMode,
    identity_file: Option<String>,
    password: Option<String>,
    key_passphrase: Option<String>,
    known_hosts_file: Option<String>,
}

fn normalize_optional_trimmed(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    })
}

fn normalize_required_trimmed(value: String, field_name: &str) -> RalphResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(err_string(
            codes::INTERNAL,
            format!("{field_name} must be non-empty."),
        ));
    }
    Ok(trimmed.to_owned())
}

fn normalize_ssh_connect_args(args: RemoteSshConnectArgs) -> RalphResult<NormalizedSshConnectArgs> {
    let host = normalize_required_trimmed(args.host, "SSH host")?;
    let username = normalize_required_trimmed(args.username, "SSH username")?;

    if args.ssh_port == 0 {
        return Err(err_string(
            codes::INTERNAL,
            "SSH tunnel connect requires sshPort > 0.",
        ));
    }

    if args.remote_port == 0 {
        return Err(err_string(
            codes::INTERNAL,
            "SSH tunnel connect requires remotePort > 0.",
        ));
    }

    let identity_file = normalize_optional_trimmed(args.identity_file);
    let password = normalize_optional_trimmed(args.password);
    let key_passphrase = normalize_optional_trimmed(args.key_passphrase);
    let known_hosts_file = normalize_optional_trimmed(args.known_hosts_file);

    match args.auth_mode {
        RemoteSshAuthMode::Key => {
            if password.is_some() {
                return Err(err_string(
                    codes::INTERNAL,
                    "Password must be omitted when authMode='key'.",
                ));
            }
        }
        RemoteSshAuthMode::Password => {
            if identity_file.is_some() {
                return Err(err_string(
                    codes::INTERNAL,
                    "identityFile must be omitted when authMode='password'.",
                ));
            }
            if key_passphrase.is_some() {
                return Err(err_string(
                    codes::INTERNAL,
                    "keyPassphrase must be omitted when authMode='password'.",
                ));
            }
            if password.is_none() {
                return Err(err_string(
                    codes::INTERNAL,
                    "Password auth requires a non-empty password.",
                ));
            }
        }
    }

    Ok(NormalizedSshConnectArgs {
        host,
        username,
        ssh_port: args.ssh_port,
        remote_port: args.remote_port,
        auth_mode: args.auth_mode,
        identity_file,
        password,
        key_passphrase,
        known_hosts_file,
    })
}

fn profile_store_path(app: &AppHandle, state: &AppState) -> RalphResult<PathBuf> {
    #[cfg(not(mobile))]
    {
        let _ = app;
        let config_dir = state.xdg.ensure_config()?;
        return Ok(config_dir.join(crate::ssh_profiles::PROFILE_STORE_FILENAME));
    }

    #[cfg(mobile)]
    {
        let _ = state;
        let config_dir = app.path().app_config_dir().map_err(|error| {
            err_string(
                codes::FILESYSTEM,
                format!("Failed to resolve mobile app config directory: {error}"),
            )
        })?;
        return Ok(config_dir.join(crate::ssh_profiles::PROFILE_STORE_FILENAME));
    }
}

fn load_profiles(app: &AppHandle, state: &AppState) -> RalphResult<Vec<RemoteSshProfile>> {
    let path = profile_store_path(app, state)?;
    crate::ssh_profiles::load_profiles(&path)
}

fn save_profiles(
    app: &AppHandle,
    state: &AppState,
    profiles: &[RemoteSshProfile],
) -> RalphResult<()> {
    let path = profile_store_path(app, state)?;
    crate::ssh_profiles::save_profiles(&path, profiles)
}

fn generate_profile_id() -> String {
    let seq = NEXT_PROFILE_ID_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_millis();
    format!("sshp-{now}-{seq}")
}

fn generate_hostkey_challenge_id() -> String {
    let seq = NEXT_HOSTKEY_CHALLENGE_ID_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_millis();
    format!("sshhk-{now}-{seq}")
}

fn utc_now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn validate_profile_upsert_args(args: &RemoteSshProfileUpsertArgs) -> RalphResult<()> {
    if args.ssh_port == 0 {
        return Err(err_string(
            codes::INTERNAL,
            "SSH profile requires sshPort > 0.",
        ));
    }
    if args.remote_port == 0 {
        return Err(err_string(
            codes::INTERNAL,
            "SSH profile requires remotePort > 0.",
        ));
    }

    let name = args.name.trim();
    if name.is_empty() {
        return Err(err_string(
            codes::INTERNAL,
            "SSH profile name must be non-empty.",
        ));
    }
    let host = args.host.trim();
    if host.is_empty() {
        return Err(err_string(
            codes::INTERNAL,
            "SSH profile host must be non-empty.",
        ));
    }
    let username = args.username.trim();
    if username.is_empty() {
        return Err(err_string(
            codes::INTERNAL,
            "SSH profile username must be non-empty.",
        ));
    }

    let identity_ref_present = args
        .identity_ref
        .as_deref()
        .map(str::trim)
        .is_some_and(|v| !v.is_empty());
    let identity_file_present = args
        .identity_file
        .as_deref()
        .map(str::trim)
        .is_some_and(|v| !v.is_empty());
    let password_present = args
        .password
        .as_deref()
        .map(str::trim)
        .is_some_and(|v| !v.is_empty());
    let key_passphrase_present = args
        .key_passphrase
        .as_deref()
        .map(str::trim)
        .is_some_and(|v| !v.is_empty());

    match args.auth_mode {
        RemoteSshAuthMode::Key => {
            if args.save_password {
                return Err(err_string(
                    codes::INTERNAL,
                    "savePassword must be false when authMode='key'.",
                ));
            }
            if password_present {
                return Err(err_string(
                    codes::INTERNAL,
                    "password must be empty when authMode='key'.",
                ));
            }
        }
        RemoteSshAuthMode::Password => {
            if identity_ref_present {
                return Err(err_string(
                    codes::INTERNAL,
                    "identityRef must be empty when authMode='password'.",
                ));
            }
            if identity_file_present {
                return Err(err_string(
                    codes::INTERNAL,
                    "identityFile must be empty when authMode='password'.",
                ));
            }
            if args.save_key_passphrase {
                return Err(err_string(
                    codes::INTERNAL,
                    "saveKeyPassphrase must be false when authMode='password'.",
                ));
            }
            if key_passphrase_present {
                return Err(err_string(
                    codes::INTERNAL,
                    "keyPassphrase must be empty when authMode='password'.",
                ));
            }
            if args.save_password && !password_present {
                return Err(err_string(
                    codes::INTERNAL,
                    "savePassword=true requires non-empty password.",
                ));
            }
        }
    }

    if args.save_key_passphrase && !key_passphrase_present {
        return Err(err_string(
            codes::INTERNAL,
            "saveKeyPassphrase=true requires non-empty keyPassphrase.",
        ));
    }

    Ok(())
}

fn profile_index_by_id(profiles: &[RemoteSshProfile], profile_id: &str) -> Option<usize> {
    profiles.iter().position(|profile| profile.id == profile_id)
}

async fn disconnect_remote_transport(state: &AppState) -> RalphResult<()> {
    let conn = { state.remote.lock().await.take() };
    if let Some(conn) = conn {
        conn.shutdown().await?;
    }
    Ok(())
}

async fn disconnect_ssh_tunnel(state: &AppState) -> RalphResult<()> {
    let tunnel = { state.ssh_tunnel.lock().await.take() };
    if let Some(tunnel) = tunnel.as_ref() {
        tunnel.connection.disconnect().await?;
    }
    Ok(())
}

async fn disconnect_remote_and_tunnel(state: &AppState) -> RalphResult<()> {
    let remote_result = disconnect_remote_transport(state).await;
    let tunnel_result = disconnect_ssh_tunnel(state).await;
    match (remote_result, tunnel_result) {
        (Err(error), _) => Err(error),
        (Ok(()), Err(error)) => Err(error),
        (Ok(()), Ok(())) => Ok(()),
    }
}

fn cleanup_expired_challenges(challenges: &mut HashMap<String, PendingSshHostKeyChallenge>) {
    let now = Instant::now();
    challenges.retain(|_, pending| pending.expires_at > now);
}

async fn refresh_tunnel_liveness(state: &AppState) -> RalphResult<()> {
    {
        let mut challenges = state.ssh_host_key_challenges.lock().await;
        cleanup_expired_challenges(&mut challenges);
    }

    let mut closed_session_id: Option<u32> = None;
    {
        let mut guard = state.ssh_tunnel.lock().await;
        if let Some(tunnel) = guard.as_ref() {
            if tunnel.is_closed() {
                closed_session_id = Some(tunnel.session_id);
                guard.take();
            }
        }
    }

    if let Some(ssh_session_id) = closed_session_id {
        tracing::warn!(
            ssh_session_id,
            "Embedded SSH tunnel session closed; tearing down remote transport"
        );
        disconnect_remote_transport(state).await?;
    }

    Ok(())
}

async fn connect_remote_transport_with<F, Fut>(
    app: AppHandle,
    state: &AppState,
    ws_url: String,
    connect: F,
) -> RalphResult<ProtocolVersionInfo>
where
    F: FnOnce(String, Arc<dyn EventSink>) -> Fut,
    Fut: Future<Output = RalphResult<crate::remote::RemoteWireFrameConnection>>,
{
    validate_loopback_ws_url(&ws_url)?;

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

        if let Some(stale) = guard.take() {
            shutdown_stale_connection(stale).await;
        }
    }

    let sink: Arc<dyn EventSink> = Arc::new(crate::event_sink::TauriEventSink::new(app));
    let conn = connect(ws_url.clone(), sink).await?;
    let protocol = conn.remote_protocol();
    *guard = Some(conn);
    Ok(protocol)
}

async fn remote_connect_internal(
    app: AppHandle,
    state: &AppState,
    ws_url: String,
) -> RalphResult<ProtocolVersionInfo> {
    connect_remote_transport_with(app, state, ws_url, |ws_url, sink| async move {
        crate::remote::RemoteWireFrameConnection::connect(ws_url, sink).await
    })
    .await
}

async fn remote_connect_internal_with_stream<S>(
    app: AppHandle,
    state: &AppState,
    ws_url: String,
    stream: S,
) -> RalphResult<ProtocolVersionInfo>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    connect_remote_transport_with(app, state, ws_url, move |ws_url, sink| async move {
        crate::remote::RemoteWireFrameConnection::connect_via_stream(ws_url, stream, sink).await
    })
    .await
}

async fn register_unknown_host_challenge(
    state: &AppState,
    args: &NormalizedSshConnectArgs,
) -> RalphResult<Option<RemoteSshHostKeyChallenge>> {
    let probe = crate::ssh_tunnel::probe_unknown_host_key(
        &args.host,
        args.ssh_port,
        args.known_hosts_file.as_deref(),
    )
    .await?;

    let Some(probe) = probe else {
        return Ok(None);
    };

    let challenge_id = generate_hostkey_challenge_id();
    let expires_at =
        chrono::Utc::now() + chrono::Duration::seconds(SSH_HOSTKEY_CHALLENGE_TTL_SECONDS as i64);
    let challenge = RemoteSshHostKeyChallenge {
        challenge_id: challenge_id.clone(),
        host: args.host.clone(),
        ssh_port: args.ssh_port,
        algorithm: probe.algorithm,
        fingerprint_sha256: probe.fingerprint_sha256,
        known_hosts_target_path: probe.known_hosts_path.to_string_lossy().into_owned(),
        expires_at: expires_at.to_rfc3339(),
    };

    let pending = PendingSshHostKeyChallenge {
        challenge: challenge.clone(),
        known_hosts_path: probe.known_hosts_path,
        server_public_key: probe.server_public_key,
        expires_at: Instant::now() + Duration::from_secs(SSH_HOSTKEY_CHALLENGE_TTL_SECONDS),
    };

    let mut challenges = state.ssh_host_key_challenges.lock().await;
    cleanup_expired_challenges(&mut challenges);
    challenges.insert(challenge_id, pending);

    Ok(Some(challenge))
}

async fn connect_ssh_internal(
    app: AppHandle,
    state: &AppState,
    args: NormalizedSshConnectArgs,
    active_profile_id: Option<String>,
) -> RalphResult<RemoteSshConnectResult> {
    if let Some(challenge) = register_unknown_host_challenge(state, &args).await? {
        return Err(err_string(
            codes::INTERNAL,
            format!(
                "SSH host key for '{}:{}' is not trusted. Approve challenge '{}' and retry.",
                challenge.host, challenge.ssh_port, challenge.challenge_id
            ),
        )
        .with_context(
            "ssh_hostkey_challenge",
            serde_json::to_value(challenge).expect("challenge is serializable"),
        ));
    }

    disconnect_remote_and_tunnel(state).await?;

    let auth_payload = match args.auth_mode {
        RemoteSshAuthMode::Key => crate::ssh_tunnel::EmbeddedSshAuth::KeyFile {
            identity_file: args.identity_file.clone(),
            passphrase: args.key_passphrase.clone(),
        },
        RemoteSshAuthMode::Password => crate::ssh_tunnel::EmbeddedSshAuth::Password {
            password: args
                .password
                .clone()
                .expect("normalized password must exist for password auth"),
        },
    };

    let crate::ssh_tunnel::EmbeddedSshConnectOutcome {
        connection,
        stream,
        resolved_identity_file,
        resolved_known_hosts_file,
    } = crate::ssh_tunnel::connect(crate::ssh_tunnel::EmbeddedSshConnectArgs {
        host: args.host.clone(),
        username: args.username.clone(),
        ssh_port: args.ssh_port,
        remote_port: args.remote_port,
        auth_mode: args.auth_mode,
        auth: auth_payload,
        known_hosts_file: args.known_hosts_file.clone(),
    })
    .await?;

    let ws_url = format!("ws://127.0.0.1:{}", args.remote_port);
    let protocol = match remote_connect_internal_with_stream(app, state, ws_url.clone(), stream)
        .await
    {
        Ok(protocol) => protocol,
        Err(connect_error) => {
            let teardown_result = connection.disconnect().await;
            if let Err(teardown_error) = teardown_result {
                return Err(err_string(
                    codes::INTERNAL,
                    format!(
                        "SSH remote connect failed ({connect_error}) and SSH tunnel teardown failed: {teardown_error}"
                    ),
                ));
            }
            return Err(connect_error);
        }
    };

    let ssh_session_id = connection.session_id();
    let known_hosts_file = Some(resolved_known_hosts_file.clone());

    {
        let mut guard = state.ssh_tunnel.lock().await;
        if guard.is_some() {
            disconnect_remote_transport(state).await?;
            connection.disconnect().await?;
            return Err(err_string(
                codes::INTERNAL,
                "SSH tunnel became active concurrently. Disconnect and retry.",
            ));
        }

        *guard = Some(SshTunnelSession {
            session_id: ssh_session_id,
            connection,
            host: args.host.clone(),
            username: args.username.clone(),
            ssh_port: args.ssh_port,
            remote_port: args.remote_port,
            auth_mode: args.auth_mode,
            transport_kind: RemoteSshTransportKind::Stream,
            active_profile_id: active_profile_id.clone(),
            identity_file: resolved_identity_file.clone(),
            known_hosts_file: known_hosts_file.clone(),
        });
    }

    Ok(RemoteSshConnectResult {
        ws_url,
        protocol,
        ssh_session_id,
        host: args.host,
        username: args.username,
        ssh_port: args.ssh_port,
        remote_port: args.remote_port,
        auth_mode: args.auth_mode,
        transport_kind: RemoteSshTransportKind::Stream,
        active_profile_id,
        identity_file: resolved_identity_file,
        known_hosts_file,
    })
}

fn profile_connect_to_connect_args(
    profile: &RemoteSshProfile,
    runtime_password: Option<String>,
    runtime_key_passphrase: Option<String>,
) -> RalphResult<NormalizedSshConnectArgs> {
    let password_runtime = normalize_optional_trimmed(runtime_password);
    let key_passphrase_runtime = normalize_optional_trimmed(runtime_key_passphrase);

    match profile.auth_mode {
        RemoteSshAuthMode::Password => {
            let password = if let Some(password) = password_runtime {
                Some(password)
            } else {
                crate::ssh_secrets::get_profile_secret(
                    &profile.id,
                    crate::ssh_secrets::SshSecretKind::Password,
                )?
            };

            if password.is_none() {
                return Err(err_string(
                    codes::INTERNAL,
                    format!(
                        "SSH profile '{}' requires password auth but no password was provided or stored.",
                        profile.name
                    ),
                ));
            }

            Ok(NormalizedSshConnectArgs {
                host: profile.host.clone(),
                username: profile.username.clone(),
                ssh_port: profile.ssh_port,
                remote_port: profile.remote_port,
                auth_mode: RemoteSshAuthMode::Password,
                identity_file: None,
                password,
                key_passphrase: None,
                known_hosts_file: profile.known_hosts_file.clone(),
            })
        }
        RemoteSshAuthMode::Key => {
            let key_passphrase = if let Some(passphrase) = key_passphrase_runtime {
                Some(passphrase)
            } else {
                crate::ssh_secrets::get_profile_secret(
                    &profile.id,
                    crate::ssh_secrets::SshSecretKind::KeyPassphrase,
                )?
            };

            if let Some(identity_ref) = profile.identity_ref.as_deref() {
                let key_material = crate::ssh_secrets::get_profile_secret(
                    &profile.id,
                    crate::ssh_secrets::SshSecretKind::IdentityKey,
                )?
                .ok_or_else(|| {
                    err_string(
                        codes::INTERNAL,
                        format!(
                            "SSH profile '{}' references identityRef '{}' but no key material is stored.",
                            profile.name, identity_ref
                        ),
                    )
                })?;

                let _ = key_material;
                // Keep the profile connection path deterministic: imported key material is represented
                // in identityFile as a synthetic label and resolved inside ssh_tunnel via default path
                // only when identityRef is absent.
                return Ok(NormalizedSshConnectArgs {
                    host: profile.host.clone(),
                    username: profile.username.clone(),
                    ssh_port: profile.ssh_port,
                    remote_port: profile.remote_port,
                    auth_mode: RemoteSshAuthMode::Key,
                    identity_file: profile
                        .identity_file
                        .clone()
                        .or_else(|| Some(format!("keychain:{identity_ref}"))),
                    password: None,
                    key_passphrase,
                    known_hosts_file: profile.known_hosts_file.clone(),
                });
            }

            Ok(NormalizedSshConnectArgs {
                host: profile.host.clone(),
                username: profile.username.clone(),
                ssh_port: profile.ssh_port,
                remote_port: profile.remote_port,
                auth_mode: RemoteSshAuthMode::Key,
                identity_file: profile.identity_file.clone(),
                password: None,
                key_passphrase,
                known_hosts_file: profile.known_hosts_file.clone(),
            })
        }
    }
}

async fn connect_profile_internal(
    app: AppHandle,
    state: &AppState,
    profile: &RemoteSshProfile,
    runtime_password: Option<String>,
    runtime_key_passphrase: Option<String>,
) -> RalphResult<RemoteSshConnectResult> {
    if profile.identity_ref.is_some() {
        let key_material = crate::ssh_secrets::get_profile_secret(
            &profile.id,
            crate::ssh_secrets::SshSecretKind::IdentityKey,
        )?;
        if profile.auth_mode == RemoteSshAuthMode::Key && key_material.is_some() {
            let key_material = key_material.expect("checked Some");
            let key_passphrase =
                if let Some(passphrase) = normalize_optional_trimmed(runtime_key_passphrase) {
                    Some(passphrase)
                } else {
                    crate::ssh_secrets::get_profile_secret(
                        &profile.id,
                        crate::ssh_secrets::SshSecretKind::KeyPassphrase,
                    )?
                };

            let base = NormalizedSshConnectArgs {
                host: profile.host.clone(),
                username: profile.username.clone(),
                ssh_port: profile.ssh_port,
                remote_port: profile.remote_port,
                auth_mode: RemoteSshAuthMode::Key,
                identity_file: None,
                password: None,
                key_passphrase: key_passphrase.clone(),
                known_hosts_file: profile.known_hosts_file.clone(),
            };

            if let Some(challenge) = register_unknown_host_challenge(state, &base).await? {
                return Err(err_string(
                    codes::INTERNAL,
                    format!(
                        "SSH host key for '{}:{}' is not trusted. Approve challenge '{}' and retry.",
                        challenge.host, challenge.ssh_port, challenge.challenge_id
                    ),
                )
                .with_context(
                    "ssh_hostkey_challenge",
                    serde_json::to_value(challenge).expect("challenge is serializable"),
                ));
            }

            disconnect_remote_and_tunnel(state).await?;

            let crate::ssh_tunnel::EmbeddedSshConnectOutcome {
                connection,
                stream,
                resolved_identity_file,
                resolved_known_hosts_file,
            } = crate::ssh_tunnel::connect(crate::ssh_tunnel::EmbeddedSshConnectArgs {
                host: profile.host.clone(),
                username: profile.username.clone(),
                ssh_port: profile.ssh_port,
                remote_port: profile.remote_port,
                auth_mode: RemoteSshAuthMode::Key,
                auth: crate::ssh_tunnel::EmbeddedSshAuth::KeyMaterial {
                    key_material,
                    passphrase: key_passphrase,
                    identity_label: profile
                        .identity_ref
                        .as_ref()
                        .map(|identity_ref| format!("keychain:{identity_ref}")),
                },
                known_hosts_file: profile.known_hosts_file.clone(),
            })
            .await?;

            let ws_url = format!("ws://127.0.0.1:{}", profile.remote_port);
            let protocol = match remote_connect_internal_with_stream(
                app,
                state,
                ws_url.clone(),
                stream,
            )
            .await
            {
                Ok(protocol) => protocol,
                Err(connect_error) => {
                    let teardown_result = connection.disconnect().await;
                    if let Err(teardown_error) = teardown_result {
                        return Err(err_string(
                            codes::INTERNAL,
                            format!(
                                "SSH remote connect failed ({connect_error}) and SSH tunnel teardown failed: {teardown_error}"
                            ),
                        ));
                    }
                    return Err(connect_error);
                }
            };

            let ssh_session_id = connection.session_id();
            let known_hosts_file = Some(resolved_known_hosts_file.clone());

            {
                let mut guard = state.ssh_tunnel.lock().await;
                if guard.is_some() {
                    disconnect_remote_transport(state).await?;
                    connection.disconnect().await?;
                    return Err(err_string(
                        codes::INTERNAL,
                        "SSH tunnel became active concurrently. Disconnect and retry.",
                    ));
                }

                *guard = Some(SshTunnelSession {
                    session_id: ssh_session_id,
                    connection,
                    host: profile.host.clone(),
                    username: profile.username.clone(),
                    ssh_port: profile.ssh_port,
                    remote_port: profile.remote_port,
                    auth_mode: RemoteSshAuthMode::Key,
                    transport_kind: RemoteSshTransportKind::Stream,
                    active_profile_id: Some(profile.id.clone()),
                    identity_file: resolved_identity_file.clone(),
                    known_hosts_file: known_hosts_file.clone(),
                });
            }

            return Ok(RemoteSshConnectResult {
                ws_url,
                protocol,
                ssh_session_id,
                host: profile.host.clone(),
                username: profile.username.clone(),
                ssh_port: profile.ssh_port,
                remote_port: profile.remote_port,
                auth_mode: RemoteSshAuthMode::Key,
                transport_kind: RemoteSshTransportKind::Stream,
                active_profile_id: Some(profile.id.clone()),
                identity_file: resolved_identity_file,
                known_hosts_file,
            });
        }
    }

    let connect_args =
        profile_connect_to_connect_args(profile, runtime_password, runtime_key_passphrase)?;
    connect_ssh_internal(app, state, connect_args, Some(profile.id.clone())).await
}

#[tauri::command]
pub async fn remote_connect(
    app: AppHandle,
    state: State<'_, AppState>,
    args: RemoteConnectArgs,
) -> RalphResult<RemoteConnectResult> {
    refresh_tunnel_liveness(state.inner()).await?;
    let protocol = remote_connect_internal(app, state.inner(), args.ws_url.clone()).await?;

    Ok(RemoteConnectResult {
        ws_url: args.ws_url,
        protocol,
    })
}

#[tauri::command]
pub async fn remote_disconnect(state: State<'_, AppState>) -> RalphResult<()> {
    disconnect_remote_and_tunnel(state.inner()).await
}

#[tauri::command]
pub async fn remote_status_get(state: State<'_, AppState>) -> RalphResult<RemoteStatus> {
    refresh_tunnel_liveness(state.inner()).await?;
    let guard = state.remote.lock().await;
    let status = guard
        .as_ref()
        .map_or_else(remote_status_disconnected, remote_status_connected);

    Ok(status)
}

#[tauri::command]
pub async fn remote_ssh_connect(
    app: AppHandle,
    state: State<'_, AppState>,
    args: RemoteSshConnectArgs,
) -> RalphResult<RemoteSshConnectResult> {
    refresh_tunnel_liveness(state.inner()).await?;
    let args = normalize_ssh_connect_args(args)?;
    connect_ssh_internal(app, state.inner(), args, None).await
}

#[tauri::command]
pub async fn remote_ssh_disconnect(state: State<'_, AppState>) -> RalphResult<()> {
    disconnect_remote_and_tunnel(state.inner()).await
}

#[tauri::command]
pub async fn remote_ssh_status_get(state: State<'_, AppState>) -> RalphResult<RemoteSshStatus> {
    refresh_tunnel_liveness(state.inner()).await?;
    let guard = state.ssh_tunnel.lock().await;
    let status = guard
        .as_ref()
        .map_or_else(remote_ssh_status_disconnected, remote_ssh_status_active);
    Ok(status)
}

#[tauri::command]
pub async fn remote_ssh_profile_list(
    app: AppHandle,
    state: State<'_, AppState>,
) -> RalphResult<Vec<RemoteSshProfile>> {
    load_profiles(&app, state.inner())
}

#[tauri::command]
pub async fn remote_ssh_profile_upsert(
    app: AppHandle,
    state: State<'_, AppState>,
    args: RemoteSshProfileUpsertArgs,
) -> RalphResult<RemoteSshProfile> {
    validate_profile_upsert_args(&args)?;

    let mut profiles = load_profiles(&app, state.inner())?;
    let previous_profiles = profiles.clone();
    let id = normalize_optional_trimmed(args.id.clone()).unwrap_or_else(generate_profile_id);

    let profile = RemoteSshProfile {
        id: id.clone(),
        name: args.name.trim().to_owned(),
        host: args.host.trim().to_owned(),
        username: args.username.trim().to_owned(),
        ssh_port: args.ssh_port,
        remote_port: args.remote_port,
        auth_mode: args.auth_mode,
        identity_file: normalize_optional_trimmed(args.identity_file),
        identity_ref: normalize_optional_trimmed(args.identity_ref),
        known_hosts_file: normalize_optional_trimmed(args.known_hosts_file),
        auto_reconnect_enabled: args.auto_reconnect_enabled,
        last_used_at: profiles
            .iter()
            .find(|existing| existing.id == id)
            .and_then(|existing| existing.last_used_at.clone()),
    };

    if let Some(index) = profile_index_by_id(&profiles, &id) {
        profiles[index] = profile.clone();
    } else {
        profiles.push(profile.clone());
    }

    save_profiles(&app, state.inner(), &profiles)?;

    let secret_result = (|| -> RalphResult<()> {
        match profile.auth_mode {
            RemoteSshAuthMode::Password => {
                if args.save_password {
                    let password = normalize_optional_trimmed(args.password).ok_or_else(|| {
                        err_string(
                            codes::INTERNAL,
                            "savePassword=true requires non-empty password.",
                        )
                    })?;
                    crate::ssh_secrets::set_profile_secret(
                        &profile.id,
                        crate::ssh_secrets::SshSecretKind::Password,
                        &password,
                    )?;
                } else {
                    crate::ssh_secrets::delete_profile_secret(
                        &profile.id,
                        crate::ssh_secrets::SshSecretKind::Password,
                    )?;
                }
                crate::ssh_secrets::delete_profile_secret(
                    &profile.id,
                    crate::ssh_secrets::SshSecretKind::KeyPassphrase,
                )?;
                crate::ssh_secrets::delete_profile_secret(
                    &profile.id,
                    crate::ssh_secrets::SshSecretKind::IdentityKey,
                )?;
            }
            RemoteSshAuthMode::Key => {
                crate::ssh_secrets::delete_profile_secret(
                    &profile.id,
                    crate::ssh_secrets::SshSecretKind::Password,
                )?;

                if args.save_key_passphrase {
                    let passphrase =
                        normalize_optional_trimmed(args.key_passphrase).ok_or_else(|| {
                            err_string(
                                codes::INTERNAL,
                                "saveKeyPassphrase=true requires non-empty keyPassphrase.",
                            )
                        })?;
                    crate::ssh_secrets::set_profile_secret(
                        &profile.id,
                        crate::ssh_secrets::SshSecretKind::KeyPassphrase,
                        &passphrase,
                    )?;
                } else {
                    crate::ssh_secrets::delete_profile_secret(
                        &profile.id,
                        crate::ssh_secrets::SshSecretKind::KeyPassphrase,
                    )?;
                }

                if profile.identity_ref.is_none() {
                    crate::ssh_secrets::delete_profile_secret(
                        &profile.id,
                        crate::ssh_secrets::SshSecretKind::IdentityKey,
                    )?;
                }
            }
        }
        Ok(())
    })();

    if let Err(secret_error) = secret_result {
        if let Err(rollback_error) = save_profiles(&app, state.inner(), &previous_profiles) {
            return Err(err_string(
                codes::INTERNAL,
                format!(
                    "SSH profile upsert secret persistence failed ({secret_error}) and profile rollback failed: {rollback_error}"
                ),
            ));
        }
        return Err(secret_error);
    }

    Ok(profile)
}

#[tauri::command]
pub async fn remote_ssh_profile_delete(
    app: AppHandle,
    state: State<'_, AppState>,
    args: RemoteSshProfileDeleteArgs,
) -> RalphResult<()> {
    let profile_id = normalize_required_trimmed(args.profile_id, "profile_id")?;

    let mut profiles = load_profiles(&app, state.inner())?;
    let before_len = profiles.len();
    profiles.retain(|profile| profile.id != profile_id);

    if profiles.len() == before_len {
        return Err(err_string(
            codes::INTERNAL,
            format!("SSH profile '{}' does not exist.", profile_id),
        ));
    }

    save_profiles(&app, state.inner(), &profiles)?;
    crate::ssh_secrets::delete_all_profile_secrets(&profile_id)?;

    let should_disconnect_active = {
        let guard = state.ssh_tunnel.lock().await;
        guard
            .as_ref()
            .and_then(|session| session.active_profile_id.as_deref())
            .is_some_and(|active_profile_id| active_profile_id == profile_id.as_str())
    };
    if should_disconnect_active {
        disconnect_remote_and_tunnel(state.inner()).await?;
    }

    Ok(())
}

#[tauri::command]
pub async fn remote_ssh_profile_set_last_used(
    app: AppHandle,
    state: State<'_, AppState>,
    args: RemoteSshProfileSetLastUsedArgs,
) -> RalphResult<()> {
    let profile_id = normalize_required_trimmed(args.profile_id, "profile_id")?;
    let mut profiles = load_profiles(&app, state.inner())?;

    let now = utc_now_rfc3339();
    let mut found = false;
    for profile in &mut profiles {
        if profile.id == profile_id {
            profile.last_used_at = Some(now.clone());
            found = true;
        }
    }

    if !found {
        return Err(err_string(
            codes::INTERNAL,
            format!("SSH profile '{}' does not exist.", profile_id),
        ));
    }

    save_profiles(&app, state.inner(), &profiles)
}

#[tauri::command]
pub async fn remote_ssh_profile_connect(
    app: AppHandle,
    state: State<'_, AppState>,
    args: RemoteSshProfileConnectArgs,
) -> RalphResult<RemoteSshConnectResult> {
    refresh_tunnel_liveness(state.inner()).await?;

    let profile_id = normalize_required_trimmed(args.profile_id, "profile_id")?;
    let mut profiles = load_profiles(&app, state.inner())?;
    let profile_index = profile_index_by_id(&profiles, &profile_id).ok_or_else(|| {
        err_string(
            codes::INTERNAL,
            format!("SSH profile '{}' does not exist.", profile_id),
        )
    })?;

    let profile = profiles[profile_index].clone();
    let connect_result = connect_profile_internal(
        app.clone(),
        state.inner(),
        &profile,
        args.password,
        args.key_passphrase,
    )
    .await?;

    profiles[profile_index].last_used_at = Some(utc_now_rfc3339());
    save_profiles(&app, state.inner(), &profiles)?;

    Ok(connect_result)
}

#[tauri::command]
pub async fn remote_ssh_identity_import(
    app: AppHandle,
    state: State<'_, AppState>,
    args: RemoteSshIdentityImportArgs,
) -> RalphResult<RemoteSshIdentityImportResult> {
    let profile_id = normalize_required_trimmed(args.profile_id, "profile_id")?;
    let file_name = normalize_required_trimmed(args.file_name, "file_name")?;

    let decoded = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        args.bytes_base64.as_bytes(),
    )
    .map_err(|error| {
        err_string(
            codes::INTERNAL,
            format!("Failed to decode imported SSH key bytes: {error}"),
        )
    })?;

    let key_material = String::from_utf8(decoded).map_err(|error| {
        err_string(
            codes::INTERNAL,
            format!("Imported SSH key bytes are not UTF-8 text: {error}"),
        )
    })?;

    let passphrase = normalize_optional_trimmed(args.passphrase);
    if args.save_passphrase && passphrase.is_none() {
        return Err(err_string(
            codes::INTERNAL,
            "savePassphrase=true requires non-empty passphrase.",
        ));
    }

    let _validated_key = russh::keys::decode_secret_key(&key_material, passphrase.as_deref())
        .map_err(|error| {
            err_string(
                codes::INTERNAL,
                format!("Imported SSH key is invalid or passphrase is incorrect: {error}"),
            )
        })?;

    let mut profiles = load_profiles(&app, state.inner())?;
    let previous_profiles = profiles.clone();
    let profile_index = profile_index_by_id(&profiles, &profile_id).ok_or_else(|| {
        err_string(
            codes::INTERNAL,
            format!("SSH profile '{}' does not exist.", profile_id),
        )
    })?;

    let identity_ref = format!("keychain:{}:{}", profile_id, file_name);
    profiles[profile_index].identity_ref = Some(identity_ref.clone());
    profiles[profile_index].auth_mode = RemoteSshAuthMode::Key;

    save_profiles(&app, state.inner(), &profiles)?;

    let secret_result = (|| -> RalphResult<()> {
        crate::ssh_secrets::set_profile_secret(
            &profile_id,
            crate::ssh_secrets::SshSecretKind::IdentityKey,
            &key_material,
        )?;
        crate::ssh_secrets::delete_profile_secret(
            &profile_id,
            crate::ssh_secrets::SshSecretKind::Password,
        )?;

        if args.save_passphrase {
            let passphrase = passphrase.expect("passphrase pre-validated for savePassphrase=true");
            crate::ssh_secrets::set_profile_secret(
                &profile_id,
                crate::ssh_secrets::SshSecretKind::KeyPassphrase,
                &passphrase,
            )?;
        } else {
            crate::ssh_secrets::delete_profile_secret(
                &profile_id,
                crate::ssh_secrets::SshSecretKind::KeyPassphrase,
            )?;
        }

        Ok(())
    })();

    if let Err(secret_error) = secret_result {
        if let Err(rollback_error) = save_profiles(&app, state.inner(), &previous_profiles) {
            return Err(err_string(
                codes::INTERNAL,
                format!(
                    "SSH identity import secret persistence failed ({secret_error}) and profile rollback failed: {rollback_error}"
                ),
            ));
        }
        return Err(secret_error);
    }

    Ok(RemoteSshIdentityImportResult { identity_ref })
}

#[tauri::command]
pub async fn remote_ssh_hostkey_challenge_approve(
    state: State<'_, AppState>,
    args: RemoteSshHostKeyChallengeApproveArgs,
) -> RalphResult<()> {
    let challenge_id = normalize_required_trimmed(args.challenge_id, "challenge_id")?;

    let pending = {
        let mut challenges = state.ssh_host_key_challenges.lock().await;
        cleanup_expired_challenges(&mut challenges);
        challenges.remove(&challenge_id)
    }
    .ok_or_else(|| {
        err_string(
            codes::INTERNAL,
            format!(
                "Unknown or expired SSH host-key challenge '{}'.",
                challenge_id
            ),
        )
    })?;

    if pending.expires_at <= Instant::now() {
        return Err(err_string(
            codes::INTERNAL,
            format!("SSH host-key challenge '{}' has expired.", challenge_id),
        ));
    }

    if let Some(parent) = pending.known_hosts_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            err_string(
                codes::FILESYSTEM,
                format!(
                    "Failed to create known_hosts directory '{}': {error}",
                    parent.display()
                ),
            )
        })?;
    }

    russh::keys::known_hosts::learn_known_hosts_path(
        &pending.challenge.host,
        pending.challenge.ssh_port,
        &pending.server_public_key,
        &pending.known_hosts_path,
    )
    .map_err(|error| {
        err_string(
            codes::INTERNAL,
            format!(
                "Failed to write approved SSH host key to '{}': {error}",
                pending.known_hosts_path.display()
            ),
        )
    })?;

    Ok(())
}

#[tauri::command]
pub async fn remote_ssh_hostkey_challenge_reject(
    state: State<'_, AppState>,
    args: RemoteSshHostKeyChallengeRejectArgs,
) -> RalphResult<()> {
    let challenge_id = normalize_required_trimmed(args.challenge_id, "challenge_id")?;

    let removed = {
        let mut challenges = state.ssh_host_key_challenges.lock().await;
        cleanup_expired_challenges(&mut challenges);
        challenges.remove(&challenge_id)
    };

    if removed.is_none() {
        return Err(err_string(
            codes::INTERNAL,
            format!(
                "Unknown or expired SSH host-key challenge '{}'.",
                challenge_id
            ),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_ssh_connect_args, validate_loopback_ws_url, validate_profile_upsert_args,
    };
    use core_contracts::remote::{
        RemoteSshAuthMode, RemoteSshConnectArgs, RemoteSshProfileUpsertArgs,
    };

    fn valid_profile_upsert_args(auth_mode: RemoteSshAuthMode) -> RemoteSshProfileUpsertArgs {
        RemoteSshProfileUpsertArgs {
            id: None,
            name: "Home".to_owned(),
            host: "build-box.local".to_owned(),
            username: "vince".to_owned(),
            ssh_port: 22,
            remote_port: 9944,
            auth_mode,
            identity_file: None,
            identity_ref: None,
            known_hosts_file: None,
            auto_reconnect_enabled: false,
            password: None,
            key_passphrase: None,
            save_password: false,
            save_key_passphrase: false,
        }
    }

    #[test]
    fn accepts_ipv4_loopback_ws() {
        validate_loopback_ws_url("ws://127.0.0.1:9944")
            .expect("expected loopback url to be accepted");
    }

    #[test]
    fn accepts_ipv6_loopback_ws() {
        validate_loopback_ws_url("ws://[::1]:9944").expect("expected loopback url to be accepted");
    }

    #[test]
    fn rejects_non_loopback_host() {
        let error = validate_loopback_ws_url("ws://192.168.1.10:9944")
            .expect_err("expected non-loopback url to be rejected");
        assert!(
            error.to_string().contains("Create an SSH tunnel first"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn rejects_localhost_domain_alias() {
        let error = validate_loopback_ws_url("ws://localhost:9944")
            .expect_err("expected localhost domain alias to be rejected");
        assert!(
            error
                .to_string()
                .contains("only ws://127.0.0.1:<port> or ws://[::1]:<port>"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn rejects_non_ws_scheme() {
        let error = validate_loopback_ws_url("wss://127.0.0.1:9944")
            .expect_err("expected wss url to be rejected");
        assert!(
            error
                .to_string()
                .contains("SSH tunnel mode only allows plain ws://"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn rejects_missing_explicit_port() {
        let error = validate_loopback_ws_url("ws://127.0.0.1")
            .expect_err("expected url without explicit port to be rejected");
        assert!(
            error.to_string().contains("explicit port is required"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn normalize_ssh_connect_args_key_auth() {
        let normalized = normalize_ssh_connect_args(RemoteSshConnectArgs {
            host: "  build-box.local  ".to_owned(),
            username: "  vince  ".to_owned(),
            ssh_port: 22,
            remote_port: 9944,
            auth_mode: RemoteSshAuthMode::Key,
            identity_file: Some("  /Users/vince/.ssh/id_ed25519  ".to_owned()),
            password: None,
            key_passphrase: Some("  passphrase ".to_owned()),
            known_hosts_file: Some("  /Users/vince/.ssh/known_hosts  ".to_owned()),
        })
        .expect("expected args to normalize");

        assert_eq!(normalized.host, "build-box.local");
        assert_eq!(normalized.username, "vince");
        assert_eq!(normalized.ssh_port, 22);
        assert_eq!(normalized.remote_port, 9944);
        assert_eq!(
            normalized.identity_file,
            Some("/Users/vince/.ssh/id_ed25519".to_owned())
        );
        assert_eq!(normalized.password, None);
        assert_eq!(normalized.key_passphrase, Some("passphrase".to_owned()));
        assert_eq!(
            normalized.known_hosts_file,
            Some("/Users/vince/.ssh/known_hosts".to_owned())
        );
    }

    #[test]
    fn normalize_ssh_connect_args_password_auth_requires_password() {
        let error = normalize_ssh_connect_args(RemoteSshConnectArgs {
            host: "build-box.local".to_owned(),
            username: "vince".to_owned(),
            ssh_port: 22,
            remote_port: 9944,
            auth_mode: RemoteSshAuthMode::Password,
            identity_file: None,
            password: None,
            key_passphrase: None,
            known_hosts_file: None,
        })
        .expect_err("expected missing password to fail");

        assert!(
            error.to_string().contains("requires a non-empty password"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn normalize_ssh_connect_args_password_auth_rejects_identity_file() {
        let error = normalize_ssh_connect_args(RemoteSshConnectArgs {
            host: "build-box.local".to_owned(),
            username: "vince".to_owned(),
            ssh_port: 22,
            remote_port: 9944,
            auth_mode: RemoteSshAuthMode::Password,
            identity_file: Some("/tmp/id_ed25519".to_owned()),
            password: Some("pw".to_owned()),
            key_passphrase: None,
            known_hosts_file: None,
        })
        .expect_err("expected identityFile in password mode to fail");

        assert!(
            error.to_string().contains("identityFile must be omitted"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn validate_profile_upsert_rejects_password_when_key_mode() {
        let mut args = valid_profile_upsert_args(RemoteSshAuthMode::Key);
        args.password = Some("secret".to_owned());

        let error = validate_profile_upsert_args(&args)
            .expect_err("expected key mode to reject non-empty password");
        assert!(
            error
                .to_string()
                .contains("password must be empty when authMode='key'"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn validate_profile_upsert_rejects_key_passphrase_when_password_mode() {
        let mut args = valid_profile_upsert_args(RemoteSshAuthMode::Password);
        args.key_passphrase = Some("secret".to_owned());

        let error = validate_profile_upsert_args(&args)
            .expect_err("expected password mode to reject key passphrase");
        assert!(
            error
                .to_string()
                .contains("keyPassphrase must be empty when authMode='password'"),
            "unexpected error: {error}"
        );
    }
}
