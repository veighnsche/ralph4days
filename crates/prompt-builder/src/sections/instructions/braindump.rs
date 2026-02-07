use crate::context::PromptContext;
use crate::recipe::Section;

pub fn default_text() -> String {
    r#"## Instructions

You are receiving a raw braindump from the user. Your job is to analyze it and create structured project data.

### What to do

1. **Read the braindump carefully.** Identify distinct features, areas of work, and concrete tasks.
2. **Create features** using the `create_feature` MCP tool. Group related work into cohesive features. Each feature should have a clear name, display name, and description.
3. **Create or update disciplines** using the `create_discipline` MCP tool if the work requires disciplines beyond the defaults. Configure system_prompt, skills, and conventions for each.
4. **Create tasks** using the `create_task` MCP tool. Each task should:
   - Belong to exactly one feature and one discipline
   - Have a clear, actionable title
   - Include a description explaining what needs to be done
   - List specific acceptance criteria
   - Set appropriate priority (low, medium, high, critical)
   - Specify dependencies on other tasks via `depends_on` where ordering matters
5. **Ask clarifying questions** if the braindump is ambiguous or incomplete. It is better to ask than to guess wrong.

### Guidelines

- Prefer many small, focused tasks over few large ones
- Each task should be completable in a single Claude session (1-10 turns)
- Set `estimated_turns` to help with scheduling
- Use `context_files` to point tasks at the relevant source files
- Use `hints` to give the executing agent useful tips
- Create dependencies between tasks when one must complete before another can start"#
        .to_string()
}

fn build(ctx: &PromptContext) -> Option<String> {
    if let Some(text) = ctx.instruction_overrides.get("braindump_instructions") {
        return Some(text.clone());
    }
    Some(default_text())
}

pub fn braindump_instructions() -> Section {
    Section {
        name: "braindump_instructions",
        build,
    }
}
