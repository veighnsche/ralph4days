use ralph_errors::{codes, RalphResultExt};
use std::path::{Path, PathBuf};

const APP_NAME: &str = "ralph4days";

#[allow(dead_code)]
pub struct XdgDirs {
    data: PathBuf,
    config: PathBuf,
    cache: PathBuf,
    state: PathBuf,
}

#[allow(dead_code)]
impl XdgDirs {
    pub fn fallback() -> Self {
        let base = std::env::temp_dir().join(format!("{APP_NAME}-fallback"));
        Self {
            data: base.join("data"),
            config: base.join("config"),
            cache: base.join("cache"),
            state: base.join("state"),
        }
    }

    pub fn resolve() -> Result<Self, String> {
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

    pub fn ensure_data(&self) -> Result<&Path, String> {
        std::fs::create_dir_all(&self.data)
            .ralph_err(codes::FILESYSTEM, "Failed to create XDG data directory")?;
        Ok(&self.data)
    }

    pub fn ensure_config(&self) -> Result<&Path, String> {
        std::fs::create_dir_all(&self.config)
            .ralph_err(codes::FILESYSTEM, "Failed to create XDG config directory")?;
        Ok(&self.config)
    }

    pub fn ensure_cache(&self) -> Result<&Path, String> {
        std::fs::create_dir_all(&self.cache)
            .ralph_err(codes::FILESYSTEM, "Failed to create XDG cache directory")?;
        Ok(&self.cache)
    }

    pub fn ensure_state(&self) -> Result<&Path, String> {
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

    #[test]
    fn resolve_produces_paths_ending_with_app_name() {
        let dirs = XdgDirs::resolve().unwrap();
        assert!(dirs.data().ends_with(APP_NAME));
        assert!(dirs.config().ends_with(APP_NAME));
        assert!(dirs.cache().ends_with(APP_NAME));
        assert!(dirs.state().ends_with(APP_NAME));
    }

    #[test]
    fn ensure_data_creates_directory() {
        let dirs = XdgDirs::resolve().unwrap();
        dirs.ensure_data().unwrap();
        assert!(dirs.data().exists());
    }
}
