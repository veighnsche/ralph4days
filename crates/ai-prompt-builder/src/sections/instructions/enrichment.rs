use crate::context::PromptContext;
use crate::recipe::Section;

pub fn default_text() -> String {
    "## Instructions

You are enriching a draft task with concrete implementation details. The task was created as a lightweight placeholder during braindump. Now the codebase exists and you can write specific pseudocode.

### What to do

1. **Read the codebase state** above carefully. Understand the current file structure and conventions.
2. **Read the task title and description.** Understand the intent.
3. **Write concrete pseudocode** that references actual files, functions, and modules in the codebase. This is implementation guidance for the executing agent — not runnable code.
4. **Set acceptance criteria** that are specific and verifiable.
5. **Set context files** pointing at the actual source files the executing agent will need to read or modify.
6. **Call `enrich_task`** with the pseudocode, acceptance criteria, and context files. This promotes the task from draft to pending.

### Rules

- Do NOT execute the task. Only plan it.
- Do NOT create new tasks or modify other tasks.
- Reference real files and functions from the codebase state — do not guess.
- Keep pseudocode concise. The executing agent has full access to the codebase.
- If the task is unclear or impossible given current codebase state, explain why instead of enriching.".to_owned()
}

#[allow(clippy::unnecessary_wraps)]
fn build(ctx: &PromptContext) -> Option<String> {
    if let Some(text) = ctx.instruction_overrides.get("enrichment_instructions") {
        return Some(text.clone());
    }
    Some(default_text())
}

pub fn enrichment_instructions() -> Section {
    Section {
        name: "enrichment_instructions",
        build,
    }
}
