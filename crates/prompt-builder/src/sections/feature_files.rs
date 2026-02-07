use crate::context::PromptContext;
use crate::recipe::Section;

fn build(ctx: &PromptContext) -> Option<String> {
    let feature = ctx.target_task_feature()?;

    let mut knowledge_sections = String::new();
    for path in &feature.knowledge_paths {
        if let Some(content) = ctx.file_contents.get(path) {
            knowledge_sections.push_str(&format!("### {path}\n\n```\n{content}\n```\n\n"));
        }
    }

    let mut source_sections = String::new();
    for path in &feature.context_files {
        if let Some(content) = ctx.file_contents.get(path) {
            source_sections.push_str(&format!("### {path}\n\n```\n{content}\n```\n\n"));
        }
    }

    if knowledge_sections.is_empty() && source_sections.is_empty() {
        return None;
    }

    let mut out = String::new();

    if !knowledge_sections.is_empty() {
        out.push_str("## Reference Documents\n\n");
        out.push_str(&knowledge_sections);
    }

    if !source_sections.is_empty() {
        out.push_str("## Source Files\n\n");
        out.push_str(&source_sections);
    }

    Some(out.trim_end().to_string())
}

pub fn feature_files() -> Section {
    Section {
        name: "feature_files",
        build,
    }
}
