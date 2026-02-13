use super::state::{AppState, CommandContext};
use ralph_errors::{codes, RalphResultExt};
use ralph_macros::ipc_type;
use serde::Deserialize;
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
}

#[tauri::command]
pub fn get_disciplines_config(state: State<'_, AppState>) -> Result<Vec<DisciplineConfig>, String> {
    CommandContext::from_tauri_state(&state).db(|db| {
        Ok(db
            .get_disciplines()
            .iter()
            .map(|d| DisciplineConfig {
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
            })
            .collect())
    })
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureCommentData {
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
pub struct FeatureData {
    pub id: u32,
    pub name: String,
    pub display_name: String,
    pub acronym: String,
    pub description: Option<String>,
    pub created: Option<String>,
    pub status: String,
    pub comments: Vec<FeatureCommentData>,
}

fn to_comment_data(c: &sqlite_db::FeatureComment) -> FeatureCommentData {
    FeatureCommentData {
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

#[tauri::command]
pub fn get_features(state: State<'_, AppState>) -> Result<Vec<FeatureData>, String> {
    CommandContext::from_tauri_state(&state).db(|db| {
        Ok(db
            .get_features()
            .iter()
            .map(|f| FeatureData {
                id: f.id,
                name: f.name.clone(),
                display_name: f.display_name.clone(),
                acronym: f.acronym.clone(),
                description: f.description.clone(),
                created: f.created.clone(),
                status: f.status.as_str().to_owned(),
                comments: f.comments.iter().map(to_comment_data).collect(),
            })
            .collect())
    })
}

#[derive(Deserialize)]
pub struct CreateFeatureParams {
    pub name: String,
    pub display_name: String,
    pub acronym: String,
    pub description: Option<String>,
}

#[tauri::command]
pub fn create_feature(
    state: State<'_, AppState>,
    params: CreateFeatureParams,
) -> Result<(), String> {
    CommandContext::from_tauri_state(&state).db(|db| {
        db.create_feature(sqlite_db::FeatureInput {
            name: params.name,
            display_name: params.display_name,
            acronym: params.acronym,
            description: params.description,
        })
    })
}

#[derive(Deserialize)]
pub struct UpdateFeatureParams {
    pub name: String,
    pub display_name: String,
    pub acronym: String,
    pub description: Option<String>,
}

#[tauri::command]
pub fn update_feature(
    state: State<'_, AppState>,
    params: UpdateFeatureParams,
) -> Result<(), String> {
    CommandContext::from_tauri_state(&state).db(|db| {
        db.update_feature(sqlite_db::FeatureInput {
            name: params.name,
            display_name: params.display_name,
            acronym: params.acronym,
            description: params.description,
        })
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddFeatureCommentParams {
    pub feature_name: String,
    pub category: String,
    pub discipline: Option<String>,
    pub agent_task_id: Option<u32>,
    pub body: String,
    pub summary: Option<String>,
    pub reason: Option<String>,
    pub source_iteration: Option<u32>,
}

#[tauri::command]
pub async fn add_feature_comment(
    state: State<'_, AppState>,
    params: AddFeatureCommentParams,
) -> Result<(), String> {
    let command_ctx = CommandContext::from_tauri_state(&state);
    let path = db_path(&command_ctx)?;
    let (comment_id, embedding_text) = command_ctx.db_tx(|db| {
        db.add_feature_comment(sqlite_db::AddFeatureCommentInput {
            feature_name: params.feature_name.clone(),
            category: params.category.clone(),
            discipline: params.discipline.clone(),
            agent_task_id: params.agent_task_id,
            body: params.body.clone(),
            summary: params.summary.clone(),
            reason: params.reason.clone(),
            source_iteration: params.source_iteration,
        })?;

        let features = db.get_features();
        let cid = features
            .iter()
            .find(|f| f.name == params.feature_name)
            .and_then(|f| f.comments.last())
            .map(|c| c.id)
            .ok_or("Failed to get new comment ID")?;

        let text = ralph_external::comment_embeddings::build_embedding_text(
            &params.category,
            &params.body,
            params.reason.as_deref(),
        );
        Ok((cid, text))
    })?;

    let ext_config = ralph_external::ExternalServicesConfig::load()?;
    let embed_config = build_embedding_config(&ext_config);
    let result =
        ralph_external::comment_embeddings::embed_text(&embed_config, &embedding_text).await?;

    let db = sqlite_db::SqliteDb::open(&path, None)?;
    db.upsert_comment_embedding(comment_id, &result.vector, &result.model, &result.hash)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFeatureCommentParams {
    pub feature_name: String,
    pub comment_id: u32,
    pub body: String,
    pub summary: Option<String>,
    pub reason: Option<String>,
}

#[tauri::command]
pub async fn update_feature_comment(
    state: State<'_, AppState>,
    params: UpdateFeatureCommentParams,
) -> Result<(), String> {
    let command_ctx = CommandContext::from_tauri_state(&state);
    let path = db_path(&command_ctx)?;
    let (embedding_text, needs_embed) = command_ctx.db_tx(|db| {
        db.update_feature_comment(
            &params.feature_name,
            params.comment_id,
            &params.body,
            params.summary.clone(),
            params.reason.clone(),
        )?;

        let features = db.get_features();
        let category = features
            .iter()
            .find(|f| f.name == params.feature_name)
            .and_then(|f| f.comments.iter().find(|c| c.id == params.comment_id))
            .map(|c| c.category.clone())
            .ok_or("Comment not found after update")?;

        let text = ralph_external::comment_embeddings::build_embedding_text(
            &category,
            &params.body,
            params.reason.as_deref(),
        );
        let needs = ralph_external::comment_embeddings::should_embed(
            db,
            params.comment_id,
            &category,
            &params.body,
            params.reason.as_deref(),
        )
        .is_some();
        Ok((text, needs))
    })?;

    if !needs_embed {
        return Ok(());
    }

    let ext_config = ralph_external::ExternalServicesConfig::load()?;
    let embed_config = build_embedding_config(&ext_config);
    let result =
        ralph_external::comment_embeddings::embed_text(&embed_config, &embedding_text).await?;

    let db = sqlite_db::SqliteDb::open(&path, None)?;
    db.upsert_comment_embedding(
        params.comment_id,
        &result.vector,
        &result.model,
        &result.hash,
    )
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFeatureCommentParams {
    pub feature_name: String,
    pub comment_id: u32,
}

#[tauri::command]
pub fn delete_feature_comment(
    state: State<'_, AppState>,
    params: DeleteFeatureCommentParams,
) -> Result<(), String> {
    CommandContext::from_tauri_state(&state)
        .db(|db| db.delete_feature_comment(&params.feature_name, params.comment_id))
}

#[derive(Deserialize)]
pub struct CreateDisciplineParams {
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
    pub skills: Option<Vec<String>>,
    pub conventions: Option<String>,
    pub mcp_servers: Option<Vec<McpServerConfigData>>,
}

#[tauri::command]
pub fn create_discipline(
    state: State<'_, AppState>,
    params: CreateDisciplineParams,
) -> Result<(), String> {
    let normalized_name = params
        .name
        .to_lowercase()
        .trim()
        .replace(char::is_whitespace, "-");

    let skills_json = serde_json::to_string(&params.skills.unwrap_or_default())
        .ralph_err(codes::DISCIPLINE_OPS, "Failed to serialize skills")?;

    let mcp_servers: Vec<sqlite_db::McpServerConfig> = params
        .mcp_servers
        .unwrap_or_default()
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
            display_name: params.display_name,
            acronym: params.acronym,
            icon: params.icon,
            color: params.color,
            description: None,
            system_prompt: params.system_prompt,
            agent: params.agent,
            model: params.model,
            effort: params.effort,
            thinking: params.thinking,
            skills: skills_json,
            conventions: params.conventions,
            mcp_servers: mcp_json,
            image_path: None,
            crops: None,
            image_prompt: None,
        })
    })
}

#[derive(Deserialize)]
pub struct UpdateDisciplineParams {
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
    pub skills: Option<Vec<String>>,
    pub conventions: Option<String>,
    pub mcp_servers: Option<Vec<McpServerConfigData>>,
}

#[tauri::command]
pub fn update_discipline(
    state: State<'_, AppState>,
    params: UpdateDisciplineParams,
) -> Result<(), String> {
    let skills_json = serde_json::to_string(&params.skills.unwrap_or_default())
        .ralph_err(codes::DISCIPLINE_OPS, "Failed to serialize skills")?;

    let mcp_servers: Vec<sqlite_db::McpServerConfig> = params
        .mcp_servers
        .unwrap_or_default()
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
        db.update_discipline(sqlite_db::DisciplineInput {
            name: params.name,
            display_name: params.display_name,
            acronym: params.acronym,
            icon: params.icon,
            color: params.color,
            description: None,
            system_prompt: params.system_prompt,
            agent: params.agent,
            model: params.model,
            effort: params.effort,
            thinking: params.thinking,
            skills: skills_json,
            conventions: params.conventions,
            mcp_servers: mcp_json,
            image_path: None,
            crops: None,
            image_prompt: None,
        })
    })
}

#[tauri::command]
pub fn delete_feature(state: State<'_, AppState>, name: String) -> Result<(), String> {
    CommandContext::from_tauri_state(&state).db(|db| db.delete_feature(name))
}

#[tauri::command]
pub fn delete_discipline(state: State<'_, AppState>, name: String) -> Result<(), String> {
    CommandContext::from_tauri_state(&state).db(|db| db.delete_discipline(name))
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
pub fn get_stack_metadata() -> Vec<StackMetadataData> {
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
pub fn get_discipline_image_data(
    state: State<'_, AppState>,
    name: String,
) -> Result<Option<String>, String> {
    use base64::Engine;

    let disc = CommandContext::from_tauri_state(&state)
        .db(|db| Ok(db.get_disciplines().into_iter().find(|d| d.name == name)))?;

    let Some(disc) = disc else {
        return Ok(None);
    };
    let Some(ref image_path) = disc.image_path else {
        return Ok(None);
    };

    let project_path = CommandContext::from_tauri_state(&state).locked_project_path()?;
    let abs_path = project_path.join(".ralph").join(image_path);

    std::fs::read(&abs_path).map_or(Ok(None), |bytes| {
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        Ok(Some(b64))
    })
}

#[tauri::command]
pub fn get_cropped_image(
    state: State<'_, AppState>,
    name: String,
    crop: CropBoxData,
    label: String,
) -> Result<Option<String>, String> {
    use base64::Engine;

    let disc = CommandContext::from_tauri_state(&state)
        .db(|db| Ok(db.get_disciplines().into_iter().find(|d| d.name == name)))?;

    let Some(disc) = disc else {
        return Ok(None);
    };
    let Some(ref image_path) = disc.image_path else {
        return Ok(None);
    };

    let project_path = CommandContext::from_tauri_state(&state).locked_project_path()?;
    let cache_dir = project_path.join(".ralph").join("cache").join("crops");
    let cache_key = format!(
        "{}_{}_{}_{}_{}_{}.png",
        name, label, crop.x, crop.y, crop.w, crop.h
    );
    let cache_path = cache_dir.join(&cache_key);

    if cache_path.exists() {
        return std::fs::read(&cache_path).map_or(Ok(None), |bytes| {
            Ok(Some(
                base64::engine::general_purpose::STANDARD.encode(&bytes),
            ))
        });
    }

    let abs_path = project_path.join(".ralph").join(image_path);
    let Ok(src_bytes) = std::fs::read(&abs_path) else {
        return Ok(None);
    };

    let img = image::load_from_memory(&src_bytes).map_err(|e| e.to_string())?;
    let (iw, ih) = (img.width(), img.height());

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let (sx, sy, sw, sh) = (
        (crop.x * iw as f32) as u32,
        (crop.y * ih as f32) as u32,
        (crop.w * iw as f32).min((iw as f32) - (crop.x * iw as f32)) as u32,
        (crop.h * ih as f32).min((ih as f32) - (crop.y * ih as f32)) as u32,
    );

    if sw == 0 || sh == 0 {
        return Ok(None);
    }

    let cropped = img.crop_imm(sx, sy, sw, sh);

    std::fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;
    cropped.save(&cache_path).map_err(|e| e.to_string())?;

    std::fs::read(&cache_path).map_or(Ok(None), |bytes| {
        Ok(Some(
            base64::engine::general_purpose::STANDARD.encode(&bytes),
        ))
    })
}
