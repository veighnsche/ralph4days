use crate::context::PromptContext;
use crate::recipe::Section;
use sqlite_db::TaskStatus;

fn build(ctx: &PromptContext) -> Option<String> {
    if ctx.tasks.is_empty() {
        return None;
    }

    let tasks: Vec<_> = match ctx.target_task() {
        Some(task) => ctx
            .tasks
            .iter()
            .filter(|t| t.feature == task.feature)
            .collect(),
        None => ctx.tasks.iter().collect(),
    };

    if tasks.is_empty() {
        return None;
    }

    let total = tasks.len();
    let mut pending = 0u32;
    let mut in_progress = 0u32;
    let mut done = 0u32;
    let mut blocked = 0u32;

    for t in &tasks {
        match t.status {
            TaskStatus::Pending => pending += 1,
            TaskStatus::InProgress => in_progress += 1,
            TaskStatus::Done => done += 1,
            TaskStatus::Blocked => blocked += 1,
            TaskStatus::Skipped => {} // not counted separately
        }
    }

    Some(format!(
        "## Feature State\n\n{done}/{total} tasks complete\n\n\
         - Pending: {pending}\n\
         - In Progress: {in_progress}\n\
         - Done: {done}\n\
         - Blocked: {blocked}"
    ))
}

pub fn feature_state() -> Section {
    Section {
        name: "feature_state",
        build,
    }
}
