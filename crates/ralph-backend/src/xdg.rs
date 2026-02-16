use ralph_errors::{codes, RalphResult, RalphResultExt};
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
        let data = dirs::data_dir()
            .ok_or_else(|| ralph_errors::err_string(codes::FILESYSTEM, "No XDG data directory"))?
            .join(APP_NAME);

        let config = dirs::config_dir()
            .ok_or_else(|| ralph_errors::err_string(codes::FILESYSTEM, "No XDG config directory"))?
            .join(APP_NAME);

        let cache = dirs::cache_dir()
            .ok_or_else(|| ralph_errors::err_string(codes::FILESYSTEM, "No XDG cache directory"))?
            .join(APP_NAME);

        let state = dirs::state_dir()
            .ok_or_else(|| ralph_errors::err_string(codes::FILESYSTEM, "No XDG state directory"))?
            .join(APP_NAME);

        Ok(Self {
            data,
            config,
            cache,
            state,
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
}
