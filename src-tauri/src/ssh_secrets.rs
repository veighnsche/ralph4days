use core_errors::{codes, err_string, RalphResult};

const SECRET_SERVICE_NAME: &str = "com.vince.ralph.ssh";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SshSecretKind {
    Password,
    IdentityKey,
    KeyPassphrase,
}

impl SshSecretKind {
    fn suffix(self) -> &'static str {
        match self {
            Self::Password => "password",
            Self::IdentityKey => "identity-key",
            Self::KeyPassphrase => "key-passphrase",
        }
    }
}

fn account_name(profile_id: &str, kind: SshSecretKind) -> String {
    format!("profile:{profile_id}:{}", kind.suffix())
}

#[cfg(any(target_os = "ios", target_os = "macos"))]
fn keychain_set(account: &str, secret: &str) -> RalphResult<()> {
    security_framework::passwords::set_generic_password(
        SECRET_SERVICE_NAME,
        account,
        secret.as_bytes(),
    )
    .map_err(|error| {
        err_string(
            codes::INTERNAL,
            format!("Failed to write SSH secret '{account}' to keychain: {error}"),
        )
    })?;
    Ok(())
}

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
fn keychain_set(account: &str, _secret: &str) -> RalphResult<()> {
    Err(err_string(
        codes::INTERNAL,
        format!(
            "Persisted SSH secrets are unsupported on this platform (requested account '{account}')."
        ),
    ))
}

#[cfg(any(target_os = "ios", target_os = "macos"))]
fn keychain_get(account: &str) -> RalphResult<Option<String>> {
    match security_framework::passwords::get_generic_password(SECRET_SERVICE_NAME, account) {
        Ok(secret) => {
            let text = String::from_utf8(secret).map_err(|error| {
                err_string(
                    codes::INTERNAL,
                    format!("SSH secret '{account}' in keychain is not valid UTF-8: {error}"),
                )
            })?;
            Ok(Some(text))
        }
        Err(error) if error.code() == security_framework_sys::base::errSecItemNotFound => Ok(None),
        Err(error) => Err(err_string(
            codes::INTERNAL,
            format!("Failed to read SSH secret '{account}' from keychain: {error}"),
        )),
    }
}

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
fn keychain_get(_account: &str) -> RalphResult<Option<String>> {
    Ok(None)
}

#[cfg(any(target_os = "ios", target_os = "macos"))]
fn keychain_delete(account: &str) -> RalphResult<()> {
    match security_framework::passwords::delete_generic_password(SECRET_SERVICE_NAME, account) {
        Ok(()) => Ok(()),
        Err(error) if error.code() == security_framework_sys::base::errSecItemNotFound => Ok(()),
        Err(error) => Err(err_string(
            codes::INTERNAL,
            format!("Failed to delete SSH secret '{account}' from keychain: {error}"),
        )),
    }
}

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
fn keychain_delete(_account: &str) -> RalphResult<()> {
    Ok(())
}

pub(crate) fn set_profile_secret(
    profile_id: &str,
    kind: SshSecretKind,
    secret: &str,
) -> RalphResult<()> {
    keychain_set(&account_name(profile_id, kind), secret)
}

pub(crate) fn get_profile_secret(
    profile_id: &str,
    kind: SshSecretKind,
) -> RalphResult<Option<String>> {
    keychain_get(&account_name(profile_id, kind))
}

pub(crate) fn delete_profile_secret(profile_id: &str, kind: SshSecretKind) -> RalphResult<()> {
    keychain_delete(&account_name(profile_id, kind))
}

pub(crate) fn delete_all_profile_secrets(profile_id: &str) -> RalphResult<()> {
    delete_profile_secret(profile_id, SshSecretKind::Password)?;
    delete_profile_secret(profile_id, SshSecretKind::IdentityKey)?;
    delete_profile_secret(profile_id, SshSecretKind::KeyPassphrase)?;
    Ok(())
}
