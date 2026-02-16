use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalServicesConfig {
    pub version: u32,
    pub ollama: OllamaConfig,
    pub comfy: ComfyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub api_url: String,
    pub embedding_model: String,
    pub embedding_dims: u32,
    pub llm_model: String,
    pub llm_temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComfyConfig {
    pub api_url: String,
    pub default_workflow: String,
    pub timeout_secs: u64,
}

impl Default for ExternalServicesConfig {
    fn default() -> Self {
        Self {
            version: 1,
            ollama: OllamaConfig {
                api_url: "http://localhost:11434".into(),
                embedding_model: "nomic-embed-text".into(),
                embedding_dims: 768,
                llm_model: "qwen2.5-coder:7b".into(),
                llm_temperature: 0.7,
            },
            comfy: ComfyConfig {
                api_url: "http://localhost:8188".into(),
                default_workflow: "discipline_character.json".into(),
                timeout_secs: 300,
            },
        }
    }
}

impl ExternalServicesConfig {
    pub fn config_path() -> Result<PathBuf, String> {
        let config_dir = dirs::config_dir()
            .ok_or("No config directory on this platform")?
            .join("ralph");

        std::fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config dir: {e}"))?;

        Ok(config_dir.join("external_services.json"))
    }

    pub fn load() -> Result<Self, String> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let contents =
            std::fs::read_to_string(&path).map_err(|e| format!("Failed to read config: {e}"))?;

        let config: Self =
            serde_json::from_str(&contents).map_err(|e| format!("Failed to parse config: {e}"))?;

        config.validate()?;

        Ok(config)
    }

    pub fn save(&self) -> Result<(), String> {
        self.validate()?;

        let path = Self::config_path()?;

        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {e}"))?;

        std::fs::write(&path, contents).map_err(|e| format!("Failed to write config: {e}"))?;

        Ok(())
    }

    fn validate(&self) -> Result<(), String> {
        validate_url(&self.ollama.api_url, "Ollama API URL")?;
        validate_url(&self.comfy.api_url, "ComfyUI API URL")?;
        validate_workflow_path(&self.comfy.default_workflow)?;

        if self.ollama.embedding_dims == 0 {
            return Err("Embedding dimensions must be > 0".into());
        }

        if !(0.0..=2.0).contains(&self.ollama.llm_temperature) {
            return Err("LLM temperature must be between 0.0 and 2.0".into());
        }

        Ok(())
    }
}

fn validate_url(url: &str, name: &str) -> Result<(), String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(format!("{name} must use http:// or https://"));
    }

    reqwest::Url::parse(url).map_err(|e| format!("Invalid {name}: {e}"))?;

    Ok(())
}

fn validate_workflow_path(path: &str) -> Result<(), String> {
    let path_buf = Path::new(path);

    if path_buf.is_absolute() {
        return Err("Workflow path must be relative (filename only)".into());
    }

    if path.contains("..") {
        return Err("Workflow path cannot contain '..' (path traversal blocked)".into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = ExternalServicesConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn rejects_invalid_urls() {
        let mut config = ExternalServicesConfig::default();
        config.ollama.api_url = "file:///etc/passwd".into();
        assert!(config.validate().is_err());
    }

    #[test]
    fn rejects_path_traversal() {
        let mut config = ExternalServicesConfig::default();
        config.comfy.default_workflow = "../../etc/passwd".into();
        assert!(config.validate().is_err());
    }

    #[test]
    fn rejects_absolute_workflow_paths() {
        let mut config = ExternalServicesConfig::default();
        config.comfy.default_workflow = "/etc/passwd".into();
        assert!(config.validate().is_err());
    }

    #[test]
    fn rejects_invalid_temperature() {
        let mut config = ExternalServicesConfig::default();
        config.ollama.llm_temperature = 5.0;
        assert!(config.validate().is_err());
    }
}
