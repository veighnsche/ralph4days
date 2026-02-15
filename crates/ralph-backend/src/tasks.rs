use ralph_errors::{codes, err_string, RalphResult};
use ralph_macros::ipc_type;
use sqlite_db::SqliteDb;

fn get_task_or_error(db: &SqliteDb, id: u32) -> RalphResult<sqlite_db::Task> {
    db.get_task_by_id(id)?.ok_or_else(|| {
        err_string(
            codes::TASK_OPS,
            format!("Task {id} not found after mutation"),
        )
    })
}

fn validate_subsystem_name(name: &str) -> RalphResult<()> {
    if name.contains('/') || name.contains(':') || name.contains('\\') {
        return Err(err_string(
            codes::TASK_VALIDATION,
            "Subsystem name cannot contain /, :, or \\\\",
        ));
    }
    Ok(())
}

fn normalize_subsystem_name(name: &str) -> String {
    // Mirror src/lib/acronym.ts normalizeFeatureName:
    // - lowercase
    // - trim
    // - collapse whitespace runs to "-"
    // - collapse "_" runs to "-"
    let lower = name.to_lowercase();
    let trimmed = lower.trim();
    let whitespace_normalized = trimmed
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    let mut out = String::with_capacity(whitespace_normalized.len());
    let mut in_underscores = false;
    for ch in whitespace_normalized.chars() {
        if ch == '_' {
            if !in_underscores {
                out.push('-');
                in_underscores = true;
            }
        } else {
            out.push(ch);
            in_underscores = false;
        }
    }
    out
}

#[ipc_type]
pub struct TasksCreateArgs {
    pub subsystem: String,
    pub discipline: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<sqlite_db::Priority>,
    pub tags: Vec<String>,
    pub depends_on: Vec<u32>,
    pub acceptance_criteria: Vec<String>,
    pub context_files: Vec<String>,
    pub output_artifacts: Vec<String>,
    pub hints: Option<String>,
    pub estimated_turns: Option<u32>,
    pub provenance: Option<sqlite_db::TaskProvenance>,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub thinking: Option<bool>,
}

#[ipc_type]
pub struct TasksUpdateArgs {
    pub id: u32,
    pub subsystem: String,
    pub discipline: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<sqlite_db::Priority>,
    pub tags: Vec<String>,
    pub depends_on: Vec<u32>,
    pub acceptance_criteria: Vec<String>,
    pub context_files: Vec<String>,
    pub output_artifacts: Vec<String>,
    pub hints: Option<String>,
    pub estimated_turns: Option<u32>,
    pub provenance: Option<sqlite_db::TaskProvenance>,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub thinking: Option<bool>,
}

#[ipc_type]
pub struct TasksSetStatusArgs {
    pub id: u32,
    pub status: String,
}

#[ipc_type]
pub struct TasksDeleteArgs {
    pub id: u32,
}

#[ipc_type]
pub struct TasksGetArgs {
    pub id: u32,
}

#[ipc_type]
pub struct TasksSignalAddArgs {
    pub task_id: u32,
    pub discipline: Option<String>,
    pub agent_task_id: Option<u32>,
    pub priority: Option<String>,
    pub body: String,
}

#[ipc_type]
pub struct TasksSignalUpdateArgs {
    pub task_id: u32,
    pub signal_id: u32,
    pub body: String,
}

#[ipc_type]
pub struct TasksSignalDeleteArgs {
    pub task_id: u32,
    pub signal_id: u32,
}

#[ipc_type]
pub struct TasksSignalSummariesGetArgs {
    pub task_ids: Vec<u32>,
}

#[ipc_type]
pub struct TasksAskAnswerArgs {
    pub signal_id: u32,
    pub answer: String,
}

#[ipc_type]
pub struct TasksCommentReplyAddArgs {
    pub task_id: u32,
    pub parent_comment_id: u32,
    pub priority: Option<String>,
    pub body: String,
}

#[ipc_type]
pub struct TasksSignalCommentUpdateArgs {
    pub comment_id: u32,
    pub body: String,
}

#[ipc_type]
pub struct TasksSignalCommentDeleteArgs {
    pub comment_id: u32,
}

#[ipc_type]
pub struct TasksSignalCommentsListArgs {
    pub signal_id: u32,
}

pub fn tasks_create(db: &SqliteDb, args: TasksCreateArgs) -> RalphResult<String> {
    validate_subsystem_name(&args.subsystem)?;
    let normalized_subsystem = normalize_subsystem_name(&args.subsystem);

    let task_input = sqlite_db::TaskInput {
        subsystem: normalized_subsystem,
        discipline: args.discipline,
        title: args.title,
        description: args.description,
        status: None,
        priority: args.priority,
        tags: args.tags,
        depends_on: args.depends_on,
        acceptance_criteria: Some(args.acceptance_criteria),
        context_files: args.context_files,
        output_artifacts: args.output_artifacts,
        hints: args.hints,
        estimated_turns: args.estimated_turns,
        provenance: args.provenance,
        agent: args.agent,
        model: args.model,
        effort: args.effort,
        thinking: args.thinking,
    };

    let task_id = db.create_task(task_input)?;
    Ok(task_id.to_string())
}

pub fn tasks_update(db: &SqliteDb, args: TasksUpdateArgs) -> RalphResult<sqlite_db::Task> {
    validate_subsystem_name(&args.subsystem)?;
    let normalized_subsystem = normalize_subsystem_name(&args.subsystem);

    let task_id = args.id;
    let task_input = sqlite_db::TaskInput {
        subsystem: normalized_subsystem,
        discipline: args.discipline,
        title: args.title,
        description: args.description,
        status: None,
        priority: args.priority,
        tags: args.tags,
        depends_on: args.depends_on,
        acceptance_criteria: Some(args.acceptance_criteria),
        context_files: args.context_files,
        output_artifacts: args.output_artifacts,
        hints: args.hints,
        estimated_turns: args.estimated_turns,
        provenance: args.provenance,
        agent: args.agent,
        model: args.model,
        effort: args.effort,
        thinking: args.thinking,
    };

    db.update_task(task_id, task_input)?;
    get_task_or_error(db, task_id)
}

pub fn tasks_set_status(db: &SqliteDb, args: TasksSetStatusArgs) -> RalphResult<sqlite_db::Task> {
    let status = sqlite_db::TaskStatus::parse(&args.status).ok_or_else(|| {
        err_string(
            codes::TASK_VALIDATION,
            format!("Invalid status: {}", args.status),
        )
    })?;

    db.set_task_status(args.id, status)?;
    get_task_or_error(db, args.id)
}

pub fn tasks_delete(db: &SqliteDb, args: TasksDeleteArgs) -> RalphResult<()> {
    db.delete_task(args.id)
}

pub fn tasks_signal_add(db: &SqliteDb, args: TasksSignalAddArgs) -> RalphResult<sqlite_db::Task> {
    db.add_signal(
        args.task_id,
        args.discipline,
        args.agent_task_id,
        args.priority,
        args.body,
    )?;
    get_task_or_error(db, args.task_id)
}

pub fn tasks_signal_update(
    db: &SqliteDb,
    args: TasksSignalUpdateArgs,
) -> RalphResult<sqlite_db::Task> {
    db.update_signal(args.task_id, args.signal_id, args.body)?;
    get_task_or_error(db, args.task_id)
}

pub fn tasks_signal_delete(
    db: &SqliteDb,
    args: TasksSignalDeleteArgs,
) -> RalphResult<sqlite_db::Task> {
    db.delete_signal(args.task_id, args.signal_id)?;
    get_task_or_error(db, args.task_id)
}

pub fn tasks_list(db: &SqliteDb) -> RalphResult<Vec<sqlite_db::Task>> {
    db.get_tasks()
}

pub fn tasks_get(db: &SqliteDb, args: TasksGetArgs) -> RalphResult<sqlite_db::Task> {
    get_task_or_error(db, args.id)
}

pub fn tasks_list_items(db: &SqliteDb) -> RalphResult<Vec<sqlite_db::TaskListItem>> {
    db.get_task_list_items()
}

pub fn tasks_signal_summaries_get(
    db: &SqliteDb,
    args: TasksSignalSummariesGetArgs,
) -> RalphResult<std::collections::HashMap<u32, sqlite_db::TaskSignalSummary>> {
    db.get_signal_summaries(&args.task_ids)
}

pub fn tasks_ask_answer(db: &SqliteDb, args: TasksAskAnswerArgs) -> RalphResult<()> {
    db.answer_ask(args.signal_id, args.answer)
}

pub fn tasks_comment_reply_add(
    db: &SqliteDb,
    args: TasksCommentReplyAddArgs,
) -> RalphResult<sqlite_db::Task> {
    db.add_signal_with_parent(
        args.task_id,
        None,
        args.priority,
        args.body,
        Some(args.parent_comment_id),
    )?;
    get_task_or_error(db, args.task_id)
}

pub fn tasks_signal_comment_add(
    db: &SqliteDb,
    args: sqlite_db::TaskSignalCommentCreateInput,
) -> RalphResult<u32> {
    db.add_task_signal_comment(args)
}

pub fn tasks_signal_comment_update(
    db: &SqliteDb,
    args: TasksSignalCommentUpdateArgs,
) -> RalphResult<()> {
    db.update_task_signal_comment(args.comment_id, args.body)
}

pub fn tasks_signal_comment_delete(
    db: &SqliteDb,
    args: TasksSignalCommentDeleteArgs,
) -> RalphResult<()> {
    db.delete_task_signal_comment(args.comment_id)
}

pub fn tasks_signal_comments_list(
    db: &SqliteDb,
    args: TasksSignalCommentsListArgs,
) -> RalphResult<Vec<sqlite_db::TaskSignalComment>> {
    db.get_task_signal_comments(args.signal_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subsystem_name_validation_rejects_forbidden_characters() {
        let err = validate_subsystem_name("bad/name").unwrap_err();
        let rendered = err.to_string();
        assert!(rendered.contains("[R-3000]"));
        assert!(rendered.contains("Subsystem name cannot contain"));

        assert!(validate_subsystem_name("also:bad").is_err());
        assert!(validate_subsystem_name(r"also\\bad").is_err());
    }

    #[test]
    fn subsystem_name_normalization_matches_frontend_semantics() {
        assert_eq!(normalize_subsystem_name("  My Feature  "), "my-feature");
        assert_eq!(normalize_subsystem_name("My__Feature"), "my-feature");
        assert_eq!(normalize_subsystem_name("My _ Feature"), "my---feature");
    }
}
