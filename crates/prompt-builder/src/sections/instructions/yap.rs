use crate::context::PromptContext;
use crate::recipe::Section;

pub fn default_text() -> String {
    r#"## Instructions

You are receiving additional input from the user about tasks. Review the existing tasks and the user's input, then create new tasks or update existing ones.

### What to do

1. **Review existing tasks** listed above to understand current project state.
2. **Interpret the user's input** in the context of existing features and tasks.
3. **Create new tasks** using the `create_task` MCP tool where the user describes new work.
4. **Update existing tasks** using the `update_task` MCP tool where the user wants changes to current tasks (status, description, priority, acceptance criteria, etc.).
5. **Maintain consistency** with the existing feature and discipline structure.

### Guidelines

- Be specific about acceptance criteria -- vague criteria lead to vague implementations
- Set dependencies (`depends_on`) when tasks have ordering requirements
- Use `context_files` to point tasks at relevant source files
- If the user's input conflicts with existing tasks, ask for clarification
- Preserve existing task data when updating -- only change what the user explicitly requests
- Use `hints` to pass along any useful implementation tips from the user"#
        .to_string()
}

fn build(ctx: &PromptContext) -> Option<String> {
    if let Some(text) = ctx.instruction_overrides.get("yap_instructions") {
        return Some(text.clone());
    }
    Some(default_text())
}

pub fn yap_instructions() -> Section {
    Section {
        name: "yap_instructions",
        build,
    }
}
