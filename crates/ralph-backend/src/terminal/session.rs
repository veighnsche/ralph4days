use portable_pty::MasterPty;
use ralph_errors::{codes, err_string};
use sqlite_db::SqliteDb;
use std::io::Write;
use std::sync::{Arc, Mutex};

use super::contract::{
    TerminalBridgeLaunchDefaults, TerminalBridgeLaunchSource, TerminalBridgeResolvedLaunchConfig,
};

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

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub agent: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub thinking: Option<bool>,
    pub permission_level: Option<String>,
    pub init_settings: SessionInitSettings,
    pub post_start_preamble: Option<String>,
}

fn normalize_opt_string(value: Option<String>) -> Option<String> {
    value.and_then(|s| {
        let trimmed = s.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_owned())
    })
}

fn normalize_agent(value: Option<String>) -> Option<String> {
    let normalized = normalize_opt_string(value)?.to_ascii_lowercase();
    match normalized.as_str() {
        "claude-code" => Some("claude".to_owned()),
        _ => Some(normalized),
    }
}

fn resolve_string_with_sources(
    task_value: Option<String>,
    discipline_value: Option<String>,
    default_value: Option<String>,
) -> (Option<String>, TerminalBridgeLaunchSource) {
    if task_value.is_some() {
        return (task_value, TerminalBridgeLaunchSource::Task);
    }
    if discipline_value.is_some() {
        return (discipline_value, TerminalBridgeLaunchSource::Discipline);
    }
    if default_value.is_some() {
        return (default_value, TerminalBridgeLaunchSource::Default);
    }
    (None, TerminalBridgeLaunchSource::Unset)
}

fn resolve_bool_with_sources(
    task_value: Option<bool>,
    discipline_value: Option<bool>,
    default_value: Option<bool>,
) -> (Option<bool>, TerminalBridgeLaunchSource) {
    if task_value.is_some() {
        return (task_value, TerminalBridgeLaunchSource::Task);
    }
    if discipline_value.is_some() {
        return (discipline_value, TerminalBridgeLaunchSource::Discipline);
    }
    if default_value.is_some() {
        return (default_value, TerminalBridgeLaunchSource::Default);
    }
    (None, TerminalBridgeLaunchSource::Unset)
}

fn validate_agent(agent: Option<&str>) -> Result<(), String> {
    let Some(agent) = agent else {
        return Ok(());
    };

    // Canonical agent ids are defined by the provider layer.
    if agent == super::providers::AGENT_CODEX || agent == super::providers::AGENT_CLAUDE {
        return Ok(());
    }

    if agent == super::providers::AGENT_SHELL && super::providers::shell_agent_enabled() {
        return Ok(());
    }

    Err(err_string(
        codes::TERMINAL,
        format!("Unknown agent '{agent}'. Expected 'codex' or 'claude'"),
    ))
}

fn model_supports_effort(agent: Option<&str>, model: Option<&str>) -> bool {
    let Some(model) = model else {
        return false;
    };

    super::providers::list_model_entries_for_agent(agent)
        .iter()
        .find(|entry| entry.name == model || entry.session_model.as_deref() == Some(model))
        .is_some_and(|entry| !entry.effort_options.is_empty())
}

pub fn resolve_task_launch_config(
    db: &SqliteDb,
    task_id: u32,
    defaults: TerminalBridgeLaunchDefaults,
) -> Result<TerminalBridgeResolvedLaunchConfig, String> {
    let task = db.get_task_by_id(task_id).ok_or_else(|| {
        err_string(
            codes::TASK_OPS,
            format!("Task '{task_id}' not found for terminal launch resolution"),
        )
    })?;

    let discipline = db
        .get_disciplines()
        .into_iter()
        .find(|d| d.name == task.discipline)
        .ok_or_else(|| {
            err_string(
                codes::DISCIPLINE_OPS,
                format!(
                    "Discipline '{}' not found for task '{task_id}'",
                    task.discipline
                ),
            )
        })?;

    let (agent, agent_source) = resolve_string_with_sources(
        normalize_agent(task.agent),
        normalize_agent(discipline.agent),
        normalize_agent(defaults.agent),
    );

    let (model, model_source) = resolve_string_with_sources(
        normalize_opt_string(task.model),
        normalize_opt_string(discipline.model),
        normalize_opt_string(defaults.model),
    );

    let (effort, effort_source) = resolve_string_with_sources(
        normalize_opt_string(task.effort),
        normalize_opt_string(discipline.effort),
        normalize_opt_string(defaults.effort),
    );

    let (thinking, thinking_source) =
        resolve_bool_with_sources(task.thinking, discipline.thinking, defaults.thinking);

    let (permission_level, permission_level_source) =
        resolve_string_with_sources(None, None, normalize_opt_string(defaults.permission_level));

    validate_agent(agent.as_deref())?;
    super::providers::resolve_session_model_for_agent(agent.as_deref(), model.clone())?;
    super::providers::resolve_session_effort_for_agent(
        agent.as_deref(),
        model.as_deref(),
        effort.clone(),
    )?;

    let supports_effort = model_supports_effort(agent.as_deref(), model.as_deref());

    Ok(TerminalBridgeResolvedLaunchConfig {
        agent,
        model,
        effort,
        thinking,
        permission_level,
        agent_source,
        model_source,
        effort_source,
        thinking_source,
        permission_level_source,
        model_supports_effort: supports_effort,
    })
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
    pub reader_handle: Option<std::thread::JoinHandle<()>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{project_initialize, ProjectInitializeArgs};
    use tempfile::tempdir;

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

    #[test]
    fn task_launch_resolution_applies_task_then_discipline_then_default_precedence() {
        let dir = tempdir().expect("tempdir");
        let project_path = dir.path().to_path_buf();

        project_initialize(ProjectInitializeArgs {
            path: project_path.to_string_lossy().to_string(),
            project_title: "Launch Resolver Test".to_owned(),
            stack: 1,
        })
        .expect("project_initialize");

        let db_path = project_path.join(".ralph").join("db").join("ralph.db");
        let db = SqliteDb::open(&db_path, None).expect("open db");

        // Make the discipline provide claude defaults.
        let discipline = db
            .get_disciplines()
            .into_iter()
            .find(|d| d.name == "implementation")
            .expect("implementation discipline exists");

        let skills_json = serde_json::to_string(&discipline.skills).expect("skills json");
        let mcp_json = serde_json::to_string(&discipline.mcp_servers).expect("mcp json");

        db.update_discipline(sqlite_db::DisciplineInput {
            name: discipline.name.clone(),
            display_name: discipline.display_name.clone(),
            acronym: discipline.acronym.clone(),
            icon: discipline.icon.clone(),
            color: discipline.color.clone(),
            description: discipline.description.clone(),
            system_prompt: discipline.system_prompt.clone(),
            agent: Some("claude".to_owned()),
            model: Some("opus".to_owned()),
            effort: Some("high".to_owned()),
            thinking: Some(true),
            skills: skills_json,
            conventions: discipline.conventions.clone(),
            mcp_servers: mcp_json,
            image_path: discipline.image_path.clone(),
            crops: discipline.crops.clone(),
            image_prompt: discipline.image_prompt.clone(),
        })
        .expect("update discipline");

        db.create_subsystem(sqlite_db::SubsystemInput {
            name: "demo".to_owned(),
            display_name: "Demo".to_owned(),
            acronym: "DEMO".to_owned(),
            description: None,
        })
        .expect("create_subsystem");

        // Create a task that overrides agent only; model/effort/thinking should come from discipline.
        let task_id = db
            .create_task(sqlite_db::TaskInput {
                subsystem: "demo".to_owned(),
                discipline: "implementation".to_owned(),
                title: "Test".to_owned(),
                description: None,
                status: None,
                priority: None,
                tags: vec![],
                depends_on: vec![],
                acceptance_criteria: None,
                context_files: vec![],
                output_artifacts: vec![],
                hints: None,
                estimated_turns: None,
                provenance: None,
                agent: Some("claude".to_owned()),
                model: None,
                effort: None,
                thinking: None,
            })
            .expect("create_task");

        let resolved = resolve_task_launch_config(
            &db,
            task_id,
            TerminalBridgeLaunchDefaults {
                agent: Some("codex".to_owned()),
                model: Some("gpt-5.3-codex".to_owned()),
                effort: Some("medium".to_owned()),
                thinking: Some(false),
                permission_level: Some("balanced".to_owned()),
            },
        )
        .expect("resolve_task_launch_config");

        assert_eq!(resolved.agent.as_deref(), Some("claude"));
        assert!(matches!(
            resolved.agent_source,
            TerminalBridgeLaunchSource::Task
        ));

        assert_eq!(resolved.model.as_deref(), Some("opus"));
        assert!(matches!(
            resolved.model_source,
            TerminalBridgeLaunchSource::Discipline
        ));

        assert_eq!(resolved.effort.as_deref(), Some("high"));
        assert!(matches!(
            resolved.effort_source,
            TerminalBridgeLaunchSource::Discipline
        ));

        assert_eq!(resolved.thinking, Some(true));
        assert!(matches!(
            resolved.thinking_source,
            TerminalBridgeLaunchSource::Discipline
        ));

        assert_eq!(resolved.permission_level.as_deref(), Some("balanced"));
        assert!(matches!(
            resolved.permission_level_source,
            TerminalBridgeLaunchSource::Default
        ));

        assert!(resolved.model_supports_effort);
    }

    #[test]
    fn task_launch_resolution_hard_fails_on_invalid_agent_model_pair() {
        let dir = tempdir().expect("tempdir");
        let project_path = dir.path().to_path_buf();

        project_initialize(ProjectInitializeArgs {
            path: project_path.to_string_lossy().to_string(),
            project_title: "Launch Resolver Reject Test".to_owned(),
            stack: 1,
        })
        .expect("project_initialize");

        let db_path = project_path.join(".ralph").join("db").join("ralph.db");
        let db = SqliteDb::open(&db_path, None).expect("open db");

        // Give the discipline a claude-only model.
        let discipline = db
            .get_disciplines()
            .into_iter()
            .find(|d| d.name == "implementation")
            .expect("implementation discipline exists");
        let skills_json = serde_json::to_string(&discipline.skills).expect("skills json");
        let mcp_json = serde_json::to_string(&discipline.mcp_servers).expect("mcp json");
        db.update_discipline(sqlite_db::DisciplineInput {
            name: discipline.name.clone(),
            display_name: discipline.display_name.clone(),
            acronym: discipline.acronym.clone(),
            icon: discipline.icon.clone(),
            color: discipline.color.clone(),
            description: discipline.description.clone(),
            system_prompt: discipline.system_prompt.clone(),
            agent: Some("claude".to_owned()),
            model: Some("opus".to_owned()),
            effort: Some("high".to_owned()),
            thinking: None,
            skills: skills_json,
            conventions: discipline.conventions.clone(),
            mcp_servers: mcp_json,
            image_path: discipline.image_path.clone(),
            crops: discipline.crops.clone(),
            image_prompt: discipline.image_prompt.clone(),
        })
        .expect("update discipline");

        db.create_subsystem(sqlite_db::SubsystemInput {
            name: "demo".to_owned(),
            display_name: "Demo".to_owned(),
            acronym: "DEMO".to_owned(),
            description: None,
        })
        .expect("create_subsystem");

        // Task forces codex agent but does not set model, so discipline model would win and be invalid.
        let task_id = db
            .create_task(sqlite_db::TaskInput {
                subsystem: "demo".to_owned(),
                discipline: "implementation".to_owned(),
                title: "Reject".to_owned(),
                description: None,
                status: None,
                priority: None,
                tags: vec![],
                depends_on: vec![],
                acceptance_criteria: None,
                context_files: vec![],
                output_artifacts: vec![],
                hints: None,
                estimated_turns: None,
                provenance: None,
                agent: Some("codex".to_owned()),
                model: None,
                effort: None,
                thinking: None,
            })
            .expect("create_task");

        let err = resolve_task_launch_config(
            &db,
            task_id,
            TerminalBridgeLaunchDefaults {
                agent: Some("codex".to_owned()),
                model: Some("gpt-5.3-codex".to_owned()),
                effort: Some("medium".to_owned()),
                thinking: Some(false),
                permission_level: Some("balanced".to_owned()),
            },
        )
        .unwrap_err();

        assert!(
            err.contains("[R-7000]") || err.contains("[R-8100]"),
            "expected a coded error, got: {err}"
        );
    }
}
