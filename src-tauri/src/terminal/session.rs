use portable_pty::MasterPty;
use std::io::Write;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct SessionInitSettings {
    pub prompt_suggestion_enabled: bool,
    pub terminal_progress_bar_enabled: bool,
    pub respect_gitignore: bool,
    pub spinner_tips_enabled: bool,
    pub prefers_reduced_motion: bool,
    pub output_style: String,
    pub auto_updates_channel: String,
}

impl Default for SessionInitSettings {
    fn default() -> Self {
        Self {
            prompt_suggestion_enabled: false,
            terminal_progress_bar_enabled: false,
            respect_gitignore: false,
            spinner_tips_enabled: false,
            prefers_reduced_motion: true,
            output_style: "default".to_owned(),
            auto_updates_channel: "latest".to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct SessionConfig {
    pub agent: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub thinking: Option<bool>,
    pub permission_level: Option<String>,
    pub init_settings: SessionInitSettings,
    pub post_start_preamble: Option<String>,
}

pub(crate) fn build_settings_json(
    init_settings: &SessionInitSettings,
    thinking: Option<bool>,
) -> String {
    let mut settings = serde_json::Map::new();

    settings.insert(
        "promptSuggestionEnabled".into(),
        init_settings.prompt_suggestion_enabled.into(),
    );
    settings.insert(
        "terminalProgressBarEnabled".into(),
        init_settings.terminal_progress_bar_enabled.into(),
    );
    settings.insert(
        "respectGitignore".into(),
        init_settings.respect_gitignore.into(),
    );
    settings.insert(
        "spinnerTipsEnabled".into(),
        init_settings.spinner_tips_enabled.into(),
    );
    settings.insert(
        "prefersReducedMotion".into(),
        init_settings.prefers_reduced_motion.into(),
    );
    settings.insert(
        "outputStyle".into(),
        init_settings.output_style.clone().into(),
    );
    settings.insert(
        "autoUpdatesChannel".into(),
        init_settings.auto_updates_channel.clone().into(),
    );

    if let Some(thinking) = thinking {
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
            agent: None,
            model: None,
            effort: None,
            thinking: None,
            permission_level: None,
            init_settings: SessionInitSettings::default(),
            post_start_preamble: None,
        };

        let json = build_settings_json(&config.init_settings, config.thinking);
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
            agent: None,
            model: None,
            effort: None,
            thinking: Some(true),
            permission_level: None,
            init_settings: SessionInitSettings::default(),
            post_start_preamble: None,
        };

        let json = build_settings_json(&config.init_settings, config.thinking);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["alwaysThinkingEnabled"], true);
    }

    #[test]
    fn test_build_settings_json_with_thinking_disabled() {
        let config = SessionConfig {
            agent: None,
            model: None,
            effort: None,
            thinking: Some(false),
            permission_level: None,
            init_settings: SessionInitSettings::default(),
            post_start_preamble: None,
        };

        let json = build_settings_json(&config.init_settings, config.thinking);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["alwaysThinkingEnabled"], false);
    }

    #[test]
    fn test_build_settings_json_with_model() {
        let config = SessionConfig {
            agent: None,
            model: Some("claude-opus-4".to_owned()),
            effort: None,
            thinking: Some(true),
            permission_level: None,
            init_settings: SessionInitSettings::default(),
            post_start_preamble: None,
        };

        let json = build_settings_json(&config.init_settings, config.thinking);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Model is NOT in settings JSON (it's a CLI flag)
        assert!(parsed.get("model").is_none());
        assert_eq!(parsed["alwaysThinkingEnabled"], true);
    }

    #[test]
    fn test_build_settings_json_output_is_valid_json() {
        let config = SessionConfig {
            agent: None,
            model: Some("haiku".to_owned()),
            effort: None,
            thinking: Some(true),
            permission_level: None,
            init_settings: SessionInitSettings::default(),
            post_start_preamble: None,
        };

        let json = build_settings_json(&config.init_settings, config.thinking);

        let result = serde_json::from_str::<serde_json::Value>(&json);
        assert!(result.is_ok());
    }
}
