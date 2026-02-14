use super::state::{AppState, CommandContext};
use crate::diagnostics;
use ralph_errors::{codes, RalphResultExt};
use ralph_macros::ipc_type;
use tauri::State;

fn build_embedding_config(
    ext_config: &ralph_external::ExternalServicesConfig,
) -> ralph_external::comment_embeddings::CommentEmbeddingConfig<'_> {
    ralph_external::comment_embeddings::CommentEmbeddingConfig {
        ollama: &ext_config.ollama,
        document_prefix: "search_document: ",
        query_prefix: "search_query: ",
        min_search_score: 0.4,
        max_search_results: 10,
    }
}

fn db_path(ctx: &CommandContext<'_>) -> Result<std::path::PathBuf, String> {
    let project_path = ctx.locked_project_path()?;
    Ok(project_path.join(".ralph").join("db").join("ralph.db"))
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerConfigData {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: std::collections::HashMap<String, String>,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CropBoxData {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisciplineCropsData {
    pub face: CropBoxData,
    pub card: CropBoxData,
    pub upperbody: Option<CropBoxData>,
    pub portrait: Option<CropBoxData>,
    pub landscape: Option<CropBoxData>,
    pub strip: Option<CropBoxData>,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisciplineImagePromptData {
    pub positive: String,
    pub negative: String,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisciplineTaskTemplateData {
    pub id: u32,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub hints: Option<String>,
    pub estimated_turns: Option<u32>,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub thinking: Option<bool>,
    pub pseudocode: Option<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub pulled_count: u32,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisciplineConfig {
    pub id: u32,
    pub name: String,
    pub display_name: String,
    pub icon: String,
    pub color: String,
    pub acronym: String,
    pub description: Option<String>,
    pub system_prompt: Option<String>,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub thinking: Option<bool>,
    pub skills: Vec<String>,
    pub conventions: Option<String>,
    pub mcp_servers: Vec<McpServerConfigData>,
    pub stack_id: Option<u8>,
    pub image_path: Option<String>,
    pub crops: Option<DisciplineCropsData>,
    pub image_prompt: Option<DisciplineImagePromptData>,
    pub task_templates: Vec<DisciplineTaskTemplateData>,
}

fn to_discipline_config(db: &sqlite_db::SqliteDb, d: &sqlite_db::Discipline) -> DisciplineConfig {
    DisciplineConfig {
        id: d.id,
        name: d.name.clone(),
        display_name: d.display_name.clone(),
        icon: d.icon.clone(),
        color: d.color.clone(),
        acronym: d.acronym.clone(),
        description: d.description.clone(),
        system_prompt: d.system_prompt.clone(),
        agent: d.agent.clone(),
        model: d.model.clone(),
        effort: d.effort.clone(),
        thinking: d.thinking,
        skills: d.skills.clone(),
        conventions: d.conventions.clone(),
        mcp_servers: d
            .mcp_servers
            .iter()
            .map(|m| McpServerConfigData {
                name: m.name.clone(),
                command: m.command.clone(),
                args: m.args.clone(),
                env: m.env.clone(),
            })
            .collect(),
        stack_id: d.stack_id,
        image_path: d.image_path.clone(),
        crops: d
            .crops
            .as_deref()
            .and_then(|s| serde_json::from_str::<DisciplineCropsData>(s).ok()),
        image_prompt: d
            .image_prompt
            .as_deref()
            .and_then(|s| serde_json::from_str::<DisciplineImagePromptData>(s).ok()),
        task_templates: db
            .get_active_task_templates_for_discipline(d.id)
            .into_iter()
            .map(|template| DisciplineTaskTemplateData {
                id: template.id,
                title: template.title,
                description: template.description,
                priority: template.priority.map(|p| p.as_str().to_owned()),
                hints: template.hints,
                estimated_turns: template.estimated_turns,
                agent: template.agent,
                model: template.model,
                effort: template.effort,
                thinking: template.thinking,
                pseudocode: template.pseudocode,
                created: template.created,
                updated: template.updated,
                pulled_count: template.pulled_count,
            })
            .collect(),
    }
}

fn get_discipline_config_or_error(
    db: &sqlite_db::SqliteDb,
    name: &str,
) -> Result<DisciplineConfig, String> {
    let disciplines = db.get_disciplines();
    let discipline = disciplines.iter().find(|d| d.name == name).ok_or_else(|| {
        ralph_errors::err_string(
            codes::DISCIPLINE_OPS,
            format!("Discipline '{name}' not found"),
        )
    })?;
    Ok(to_discipline_config(db, discipline))
}

#[tauri::command]
pub fn disciplines_list(state: State<'_, AppState>) -> Result<Vec<DisciplineConfig>, String> {
    CommandContext::from_tauri_state(&state).db(|db| {
        Ok(db
            .get_disciplines()
            .iter()
            .map(|d| to_discipline_config(db, d))
            .collect())
    })
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsystemCommentData {
    pub id: u32,
    pub category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discipline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_task_id: Option<u32>,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_iteration: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsystemData {
    pub id: u32,
    pub name: String,
    pub display_name: String,
    pub acronym: String,
    pub description: Option<String>,
    pub created: Option<String>,
    pub status: String,
    pub comments: Vec<SubsystemCommentData>,
}

fn to_comment_data(c: &sqlite_db::SubsystemComment) -> SubsystemCommentData {
    SubsystemCommentData {
        id: c.id,
        category: c.category.clone(),
        discipline: c.discipline.clone(),
        agent_task_id: c.agent_task_id,
        body: c.body.clone(),
        summary: c.summary.clone(),
        reason: c.reason.clone(),
        source_iteration: c.source_iteration,
        created: c.created.clone(),
        updated: c.updated.clone(),
    }
}

fn to_subsystem_data(subsystem: &sqlite_db::Subsystem) -> SubsystemData {
    SubsystemData {
        id: subsystem.id,
        name: subsystem.name.clone(),
        display_name: subsystem.display_name.clone(),
        acronym: subsystem.acronym.clone(),
        description: subsystem.description.clone(),
        created: subsystem.created.clone(),
        status: subsystem.status.as_str().to_owned(),
        comments: subsystem.comments.iter().map(to_comment_data).collect(),
    }
}

fn get_subsystem_data_or_error(
    db: &sqlite_db::SqliteDb,
    name: &str,
) -> Result<SubsystemData, String> {
    let subsystems = db.get_subsystems();
    let subsystem = subsystems.iter().find(|f| f.name == name).ok_or_else(|| {
        ralph_errors::err_string(codes::FEATURE_OPS, format!("Subsystem '{name}' not found"))
    })?;
    Ok(to_subsystem_data(subsystem))
}

#[tauri::command]
pub fn subsystems_list(state: State<'_, AppState>) -> Result<Vec<SubsystemData>, String> {
    CommandContext::from_tauri_state(&state)
        .db(|db| Ok(db.get_subsystems().iter().map(to_subsystem_data).collect()))
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsystemsCreateArgs {
    pub name: String,
    pub display_name: String,
    pub acronym: String,
    pub description: Option<String>,
}

#[tauri::command]
pub fn subsystems_create(
    state: State<'_, AppState>,
    args: SubsystemsCreateArgs,
) -> Result<SubsystemData, String> {
    let subsystem_name = args.name.clone();
    CommandContext::from_tauri_state(&state).db(|db| {
        db.create_subsystem(sqlite_db::SubsystemInput {
            name: args.name,
            display_name: args.display_name,
            acronym: args.acronym,
            description: args.description,
        })?;
        get_subsystem_data_or_error(db, &subsystem_name)
    })
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsystemsUpdateArgs {
    pub name: String,
    pub display_name: String,
    pub acronym: String,
    pub description: Option<String>,
}

#[tauri::command]
pub fn subsystems_update(
    state: State<'_, AppState>,
    args: SubsystemsUpdateArgs,
) -> Result<SubsystemData, String> {
    let subsystem_name = args.name.clone();
    CommandContext::from_tauri_state(&state).db(|db| {
        db.update_subsystem(sqlite_db::SubsystemInput {
            name: args.name,
            display_name: args.display_name,
            acronym: args.acronym,
            description: args.description,
        })?;
        get_subsystem_data_or_error(db, &subsystem_name)
    })
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsystemsCommentAddArgs {
    pub subsystem_name: String,
    pub category: String,
    pub discipline: Option<String>,
    pub agent_task_id: Option<u32>,
    pub body: String,
    pub summary: Option<String>,
    pub reason: Option<String>,
    pub source_iteration: Option<u32>,
}

#[tauri::command]
pub async fn subsystems_comment_add(
    state: State<'_, AppState>,
    args: SubsystemsCommentAddArgs,
) -> Result<SubsystemData, String> {
    let command_ctx = CommandContext::from_tauri_state(&state);
    let path = db_path(&command_ctx)?;
    let (comment_id, embedding_text, subsystem) = command_ctx.db_tx(|db| {
        db.add_subsystem_comment(sqlite_db::AddSubsystemCommentInput {
            subsystem_name: args.subsystem_name.clone(),
            category: args.category.clone(),
            discipline: args.discipline.clone(),
            agent_task_id: args.agent_task_id,
            body: args.body.clone(),
            summary: args.summary.clone(),
            reason: args.reason.clone(),
            source_iteration: args.source_iteration,
        })?;

        let subsystem = get_subsystem_data_or_error(db, &args.subsystem_name)?;
        let cid = subsystem
            .comments
            .last()
            .map(|c| c.id)
            .ok_or("Failed to get new comment ID")?;

        let text = ralph_external::comment_embeddings::build_embedding_text(
            &args.category,
            &args.body,
            args.reason.as_deref(),
        );
        Ok((cid, text, subsystem))
    })?;

    let ext_config = ralph_external::ExternalServicesConfig::load()?;
    let embed_config = build_embedding_config(&ext_config);
    let result =
        ralph_external::comment_embeddings::embed_text(&embed_config, &embedding_text).await?;

    let db = sqlite_db::SqliteDb::open(&path, None)?;
    db.upsert_comment_embedding(comment_id, &result.vector, &result.model, &result.hash)?;
    Ok(subsystem)
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsystemsCommentUpdateArgs {
    pub subsystem_name: String,
    pub comment_id: u32,
    pub body: String,
    pub summary: Option<String>,
    pub reason: Option<String>,
}

#[tauri::command]
pub async fn subsystems_comment_update(
    state: State<'_, AppState>,
    args: SubsystemsCommentUpdateArgs,
) -> Result<SubsystemData, String> {
    let command_ctx = CommandContext::from_tauri_state(&state);
    let path = db_path(&command_ctx)?;
    let (embedding_text, needs_embed, subsystem) = command_ctx.db_tx(|db| {
        db.update_subsystem_comment(
            &args.subsystem_name,
            args.comment_id,
            &args.body,
            args.summary.clone(),
            args.reason.clone(),
        )?;

        let subsystem = get_subsystem_data_or_error(db, &args.subsystem_name)?;
        let category = subsystem
            .comments
            .iter()
            .find(|c| c.id == args.comment_id)
            .map(|c| c.category.clone())
            .ok_or("Comment not found after update")?;

        let text = ralph_external::comment_embeddings::build_embedding_text(
            &category,
            &args.body,
            args.reason.as_deref(),
        );
        let needs = ralph_external::comment_embeddings::should_embed(
            db,
            args.comment_id,
            &category,
            &args.body,
            args.reason.as_deref(),
        )
        .is_some();
        Ok((text, needs, subsystem))
    })?;

    if !needs_embed {
        return Ok(subsystem);
    }

    let ext_config = ralph_external::ExternalServicesConfig::load()?;
    let embed_config = build_embedding_config(&ext_config);
    let result =
        ralph_external::comment_embeddings::embed_text(&embed_config, &embedding_text).await?;

    let db = sqlite_db::SqliteDb::open(&path, None)?;
    db.upsert_comment_embedding(args.comment_id, &result.vector, &result.model, &result.hash)?;
    Ok(subsystem)
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsystemsCommentDeleteArgs {
    pub subsystem_name: String,
    pub comment_id: u32,
}

#[tauri::command]
pub fn subsystems_comment_delete(
    state: State<'_, AppState>,
    args: SubsystemsCommentDeleteArgs,
) -> Result<SubsystemData, String> {
    CommandContext::from_tauri_state(&state).db(|db| {
        db.delete_subsystem_comment(&args.subsystem_name, args.comment_id)?;
        get_subsystem_data_or_error(db, &args.subsystem_name)
    })
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisciplinesCreateArgs {
    pub name: String,
    pub display_name: String,
    pub acronym: String,
    pub icon: String,
    pub color: String,
    pub system_prompt: Option<String>,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub thinking: Option<bool>,
    pub skills: Vec<String>,
    pub conventions: Option<String>,
    pub mcp_servers: Vec<McpServerConfigData>,
}

#[tauri::command]
pub fn disciplines_create(
    state: State<'_, AppState>,
    args: DisciplinesCreateArgs,
) -> Result<DisciplineConfig, String> {
    let normalized_name = args
        .name
        .to_lowercase()
        .trim()
        .replace(char::is_whitespace, "-");
    let discipline_name = normalized_name.clone();

    let skills_json = serde_json::to_string(&args.skills)
        .ralph_err(codes::DISCIPLINE_OPS, "Failed to serialize skills")?;

    let mcp_servers: Vec<sqlite_db::McpServerConfig> = args
        .mcp_servers
        .iter()
        .map(|m| sqlite_db::McpServerConfig {
            name: m.name.clone(),
            command: m.command.clone(),
            args: m.args.clone(),
            env: m.env.clone(),
        })
        .collect();

    let mcp_json = serde_json::to_string(&mcp_servers)
        .ralph_err(codes::DISCIPLINE_OPS, "Failed to serialize mcp_servers")?;

    CommandContext::from_tauri_state(&state).db(|db| {
        db.create_discipline(sqlite_db::DisciplineInput {
            name: normalized_name,
            display_name: args.display_name,
            acronym: args.acronym,
            icon: args.icon,
            color: args.color,
            description: None,
            system_prompt: args.system_prompt,
            agent: args.agent,
            model: args.model,
            effort: args.effort,
            thinking: args.thinking,
            skills: skills_json,
            conventions: args.conventions,
            mcp_servers: mcp_json,
            image_path: None,
            crops: None,
            image_prompt: None,
        })?;
        get_discipline_config_or_error(db, &discipline_name)
    })
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisciplinesUpdateArgs {
    pub name: String,
    pub display_name: String,
    pub acronym: String,
    pub icon: String,
    pub color: String,
    pub system_prompt: Option<String>,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub thinking: Option<bool>,
    pub skills: Vec<String>,
    pub conventions: Option<String>,
    pub mcp_servers: Vec<McpServerConfigData>,
}

#[tauri::command]
pub fn disciplines_update(
    state: State<'_, AppState>,
    args: DisciplinesUpdateArgs,
) -> Result<DisciplineConfig, String> {
    let skills_json = serde_json::to_string(&args.skills)
        .ralph_err(codes::DISCIPLINE_OPS, "Failed to serialize skills")?;

    let mcp_servers: Vec<sqlite_db::McpServerConfig> = args
        .mcp_servers
        .iter()
        .map(|m| sqlite_db::McpServerConfig {
            name: m.name.clone(),
            command: m.command.clone(),
            args: m.args.clone(),
            env: m.env.clone(),
        })
        .collect();

    let mcp_json = serde_json::to_string(&mcp_servers)
        .ralph_err(codes::DISCIPLINE_OPS, "Failed to serialize mcp_servers")?;

    let discipline_name = args.name.clone();
    CommandContext::from_tauri_state(&state).db(|db| {
        db.update_discipline(sqlite_db::DisciplineInput {
            name: args.name,
            display_name: args.display_name,
            acronym: args.acronym,
            icon: args.icon,
            color: args.color,
            description: None,
            system_prompt: args.system_prompt,
            agent: args.agent,
            model: args.model,
            effort: args.effort,
            thinking: args.thinking,
            skills: skills_json,
            conventions: args.conventions,
            mcp_servers: mcp_json,
            image_path: None,
            crops: None,
            image_prompt: None,
        })?;
        get_discipline_config_or_error(db, &discipline_name)
    })
}

#[tauri::command]
pub fn subsystems_delete(
    state: State<'_, AppState>,
    args: SubsystemsDeleteArgs,
) -> Result<(), String> {
    CommandContext::from_tauri_state(&state).db(|db| db.delete_subsystem(args.name))
}

#[tauri::command]
pub fn disciplines_delete(
    state: State<'_, AppState>,
    args: DisciplinesDeleteArgs,
) -> Result<String, String> {
    let deleted_name = args.name.clone();
    CommandContext::from_tauri_state(&state).db(|db| {
        db.delete_discipline(args.name)?;
        Ok(deleted_name)
    })
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsystemsDeleteArgs {
    pub name: String,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisciplinesDeleteArgs {
    pub name: String,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualIdentityData {
    pub style: String,
    pub theme: String,
    pub tone: String,
    pub references: String,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StackMetadataData {
    pub stack_id: u8,
    pub name: String,
    pub description: String,
    pub philosophy: String,
    pub visual_identity: VisualIdentityData,
    pub when_to_use: Vec<String>,
    pub discipline_count: u8,
    pub characteristics: Vec<String>,
}

#[tauri::command]
pub fn stacks_metadata_list() -> Vec<StackMetadataData> {
    predefined_disciplines::get_all_stack_metadata()
        .iter()
        .map(|m| StackMetadataData {
            stack_id: m.stack_id,
            name: m.name.clone(),
            description: m.description.clone(),
            philosophy: m.philosophy.clone(),
            visual_identity: VisualIdentityData {
                style: m.visual_identity.style.clone(),
                theme: m.visual_identity.theme.clone(),
                tone: m.visual_identity.tone.clone(),
                references: m.visual_identity.references.clone(),
            },
            when_to_use: m.when_to_use.clone(),
            discipline_count: m.discipline_count,
            characteristics: m.characteristics.clone(),
        })
        .collect()
}

#[tauri::command]
pub fn disciplines_image_data_get(
    state: State<'_, AppState>,
    args: DisciplinesImageDataGetArgs,
) -> Result<Option<String>, String> {
    use base64::Engine;

    let ctx = CommandContext::from_tauri_state(&state);
    let disc = ctx.db(|db| {
        Ok(db
            .get_disciplines()
            .into_iter()
            .find(|d| d.name == args.discipline_name))
    })?;

    let Some(disc) = disc else {
        return Ok(None);
    };
    let Some(ref image_path) = disc.image_path else {
        return Ok(None);
    };

    let project_path = ctx.locked_project_path()?;
    let abs_path = project_path.join(".ralph").join(image_path);

    let bytes = std::fs::read(&abs_path).map_err(|error| {
        ralph_errors::err_string(
            codes::FILESYSTEM,
            format!(
                "Failed to read discipline image '{}' at {}: {error}",
                args.discipline_name,
                abs_path.display()
            ),
        )
    })?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(Some(b64))
}

#[tauri::command]
pub fn disciplines_cropped_image_get(
    state: State<'_, AppState>,
    args: DisciplinesCroppedImageGetArgs,
) -> Result<Option<String>, String> {
    use base64::Engine;
    use std::io::Cursor;

    let ctx = CommandContext::from_tauri_state(&state);
    let disc = ctx.db(|db| {
        Ok(db
            .get_disciplines()
            .into_iter()
            .find(|d| d.name == args.discipline_name))
    })?;

    let Some(disc) = disc else {
        return Err(ralph_errors::err_string(
            codes::DISCIPLINE_OPS,
            format!("Discipline '{}' not found", args.discipline_name),
        ));
    };
    let Some(ref image_path) = disc.image_path else {
        return Ok(None);
    };

    let project_path = ctx.locked_project_path()?;
    let cache_dir = project_path.join(".ralph").join("cache").join("crops");
    let cache_key = format!(
        "{}_{}_{}_{}_{}_{}.png",
        args.discipline_name, args.label, args.crop.x, args.crop.y, args.crop.w, args.crop.h
    );
    let cache_path = cache_dir.join(&cache_key);

    if cache_path.exists() {
        match std::fs::read(&cache_path) {
            Ok(bytes) => {
                return Ok(Some(
                    base64::engine::general_purpose::STANDARD.encode(&bytes),
                ));
            }
            Err(error) => {
                diagnostics::emit_warning(
                    "disciplines",
                    "crop-cache-read-failed",
                    &format!(
                        "Failed to read crop cache at {}: {error}. Regenerating.",
                        cache_path.display()
                    ),
                );
            }
        }
    }

    let abs_path = project_path.join(".ralph").join(image_path);
    let src_bytes = std::fs::read(&abs_path).map_err(|error| {
        ralph_errors::err_string(
            codes::FILESYSTEM,
            format!(
                "Failed to read discipline image '{}' at {}: {error}",
                args.discipline_name,
                abs_path.display()
            ),
        )
    })?;

    if !(args.crop.x.is_finite()
        && args.crop.y.is_finite()
        && args.crop.w.is_finite()
        && args.crop.h.is_finite())
    {
        return Err(ralph_errors::err_string(
            codes::DISCIPLINE_OPS,
            format!(
                "Invalid crop box (non-finite): x={} y={} w={} h={}",
                args.crop.x, args.crop.y, args.crop.w, args.crop.h
            ),
        ));
    }

    if args.crop.x < 0.0 || args.crop.y < 0.0 || args.crop.w <= 0.0 || args.crop.h <= 0.0 {
        return Err(ralph_errors::err_string(
            codes::DISCIPLINE_OPS,
            format!(
                "Invalid crop box (out of range): x={} y={} w={} h={}",
                args.crop.x, args.crop.y, args.crop.w, args.crop.h
            ),
        ));
    }

    let img = image::load_from_memory(&src_bytes).map_err(|error| {
        ralph_errors::err_string(
            codes::FILESYSTEM,
            format!(
                "Failed to decode discipline image '{}' at {}: {error}",
                args.discipline_name,
                abs_path.display()
            ),
        )
    })?;
    let (iw, ih) = (img.width(), img.height());

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let (sx, sy, sw, sh) = (
        (args.crop.x * iw as f32) as u32,
        (args.crop.y * ih as f32) as u32,
        (args.crop.w * iw as f32).min((iw as f32) - (args.crop.x * iw as f32)) as u32,
        (args.crop.h * ih as f32).min((ih as f32) - (args.crop.y * ih as f32)) as u32,
    );

    if sw == 0 || sh == 0 {
        return Err(ralph_errors::err_string(
            codes::DISCIPLINE_OPS,
            format!(
                "Crop box produced empty image (discipline='{}', label='{}', x={} y={} w={} h={})",
                args.discipline_name,
                args.label,
                args.crop.x,
                args.crop.y,
                args.crop.w,
                args.crop.h
            ),
        ));
    }

    let cropped = img.crop_imm(sx, sy, sw, sh);

    let mut buf = Cursor::new(Vec::new());
    cropped
        .write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|error| {
            ralph_errors::err_string(
                codes::FILESYSTEM,
                format!(
                    "Failed to encode crop to PNG (discipline='{}', label='{}'): {error}",
                    args.discipline_name, args.label
                ),
            )
        })?;
    let bytes = buf.into_inner();

    if let Err(error) = std::fs::create_dir_all(&cache_dir) {
        diagnostics::emit_warning(
            "disciplines",
            "crop-cache-write-failed",
            &format!(
                "Failed to create crop cache dir at {}: {error}. Continuing without cache.",
                cache_dir.display()
            ),
        );
    } else if let Err(error) = std::fs::write(&cache_path, &bytes) {
        diagnostics::emit_warning(
            "disciplines",
            "crop-cache-write-failed",
            &format!(
                "Failed to write crop cache at {}: {error}. Continuing without cache.",
                cache_path.display()
            ),
        );
    }

    Ok(Some(
        base64::engine::general_purpose::STANDARD.encode(&bytes),
    ))
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisciplinesImageDataGetArgs {
    pub discipline_name: String,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisciplinesCroppedImageGetArgs {
    pub discipline_name: String,
    pub crop: CropBoxData,
    pub label: String,
}
