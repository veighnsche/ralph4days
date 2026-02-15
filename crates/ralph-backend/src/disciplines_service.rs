use crate::disciplines_contract::{
    DisciplineConfig, DisciplineCropsData, DisciplineImagePromptData, DisciplineTaskTemplateData,
    DisciplinesCreateArgs, DisciplinesCroppedImageGetArgs, DisciplinesDeleteArgs,
    DisciplinesImageDataGetArgs, DisciplinesUpdateArgs, McpServerConfigData,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use ralph_errors::{codes, err_string, RalphResult, RalphResultExt};
use sqlite_db::SqliteDb;
use std::io::Cursor;
use std::path::Path;

fn to_discipline_config(db: &SqliteDb, d: &sqlite_db::Discipline) -> RalphResult<DisciplineConfig> {
    Ok(DisciplineConfig {
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
            .map(|s| {
                serde_json::from_str::<DisciplineCropsData>(s).map_err(|e| {
                    err_string(
                        codes::DISCIPLINE_OPS,
                        format!("Invalid crops JSON for discipline '{}': {e}", d.name),
                    )
                })
            })
            .transpose()?,
        image_prompt: d
            .image_prompt
            .as_deref()
            .map(|s| {
                serde_json::from_str::<DisciplineImagePromptData>(s).map_err(|e| {
                    err_string(
                        codes::DISCIPLINE_OPS,
                        format!("Invalid imagePrompt JSON for discipline '{}': {e}", d.name),
                    )
                })
            })
            .transpose()?,
        task_templates: db
            .get_active_task_templates_for_discipline(d.id)?
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
    })
}

fn get_discipline_config_or_error(db: &SqliteDb, name: &str) -> RalphResult<DisciplineConfig> {
    let disciplines = db.get_disciplines()?;
    let discipline = disciplines.iter().find(|d| d.name == name).ok_or_else(|| {
        err_string(
            codes::DISCIPLINE_OPS,
            format!("Discipline '{name}' not found"),
        )
    })?;
    to_discipline_config(db, discipline)
}

pub fn disciplines_list(db: &SqliteDb) -> RalphResult<Vec<DisciplineConfig>> {
    let disciplines = db.get_disciplines()?;
    disciplines
        .iter()
        .map(|d| to_discipline_config(db, d))
        .collect::<Result<Vec<_>, _>>()
}

pub fn disciplines_create(
    db: &SqliteDb,
    args: DisciplinesCreateArgs,
) -> RalphResult<DisciplineConfig> {
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
}

pub fn disciplines_update(
    db: &SqliteDb,
    args: DisciplinesUpdateArgs,
) -> RalphResult<DisciplineConfig> {
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
}

pub fn disciplines_delete(db: &SqliteDb, args: DisciplinesDeleteArgs) -> RalphResult<String> {
    let deleted_name = args.name.clone();
    db.delete_discipline(args.name)?;
    Ok(deleted_name)
}

pub fn disciplines_image_data_get(
    project_path: &Path,
    db: &SqliteDb,
    args: DisciplinesImageDataGetArgs,
) -> RalphResult<Option<String>> {
    let disc = db
        .get_disciplines()?
        .into_iter()
        .find(|d| d.name == args.discipline_name);

    let Some(disc) = disc else {
        return Ok(None);
    };
    let Some(ref image_path) = disc.image_path else {
        return Ok(None);
    };

    let abs_path = project_path.join(".ralph").join(image_path);

    let bytes = std::fs::read(&abs_path).map_err(|error| {
        err_string(
            codes::FILESYSTEM,
            format!(
                "Failed to read discipline image '{}' at {}: {error}",
                args.discipline_name,
                abs_path.display()
            ),
        )
    })?;
    Ok(Some(STANDARD.encode(&bytes)))
}

pub fn disciplines_cropped_image_get(
    project_path: &Path,
    db: &SqliteDb,
    args: DisciplinesCroppedImageGetArgs,
) -> RalphResult<Option<String>> {
    let disc = db
        .get_disciplines()?
        .into_iter()
        .find(|d| d.name == args.discipline_name);

    let Some(disc) = disc else {
        return Err(err_string(
            codes::DISCIPLINE_OPS,
            format!("Discipline '{}' not found", args.discipline_name),
        ));
    };
    let Some(ref image_path) = disc.image_path else {
        return Ok(None);
    };

    let cache_dir = project_path.join(".ralph").join("cache").join("crops");
    let cache_key = format!(
        "{}_{}_{}_{}_{}_{}.png",
        args.discipline_name, args.label, args.crop.x, args.crop.y, args.crop.w, args.crop.h
    );
    let cache_path = cache_dir.join(&cache_key);

    if cache_path.exists() {
        match std::fs::read(&cache_path) {
            Ok(bytes) => {
                return Ok(Some(STANDARD.encode(&bytes)));
            }
            Err(error) => {
                crate::diagnostics::emit_warning(
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
        err_string(
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
        return Err(err_string(
            codes::DISCIPLINE_OPS,
            format!(
                "Invalid crop box (non-finite): x={} y={} w={} h={}",
                args.crop.x, args.crop.y, args.crop.w, args.crop.h
            ),
        ));
    }

    if args.crop.x < 0.0 || args.crop.y < 0.0 || args.crop.w <= 0.0 || args.crop.h <= 0.0 {
        return Err(err_string(
            codes::DISCIPLINE_OPS,
            format!(
                "Invalid crop box (out of range): x={} y={} w={} h={}",
                args.crop.x, args.crop.y, args.crop.w, args.crop.h
            ),
        ));
    }

    let img = image::load_from_memory(&src_bytes).map_err(|error| {
        err_string(
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
        return Err(err_string(
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
            err_string(
                codes::FILESYSTEM,
                format!(
                    "Failed to encode crop to PNG (discipline='{}', label='{}'): {error}",
                    args.discipline_name, args.label
                ),
            )
        })?;
    let bytes = buf.into_inner();

    if let Err(error) = std::fs::create_dir_all(&cache_dir) {
        crate::diagnostics::emit_warning(
            "disciplines",
            "crop-cache-write-failed",
            &format!(
                "Failed to create crop cache dir at {}: {error}. Continuing without cache.",
                cache_dir.display()
            ),
        );
    } else if let Err(error) = std::fs::write(&cache_path, &bytes) {
        crate::diagnostics::emit_warning(
            "disciplines",
            "crop-cache-write-failed",
            &format!(
                "Failed to write crop cache at {}: {error}. Continuing without cache.",
                cache_path.display()
            ),
        );
    }

    Ok(Some(STANDARD.encode(&bytes)))
}
