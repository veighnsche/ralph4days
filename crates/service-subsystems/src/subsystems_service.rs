use crate::subsystems_contract::{
    SubsystemCommentData, SubsystemData, SubsystemsCommentAddArgs, SubsystemsCommentDeleteArgs,
    SubsystemsCommentUpdateArgs, SubsystemsCreateArgs, SubsystemsDeleteArgs, SubsystemsUpdateArgs,
};
use core_errors::{codes, err_string, RalphResult};
use data_sqlite::SqliteDb;
use std::path::{Path, PathBuf};

pub struct SubsystemCommentEmbeddingWork {
    pub comment_id: u32,
    pub embedding_text: String,
}

fn build_embedding_config(
    ext_config: &ai_external::ExternalServicesConfig,
) -> ai_external::comment_embeddings::CommentEmbeddingConfig<'_> {
    ai_external::comment_embeddings::CommentEmbeddingConfig {
        ollama: &ext_config.ollama,
        document_prefix: "search_document: ",
        query_prefix: "search_query: ",
        min_search_score: 0.4,
        max_search_results: 10,
    }
}

fn comment_embeddings_db_path(project_path: &Path) -> PathBuf {
    project_path.join(".ralph").join("db").join("ralph.db")
}

fn to_comment_data(c: &data_sqlite::SubsystemComment) -> SubsystemCommentData {
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

fn to_subsystem_data(subsystem: &data_sqlite::Subsystem) -> SubsystemData {
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

fn get_subsystem_data_or_error(db: &SqliteDb, name: &str) -> RalphResult<SubsystemData> {
    let subsystems = db.get_subsystems()?;
    let subsystem = subsystems
        .iter()
        .find(|f| f.name == name)
        .ok_or_else(|| err_string(codes::FEATURE_OPS, format!("Subsystem '{name}' not found")))?;
    Ok(to_subsystem_data(subsystem))
}

pub fn subsystems_list(db: &SqliteDb) -> RalphResult<Vec<SubsystemData>> {
    Ok(db.get_subsystems()?.iter().map(to_subsystem_data).collect())
}

pub fn subsystems_create(db: &SqliteDb, args: SubsystemsCreateArgs) -> RalphResult<SubsystemData> {
    let name = args.name.clone();
    db.create_subsystem(data_sqlite::SubsystemInput {
        name: args.name,
        display_name: args.display_name,
        acronym: args.acronym,
        description: args.description,
    })?;
    get_subsystem_data_or_error(db, &name)
}

pub fn subsystems_update(db: &SqliteDb, args: SubsystemsUpdateArgs) -> RalphResult<SubsystemData> {
    let name = args.name.clone();
    db.update_subsystem(data_sqlite::SubsystemInput {
        name: args.name,
        display_name: args.display_name,
        acronym: args.acronym,
        description: args.description,
    })?;
    get_subsystem_data_or_error(db, &name)
}

pub fn subsystems_delete(db: &SqliteDb, args: SubsystemsDeleteArgs) -> RalphResult<()> {
    db.delete_subsystem(args.name)
}

pub fn subsystems_comment_add_prepare(
    db: &SqliteDb,
    args: SubsystemsCommentAddArgs,
) -> RalphResult<(SubsystemData, SubsystemCommentEmbeddingWork)> {
    let subsystem_name = args.subsystem_name.clone();

    db.with_transaction(|db| {
        let comment_id = db.add_subsystem_comment(data_sqlite::AddSubsystemCommentInput {
            subsystem_name: args.subsystem_name,
            category: args.category.clone(),
            discipline: args.discipline,
            agent_task_id: args.agent_task_id,
            body: args.body.clone(),
            summary: args.summary.clone(),
            reason: args.reason.clone(),
            source_iteration: args.source_iteration,
        })?;

        let subsystem = get_subsystem_data_or_error(db, &subsystem_name)?;

        let embedding_text = ai_external::comment_embeddings::build_embedding_text(
            &args.category,
            &args.body,
            args.reason.as_deref(),
        );

        Ok((
            subsystem,
            SubsystemCommentEmbeddingWork {
                comment_id,
                embedding_text,
            },
        ))
    })
}

pub fn subsystems_comment_update_prepare(
    db: &SqliteDb,
    args: SubsystemsCommentUpdateArgs,
) -> RalphResult<(SubsystemData, Option<SubsystemCommentEmbeddingWork>)> {
    let comment_id = args.comment_id;
    db.with_transaction(|db| {
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
            .ok_or_else(|| err_string(codes::FEATURE_OPS, "Comment not found after update"))?;

        let embedding_text = ai_external::comment_embeddings::should_embed(
            db,
            comment_id,
            &category,
            &args.body,
            args.reason.as_deref(),
        )?;

        Ok((
            subsystem,
            embedding_text.map(|text| SubsystemCommentEmbeddingWork {
                comment_id,
                embedding_text: text,
            }),
        ))
    })
}

pub async fn subsystems_comment_apply_embedding(
    project_path: &Path,
    work: SubsystemCommentEmbeddingWork,
) -> RalphResult<()> {
    let ext_config = ai_external::ExternalServicesConfig::load()
        .map_err(|e| err_string(codes::FEATURE_OPS, e))?;
    let embed_config = build_embedding_config(&ext_config);
    let result = ai_external::comment_embeddings::embed_text(&embed_config, &work.embedding_text)
        .await
        .map_err(|e| err_string(codes::FEATURE_OPS, e))?;

    let path = comment_embeddings_db_path(project_path);
    let db = SqliteDb::open(&path, None)?;
    db.upsert_comment_embedding(work.comment_id, &result.vector, &result.model, &result.hash)?;
    Ok(())
}

pub fn subsystems_comment_delete(
    db: &SqliteDb,
    args: SubsystemsCommentDeleteArgs,
) -> RalphResult<SubsystemData> {
    db.delete_subsystem_comment(&args.subsystem_name, args.comment_id)?;
    get_subsystem_data_or_error(db, &args.subsystem_name)
}
