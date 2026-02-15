use crate::prompt_context::{build_prompt_context, PromptContextArgs};
use prompt_builder::CodebaseSnapshot;
use ralph_macros::ipc_type;
use serde::Deserialize;
use sqlite_db::SqliteDb;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PromptPreviewSection {
    pub name: String,
    pub content: String,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PromptPreview {
    pub sections: Vec<PromptPreviewSection>,
    pub full_prompt: String,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionConfig {
    pub name: String,
    pub enabled: bool,
    pub instruction_override: Option<String>,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptBuilderPreviewArgs {
    pub sections: Vec<SectionConfig>,
    pub user_input: Option<String>,
}

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
) -> Result<PromptPreview, String> {
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
