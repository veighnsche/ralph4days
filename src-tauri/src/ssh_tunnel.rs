use core_contracts::remote::RemoteSshAuthMode;
use core_errors::{codes, err_string, RalphResult};
use russh::client::{self, AuthResult, Handler};
use russh::keys::{self, HashAlg, PrivateKeyWithHashAlg};
use russh::Disconnect;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const DEFAULT_SSH_HOME_DIR: &str = ".ssh";
const DEFAULT_KNOWN_HOSTS_FILENAME: &str = "known_hosts";
const DEFAULT_IDENTITY_FILENAMES: &[&str] = &["id_ed25519", "id_ecdsa", "id_rsa"];

static NEXT_SSH_SESSION_ID: AtomicU32 = AtomicU32::new(1);

pub(crate) type EmbeddedSshStream = russh::ChannelStream<russh::client::Msg>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EmbeddedSshAuth {
    KeyFile {
        identity_file: Option<String>,
        passphrase: Option<String>,
    },
    KeyMaterial {
        key_material: String,
        passphrase: Option<String>,
        identity_label: Option<String>,
    },
    Password {
        password: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EmbeddedSshConnectArgs {
    pub host: String,
    pub username: String,
    pub ssh_port: u16,
    pub remote_port: u16,
    pub auth_mode: RemoteSshAuthMode,
    pub auth: EmbeddedSshAuth,
    pub known_hosts_file: Option<String>,
}

pub(crate) struct EmbeddedSshConnectOutcome {
    pub connection: EmbeddedSshConnection,
    pub stream: EmbeddedSshStream,
    pub resolved_identity_file: Option<String>,
    pub resolved_known_hosts_file: String,
}

pub(crate) struct UnknownHostKeyProbe {
    pub known_hosts_path: PathBuf,
    pub server_public_key: russh::keys::ssh_key::PublicKey,
    pub algorithm: String,
    pub fingerprint_sha256: String,
}

pub(crate) struct EmbeddedSshConnection {
    session_id: u32,
    handle: russh::client::Handle<StrictKnownHostsHandler>,
}

impl EmbeddedSshConnection {
    pub fn session_id(&self) -> u32 {
        self.session_id
    }

    pub fn is_closed(&self) -> bool {
        self.handle.is_closed()
    }

    pub async fn disconnect(&self) -> RalphResult<()> {
        let result = self
            .handle
            .disconnect(
                Disconnect::ByApplication,
                "Ralph SSH tunnel disconnect",
                "en",
            )
            .await;
        match result {
            Ok(()) => Ok(()),
            Err(error)
                if matches!(
                    error,
                    russh::Error::SendError
                        | russh::Error::Disconnect
                        | russh::Error::HUP
                        | russh::Error::ConnectionTimeout
                        | russh::Error::KeepaliveTimeout
                        | russh::Error::InactivityTimeout
                ) =>
            {
                Ok(())
            }
            Err(error) => Err(err_string(
                codes::INTERNAL,
                format!("Failed to disconnect embedded SSH session cleanly: {error}"),
            )),
        }
    }
}

#[derive(Clone)]
struct StrictKnownHostsHandler {
    host: String,
    ssh_port: u16,
    known_hosts_path: PathBuf,
}

impl Handler for StrictKnownHostsHandler {
    type Error = russh::Error;

    fn check_server_key(
        &mut self,
        server_public_key: &russh::keys::ssh_key::PublicKey,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send {
        let check = keys::check_known_hosts_path(
            &self.host,
            self.ssh_port,
            server_public_key,
            &self.known_hosts_path,
        )
        .map_err(russh::Error::from);
        std::future::ready(check)
    }
}

#[derive(Clone)]
struct ProbeHostKeyHandler {
    captured_key: Arc<Mutex<Option<russh::keys::ssh_key::PublicKey>>>,
}

impl Handler for ProbeHostKeyHandler {
    type Error = russh::Error;

    fn check_server_key(
        &mut self,
        server_public_key: &russh::keys::ssh_key::PublicKey,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send {
        *self.captured_key.lock().expect("probe key mutex poisoned") =
            Some(server_public_key.clone());
        std::future::ready(Ok(false))
    }
}

pub(crate) async fn probe_unknown_host_key(
    host: &str,
    ssh_port: u16,
    known_hosts_file: Option<&str>,
) -> RalphResult<Option<UnknownHostKeyProbe>> {
    let known_hosts_path = resolve_known_hosts_path(known_hosts_file)?;
    let server_public_key = capture_server_public_key(host, ssh_port).await?;

    let is_trusted =
        keys::check_known_hosts_path(host, ssh_port, &server_public_key, &known_hosts_path)
            .map_err(|error| {
                err_string(
                    codes::INTERNAL,
                    format!(
                        "Failed to verify SSH host key trust in known_hosts '{}': {error}",
                        known_hosts_path.display()
                    ),
                )
            })?;

    if is_trusted {
        return Ok(None);
    }

    let algorithm = server_public_key.algorithm().to_string();
    let fingerprint_sha256 = format!(
        "{}",
        server_public_key.fingerprint(russh::keys::HashAlg::Sha256)
    );

    Ok(Some(UnknownHostKeyProbe {
        known_hosts_path,
        server_public_key,
        algorithm,
        fingerprint_sha256,
    }))
}

async fn capture_server_public_key(
    host: &str,
    ssh_port: u16,
) -> RalphResult<russh::keys::ssh_key::PublicKey> {
    let captured_key: Arc<Mutex<Option<russh::keys::ssh_key::PublicKey>>> =
        Arc::new(Mutex::new(None));
    let mut config = client::Config::default();
    config.keepalive_interval = Some(Duration::from_secs(5));
    config.keepalive_max = 1;

    let connect_result = client::connect(
        Arc::new(config),
        (host, ssh_port),
        ProbeHostKeyHandler {
            captured_key: Arc::clone(&captured_key),
        },
    )
    .await;

    let connect_detail = match &connect_result {
        Ok(_) => "probe completed without server key".to_owned(),
        Err(error) => format!("{error}"),
    };

    if let Ok(handle) = connect_result {
        let _ = handle
            .disconnect(
                Disconnect::ByApplication,
                "Ralph SSH probe disconnect",
                "en",
            )
            .await;
    }

    let key = captured_key
        .lock()
        .expect("probe key mutex poisoned")
        .clone();
    let Some(server_public_key) = key else {
        return Err(err_string(
            codes::INTERNAL,
            format!(
                "Failed to capture SSH host key for '{}:{}' while probing server key: {detail}",
                host,
                ssh_port,
                detail = connect_detail
            ),
        ));
    };

    Ok(server_public_key)
}

pub(crate) async fn connect(
    args: EmbeddedSshConnectArgs,
) -> RalphResult<EmbeddedSshConnectOutcome> {
    let known_hosts_path = resolve_known_hosts_path(args.known_hosts_file.as_deref())?;
    ensure_known_host_entry_exists(&args.host, args.ssh_port, &known_hosts_path)?;

    let mut config = client::Config::default();
    config.keepalive_interval = Some(Duration::from_secs(15));
    config.keepalive_max = 3;

    let mut handle = client::connect(
        Arc::new(config),
        (args.host.as_str(), args.ssh_port),
        StrictKnownHostsHandler {
            host: args.host.clone(),
            ssh_port: args.ssh_port,
            known_hosts_path: known_hosts_path.clone(),
        },
    )
    .await
    .map_err(|error| {
        err_string(
            codes::INTERNAL,
            format!(
                "Failed to establish SSH transport to {}@{}:{}: {error}",
                args.username, args.host, args.ssh_port
            ),
        )
    })?;

    let resolved_identity_file =
        authenticate(&mut handle, &args.username, &args.auth_mode, args.auth).await?;

    let channel = handle
        .channel_open_direct_tcpip("127.0.0.1", u32::from(args.remote_port), "127.0.0.1", 0)
        .await
        .map_err(|error| {
            err_string(
                codes::INTERNAL,
                format!(
                    "Failed to open SSH direct-tcpip channel to 127.0.0.1:{} on remote host '{}': {error}",
                    args.remote_port, args.host
                ),
            )
        })?;

    let stream = channel.into_stream();
    let session_id = allocate_ssh_session_id();
    let resolved_known_hosts_file = known_hosts_path.to_string_lossy().into_owned();

    Ok(EmbeddedSshConnectOutcome {
        connection: EmbeddedSshConnection { session_id, handle },
        stream,
        resolved_identity_file,
        resolved_known_hosts_file,
    })
}

fn allocate_ssh_session_id() -> u32 {
    let session_id = NEXT_SSH_SESSION_ID.fetch_add(1, Ordering::Relaxed);
    assert!(session_id != 0, "SSH session id allocator wrapped to 0");
    session_id
}

pub(crate) fn resolve_known_hosts_path(known_hosts_file: Option<&str>) -> RalphResult<PathBuf> {
    if let Some(path) = known_hosts_file {
        let path = PathBuf::from(path);
        if path.as_os_str().is_empty() {
            return Err(err_string(
                codes::INTERNAL,
                "SSH known_hosts path cannot be empty when provided.",
            ));
        }
        return Ok(path);
    }

    let home = resolve_home_dir()?;
    Ok(home
        .join(DEFAULT_SSH_HOME_DIR)
        .join(DEFAULT_KNOWN_HOSTS_FILENAME))
}

fn ensure_known_host_entry_exists(
    host: &str,
    ssh_port: u16,
    known_hosts_path: &Path,
) -> RalphResult<()> {
    let entries = russh::keys::known_hosts::known_host_keys_path(host, ssh_port, known_hosts_path)
        .map_err(|error| {
            err_string(
                codes::INTERNAL,
                format!(
                    "Failed to read SSH known_hosts file '{}': {error}",
                    known_hosts_path.display()
                ),
            )
        })?;

    if entries.is_empty() {
        return Err(err_string(
            codes::INTERNAL,
            format!(
                "Strict SSH host verification failed: no known_hosts entry for '{}:{}' in '{}'. Add the host key first.",
                host,
                ssh_port,
                known_hosts_path.display()
            ),
        ));
    }

    Ok(())
}

async fn authenticate(
    handle: &mut russh::client::Handle<StrictKnownHostsHandler>,
    username: &str,
    auth_mode: &RemoteSshAuthMode,
    auth: EmbeddedSshAuth,
) -> RalphResult<Option<String>> {
    match (auth_mode, auth) {
        (
            RemoteSshAuthMode::Key,
            EmbeddedSshAuth::KeyFile {
                identity_file,
                passphrase,
            },
        ) => {
            let identity_path = resolve_identity_path(identity_file.as_deref())?;
            let key = load_private_key_from_file(&identity_path, passphrase.as_deref())?;
            authenticate_with_identity(handle, username, key).await?;
            Ok(Some(identity_path.to_string_lossy().into_owned()))
        }
        (
            RemoteSshAuthMode::Key,
            EmbeddedSshAuth::KeyMaterial {
                key_material,
                passphrase,
                identity_label,
            },
        ) => {
            let key = load_private_key_from_material(&key_material, passphrase.as_deref())?;
            authenticate_with_identity(handle, username, key).await?;
            Ok(identity_label)
        }
        (RemoteSshAuthMode::Password, EmbeddedSshAuth::Password { password }) => {
            authenticate_with_password(handle, username, &password).await?;
            Ok(None)
        }
        _ => Err(err_string(
            codes::INTERNAL,
            "SSH auth mode does not match provided auth payload.",
        )),
    }
}

fn resolve_identity_path(identity_file: Option<&str>) -> RalphResult<PathBuf> {
    if let Some(path) = identity_file {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
        return Err(err_string(
            codes::INTERNAL,
            format!(
                "SSH identity file '{}' does not exist or is not a file.",
                path.display()
            ),
        ));
    }

    let home = resolve_home_dir()?;
    let ssh_dir = home.join(DEFAULT_SSH_HOME_DIR);
    for candidate in DEFAULT_IDENTITY_FILENAMES {
        let path = ssh_dir.join(candidate);
        if path.is_file() {
            return Ok(path);
        }
    }

    Err(err_string(
        codes::INTERNAL,
        format!(
            "No default SSH identity file found in '{}'. Provide identityFile explicitly.",
            ssh_dir.display()
        ),
    ))
}

fn resolve_home_dir() -> RalphResult<PathBuf> {
    let Some(home) = std::env::var_os("HOME") else {
        return Err(err_string(
            codes::INTERNAL,
            "HOME is not set. Provide explicit identityFile and knownHostsFile.",
        ));
    };
    Ok(PathBuf::from(home))
}

fn load_private_key_from_file(
    identity_path: &Path,
    passphrase: Option<&str>,
) -> RalphResult<Arc<keys::PrivateKey>> {
    keys::load_secret_key(identity_path, passphrase)
        .map(Arc::new)
        .map_err(|error| match error {
            keys::Error::KeyIsEncrypted => err_string(
                codes::INTERNAL,
                format!(
                    "SSH identity file '{}' is encrypted. Provide key passphrase.",
                    identity_path.display()
                ),
            ),
            _ => err_string(
                codes::INTERNAL,
                format!(
                    "Failed to load SSH identity file '{}': {error}",
                    identity_path.display()
                ),
            ),
        })
}

fn load_private_key_from_material(
    key_material: &str,
    passphrase: Option<&str>,
) -> RalphResult<Arc<keys::PrivateKey>> {
    keys::decode_secret_key(key_material, passphrase)
        .map(Arc::new)
        .map_err(|error| match error {
            keys::Error::KeyIsEncrypted => err_string(
                codes::INTERNAL,
                "Imported SSH private key is encrypted. Provide key passphrase.",
            ),
            _ => err_string(
                codes::INTERNAL,
                format!("Failed to decode imported SSH private key: {error}"),
            ),
        })
}

async fn authenticate_with_identity(
    handle: &mut russh::client::Handle<StrictKnownHostsHandler>,
    username: &str,
    key: Arc<keys::PrivateKey>,
) -> RalphResult<()> {
    if key.algorithm().is_rsa() {
        authenticate_rsa_key(handle, username, key).await
    } else {
        authenticate_once(handle, username, key, None).await
    }
}

async fn authenticate_rsa_key(
    handle: &mut russh::client::Handle<StrictKnownHostsHandler>,
    username: &str,
    key: Arc<keys::PrivateKey>,
) -> RalphResult<()> {
    for hash_alg in [Some(HashAlg::Sha512), Some(HashAlg::Sha256)] {
        let result = handle
            .authenticate_publickey(
                username.to_owned(),
                PrivateKeyWithHashAlg::new(Arc::clone(&key), hash_alg),
            )
            .await
            .map_err(|error| {
                err_string(
                    codes::INTERNAL,
                    format!("SSH public-key authentication failed for user '{username}': {error}"),
                )
            })?;
        if result.success() {
            return Ok(());
        }
    }

    Err(err_string(
        codes::INTERNAL,
        format!(
            "SSH authentication rejected for user '{username}' with RSA key (tried rsa-sha2-512 and rsa-sha2-256)."
        ),
    ))
}

async fn authenticate_once(
    handle: &mut russh::client::Handle<StrictKnownHostsHandler>,
    username: &str,
    key: Arc<keys::PrivateKey>,
    hash_alg: Option<HashAlg>,
) -> RalphResult<()> {
    let result = handle
        .authenticate_publickey(
            username.to_owned(),
            PrivateKeyWithHashAlg::new(Arc::clone(&key), hash_alg),
        )
        .await
        .map_err(|error| {
            err_string(
                codes::INTERNAL,
                format!("SSH public-key authentication failed for user '{username}': {error}"),
            )
        })?;
    match result {
        AuthResult::Success => Ok(()),
        AuthResult::Failure {
            remaining_methods,
            partial_success,
        } => Err(err_string(
            codes::INTERNAL,
            format!(
                "SSH authentication rejected for user '{username}' (partialSuccess={partial_success}, remainingMethods={remaining_methods:?})."
            ),
        )),
    }
}

async fn authenticate_with_password(
    handle: &mut russh::client::Handle<StrictKnownHostsHandler>,
    username: &str,
    password: &str,
) -> RalphResult<()> {
    let result = handle
        .authenticate_password(username.to_owned(), password.to_owned())
        .await
        .map_err(|error| {
            err_string(
                codes::INTERNAL,
                format!("SSH password authentication failed for user '{username}': {error}"),
            )
        })?;

    match result {
        AuthResult::Success => Ok(()),
        AuthResult::Failure {
            remaining_methods,
            partial_success,
        } => Err(err_string(
            codes::INTERNAL,
            format!(
                "SSH password authentication rejected for user '{username}' (partialSuccess={partial_success}, remainingMethods={remaining_methods:?})."
            ),
        )),
    }
}
