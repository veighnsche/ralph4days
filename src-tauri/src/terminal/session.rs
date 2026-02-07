use portable_pty::MasterPty;
use std::io::Write;
use std::sync::{Arc, Mutex};

pub struct SessionConfig {
    pub model: Option<String>,
    pub thinking: Option<bool>,
}

pub(crate) fn build_settings_json(config: &SessionConfig) -> String {
    let mut settings = serde_json::Map::new();

    // Always enforced by Ralph
    settings.insert("promptSuggestionEnabled".into(), false.into());
    settings.insert("terminalProgressBarEnabled".into(), false.into());
    settings.insert("respectGitignore".into(), false.into());
    settings.insert("spinnerTipsEnabled".into(), false.into());
    settings.insert("prefersReducedMotion".into(), true.into());
    settings.insert("outputStyle".into(), "default".into());
    settings.insert("autoUpdatesChannel".into(), "latest".into());

    if let Some(thinking) = config.thinking {
        settings.insert("alwaysThinkingEnabled".into(), thinking.into());
    }

    serde_json::Value::Object(settings).to_string()
}

pub(crate) struct PTYSession {
    pub writer: Arc<Mutex<Box<dyn Write + Send>>>,
    pub master: Box<dyn MasterPty + Send>,
    pub child: Arc<Mutex<Box<dyn portable_pty::Child + Send + Sync>>>,
    // Stored for ownership — reader thread runs until EOF, then self-cleans
    pub _reader_handle: Option<std::thread::JoinHandle<()>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_settings_json_default_config() {
        let config = SessionConfig {
            model: None,
            thinking: None,
        };

        let json = build_settings_json(&config);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["promptSuggestionEnabled"], false);
        assert_eq!(parsed["terminalProgressBarEnabled"], false);
        assert_eq!(parsed["respectGitignore"], false);
        assert_eq!(parsed["spinnerTipsEnabled"], false);
        assert_eq!(parsed["prefersReducedMotion"], true);
        assert_eq!(parsed["outputStyle"], "default");
        assert_eq!(parsed["autoUpdatesChannel"], "latest");

        assert!(parsed.get("alwaysThinkingEnabled").is_none());
    }

    #[test]
    fn test_build_settings_json_with_thinking_enabled() {
        let config = SessionConfig {
            model: None,
            thinking: Some(true),
        };

        let json = build_settings_json(&config);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["alwaysThinkingEnabled"], true);
    }

    #[test]
    fn test_build_settings_json_with_thinking_disabled() {
        let config = SessionConfig {
            model: None,
            thinking: Some(false),
        };

        let json = build_settings_json(&config);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["alwaysThinkingEnabled"], false);
    }

    #[test]
    fn test_build_settings_json_with_model() {
        let config = SessionConfig {
            model: Some("claude-opus-4".to_owned()),
            thinking: Some(true),
        };

        let json = build_settings_json(&config);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Model is NOT in settings JSON (it's a CLI flag)
        assert!(parsed.get("model").is_none());
        assert_eq!(parsed["alwaysThinkingEnabled"], true);
    }

    #[test]
    fn test_build_settings_json_output_is_valid_json() {
        let config = SessionConfig {
            model: Some("haiku".to_owned()),
            thinking: Some(true),
        };

        let json = build_settings_json(&config);

        let result = serde_json::from_str::<serde_json::Value>(&json);
        assert!(result.is_ok());
    }
}
