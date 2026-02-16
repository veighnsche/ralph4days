use core_errors::{codes, RalphResult, RalphResultExt};
use std::path::{Path, PathBuf};

const APP_NAME: &str = "ralph4days";

pub struct XdgDirs {
    data: PathBuf,
    config: PathBuf,
    cache: PathBuf,
    state: PathBuf,
}

impl XdgDirs {
    pub fn resolve() -> RalphResult<Self> {
        let data_base = dirs::data_dir()
            .ok_or_else(|| core_errors::err_string(codes::FILESYSTEM, "No XDG data directory"))?;

        let config_base = dirs::config_dir()
            .ok_or_else(|| core_errors::err_string(codes::FILESYSTEM, "No XDG config directory"))?;

        let cache_base = dirs::cache_dir()
            .ok_or_else(|| core_errors::err_string(codes::FILESYSTEM, "No XDG cache directory"))?;

        let state_base = resolve_state_base(&data_base, dirs::state_dir(), dirs::data_local_dir());

        Ok(Self {
            data: data_base.join(APP_NAME),
            config: config_base.join(APP_NAME),
            cache: cache_base.join(APP_NAME),
            state: state_base.join(APP_NAME),
        })
    }

    pub fn data(&self) -> &Path {
        &self.data
    }

    pub fn config(&self) -> &Path {
        &self.config
    }

    pub fn cache(&self) -> &Path {
        &self.cache
    }

    pub fn state(&self) -> &Path {
        &self.state
    }

    pub fn ensure_data(&self) -> RalphResult<&Path> {
        std::fs::create_dir_all(&self.data)
            .ralph_err(codes::FILESYSTEM, "Failed to create XDG data directory")?;
        Ok(&self.data)
    }

    pub fn ensure_config(&self) -> RalphResult<&Path> {
        std::fs::create_dir_all(&self.config)
            .ralph_err(codes::FILESYSTEM, "Failed to create XDG config directory")?;
        Ok(&self.config)
    }

    pub fn ensure_cache(&self) -> RalphResult<&Path> {
        std::fs::create_dir_all(&self.cache)
            .ralph_err(codes::FILESYSTEM, "Failed to create XDG cache directory")?;
        Ok(&self.cache)
    }

    pub fn ensure_state(&self) -> RalphResult<&Path> {
        std::fs::create_dir_all(&self.state)
            .ralph_err(codes::FILESYSTEM, "Failed to create XDG state directory")?;
        Ok(&self.state)
    }

    #[cfg(test)]
    pub fn from_base(base: &Path) -> Self {
        Self {
            data: base.join("data").join(APP_NAME),
            config: base.join("config").join(APP_NAME),
            cache: base.join("cache").join(APP_NAME),
            state: base.join("state").join(APP_NAME),
        }
    }
}

fn resolve_state_base(
    data_base: &Path,
    state_base: Option<PathBuf>,
    data_local_base: Option<PathBuf>,
) -> PathBuf {
    if let Some(path) = state_base {
        return path;
    }

    if let Some(path) = data_local_base {
        tracing::warn!(
            "No XDG state directory reported by host; falling back to data-local directory"
        );
        return path;
    }

    tracing::warn!(
        "No XDG state/data-local directory reported by host; falling back to data directory"
    );
    data_base.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn from_base_produces_paths_ending_with_app_name() {
        let base = TempDir::new().unwrap();
        let dirs = XdgDirs::from_base(base.path());
        assert!(dirs.data().ends_with(APP_NAME));
        assert!(dirs.config().ends_with(APP_NAME));
        assert!(dirs.cache().ends_with(APP_NAME));
        assert!(dirs.state().ends_with(APP_NAME));
    }

    #[test]
    fn ensure_data_creates_directory() {
        let base = TempDir::new().unwrap();
        let dirs = XdgDirs::from_base(base.path());
        dirs.ensure_data().unwrap();
        assert!(dirs.data().exists());
    }

    #[test]
    fn resolve_state_base_prefers_state_dir() {
        let base = TempDir::new().unwrap();
        let explicit_state = base.path().join("state-explicit");
        let data_local = base.path().join("data-local");
        let resolved =
            resolve_state_base(base.path(), Some(explicit_state.clone()), Some(data_local));
        assert_eq!(resolved, explicit_state);
    }

    #[test]
    fn resolve_state_base_falls_back_to_data_local_dir() {
        let base = TempDir::new().unwrap();
        let data_local = base.path().join("data-local");
        let resolved = resolve_state_base(base.path(), None, Some(data_local.clone()));
        assert_eq!(resolved, data_local);
    }

    #[test]
    fn resolve_state_base_falls_back_to_data_dir() {
        let base = TempDir::new().unwrap();
        let resolved = resolve_state_base(base.path(), None, None);
        assert_eq!(resolved, base.path());
    }
}
