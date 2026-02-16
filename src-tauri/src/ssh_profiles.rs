use core_contracts::remote::RemoteSshProfile;
use core_errors::{codes, err_string, RalphResult, RalphResultExt};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub(crate) const PROFILE_STORE_FILENAME: &str = "ssh_profiles.v1.json";
const PROFILE_STORE_VERSION: u16 = 1;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SshProfileStoreV1 {
    version: u16,
    profiles: Vec<RemoteSshProfile>,
}

pub(crate) fn load_profiles(store_path: &Path) -> RalphResult<Vec<RemoteSshProfile>> {
    if !store_path.exists() {
        return Ok(Vec::new());
    }

    let text = std::fs::read_to_string(store_path).map_err(|error| {
        err_string(
            codes::FILESYSTEM,
            format!(
                "Failed to read SSH profile store '{}': {error}",
                store_path.display()
            ),
        )
    })?;

    let parsed: SshProfileStoreV1 = serde_json::from_str(&text).map_err(|error| {
        err_string(
            codes::FILESYSTEM,
            format!(
                "Failed to parse SSH profile store '{}': {error}",
                store_path.display()
            ),
        )
    })?;

    if parsed.version != PROFILE_STORE_VERSION {
        return Err(err_string(
            codes::FILESYSTEM,
            format!(
                "Unsupported SSH profile store version '{}' in '{}'; expected '{}'.",
                parsed.version,
                store_path.display(),
                PROFILE_STORE_VERSION
            ),
        ));
    }

    Ok(parsed.profiles)
}

pub(crate) fn save_profiles(store_path: &Path, profiles: &[RemoteSshProfile]) -> RalphResult<()> {
    if let Some(parent) = store_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            err_string(
                codes::FILESYSTEM,
                format!(
                    "Failed to create SSH profile directory '{}': {error}",
                    parent.display()
                ),
            )
        })?;
    }

    let payload = SshProfileStoreV1 {
        version: PROFILE_STORE_VERSION,
        profiles: profiles.to_vec(),
    };
    let json = serde_json::to_string_pretty(&payload)
        .ralph_err(codes::FILESYSTEM, "Failed to serialize SSH profiles")?;
    std::fs::write(store_path, json).map_err(|error| {
        err_string(
            codes::FILESYSTEM,
            format!(
                "Failed to write SSH profile store '{}': {error}",
                store_path.display()
            ),
        )
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_contracts::remote::RemoteSshAuthMode;
    use tempfile::TempDir;

    #[test]
    fn round_trip_profiles() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(PROFILE_STORE_FILENAME);

        let profiles = vec![RemoteSshProfile {
            id: "p1".to_owned(),
            name: "Home".to_owned(),
            host: "example.local".to_owned(),
            username: "vince".to_owned(),
            ssh_port: 22,
            remote_port: 9944,
            auth_mode: RemoteSshAuthMode::Key,
            identity_file: None,
            identity_ref: None,
            known_hosts_file: None,
            auto_reconnect_enabled: false,
            last_used_at: None,
        }];

        save_profiles(&path, &profiles).unwrap();
        let loaded = load_profiles(&path).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "p1");
    }

    #[test]
    fn rejects_unknown_store_version() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(PROFILE_STORE_FILENAME);
        std::fs::write(&path, r#"{"version":9,"profiles":[]}"#).unwrap();
        let err = load_profiles(&path).expect_err("expected version mismatch");
        assert!(
            err.to_string()
                .contains("Unsupported SSH profile store version"),
            "unexpected error: {err}"
        );
    }
}
