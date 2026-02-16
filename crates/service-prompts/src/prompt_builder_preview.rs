use crate::prompt_context::{build_prompt_context, PromptContextArgs};
pub use core_contracts::prompt_builder::{
    PromptBuilderPreviewArgs, PromptPreview, PromptPreviewSection, SectionConfig,
};
use core_errors::RalphResult;
use data_sqlite::SqliteDb;
use prompt_builder::CodebaseSnapshot;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

pub struct PromptBuilderPreviewDeps<'a> {
    pub db: &'a SqliteDb,
    pub project_path: &'a Path,
    pub mcp_dir: &'a Path,
    pub codebase_snapshot: &'a Mutex<Option<CodebaseSnapshot>>,
    pub api_server_port: Option<u16>,
}

pub fn prompt_builder_preview(
    deps: PromptBuilderPreviewDeps<'_>,
    args: PromptBuilderPreviewArgs,
) -> RalphResult<PromptPreview> {
    let PromptBuilderPreviewDeps {
        db,
        project_path,
        mcp_dir,
        codebase_snapshot,
        api_server_port,
    } = deps;

    let PromptBuilderPreviewArgs {
        sections,
        user_input,
    } = args;

    let overrides: HashMap<String, String> = sections
        .iter()
        .filter_map(|s| {
            if s.enabled {
                s.instruction_override
                    .as_ref()
                    .map(|override_val| (s.name.clone(), override_val.clone()))
            } else {
                None
            }
        })
        .collect();

    let ctx = build_prompt_context(PromptContextArgs {
        db,
        project_path,
        mcp_dir,
        codebase_snapshot,
        api_server_port,
        user_input,
        instruction_overrides: overrides,
        target_task_id: None,
    })?;

    let enabled_names: Vec<&str> = sections
        .iter()
        .filter(|s| s.enabled)
        .map(|s| s.name.as_str())
        .collect();

    let built_sections: Vec<PromptPreviewSection> =
        prompt_builder::build_custom_sections(&enabled_names, &ctx)
            .into_iter()
            .map(|s| PromptPreviewSection {
                name: s.name,
                content: s.content,
            })
            .collect();

    let full_prompt = built_sections
        .iter()
        .map(|s| s.content.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");

    Ok(PromptPreview {
        sections: built_sections,
        full_prompt,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use service_project::project::{project_initialize, ProjectInitializeArgs};
    use tempfile::tempdir;

    #[test]
    fn preview_full_prompt_matches_sections_order_including_user_input() {
        let dir = tempdir().expect("tempdir");
        let project_path = dir.path().to_path_buf();

        project_initialize(ProjectInitializeArgs {
            path: project_path.to_string_lossy().to_string(),
            project_title: "Preview Test".to_owned(),
            stack: 1,
        })
        .expect("project_initialize");

        let db_path = project_path.join(".ralph").join("db").join("ralph.db");
        let db = SqliteDb::open(&db_path, None).expect("open db");

        let mcp_dir = project_path.join(".mcp");
        std::fs::create_dir_all(&mcp_dir).expect("create mcp_dir");
        let codebase_snapshot = Mutex::new(None);

        let preview = prompt_builder_preview(
            PromptBuilderPreviewDeps {
                db: &db,
                project_path: project_path.as_path(),
                mcp_dir: mcp_dir.as_path(),
                codebase_snapshot: &codebase_snapshot,
                api_server_port: None,
            },
            PromptBuilderPreviewArgs {
                sections: vec![
                    SectionConfig {
                        name: "project_metadata".to_owned(),
                        enabled: true,
                        instruction_override: None,
                    },
                    SectionConfig {
                        name: "user_input".to_owned(),
                        enabled: true,
                        instruction_override: None,
                    },
                ],
                user_input: Some("hello world".to_owned()),
            },
        )
        .expect("prompt_builder_preview");

        let expected_full_prompt = preview
            .sections
            .iter()
            .map(|s| s.content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");
        assert_eq!(preview.full_prompt, expected_full_prompt);

        let meta_idx = preview
            .sections
            .iter()
            .position(|s| s.name == "project_metadata")
            .expect("project_metadata section");
        let user_idx = preview
            .sections
            .iter()
            .position(|s| s.name == "user_input")
            .expect("user_input section");
        assert!(
            meta_idx < user_idx,
            "expected project_metadata to come before user_input"
        );

        let user_section = &preview.sections[user_idx];
        assert!(
            user_section.content.contains("hello world"),
            "expected user_input content to include user_input text"
        );
    }
}
