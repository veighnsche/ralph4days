use crate::context::PromptContext;
use crate::recipe::Section;

fn build(ctx: &PromptContext) -> Option<String> {
    let task = ctx.target_task()?;

    if task.context_files.is_empty() {
        return None;
    }

    let mut out = String::from("## Task Context Files\n\n");
    let mut found_any = false;

    for path in &task.context_files {
        if let Some(content) = ctx.file_contents.get(path) {
            out.push_str(&format!("### {path}\n\n```\n{content}\n```\n\n"));
            found_any = true;
        }
    }

    if !found_any {
        return None;
    }

    Some(out.trim_end().to_string())
}

pub fn task_files() -> Section {
    Section {
        name: "task_files",
        build,
    }
}
