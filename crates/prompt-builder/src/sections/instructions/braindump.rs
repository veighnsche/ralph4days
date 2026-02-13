use crate::context::PromptContext;
use crate::recipe::Section;

pub fn default_text() -> String {
    "## Instructions

You are receiving a raw braindump from the user. Your job is to analyze it and create structured project data.

### What to do

1. **Read the braindump carefully.** Identify distinct subsystems, areas of work, and concrete tasks.
2. **Create subsystems** using the available subsystem-management MCP tools in this session. Group related work into cohesive subsystems. Each subsystem should have a clear name, display name, and description.
3. **Create or update disciplines** using the available discipline-management MCP tools if the work requires disciplines beyond the defaults. Configure system_prompt, skills, and conventions for each.
4. **Create tasks** using the available task-management MCP tools. Tasks are created as **drafts** by default. Each task should:
   - Belong to exactly one subsystem and one discipline
   - Have a clear, actionable title
   - Have a brief description of intent (1-2 sentences max)
   - Set appropriate priority (low, medium, high, critical)
   - Specify dependencies on other tasks via `depends_on` where ordering matters
5. **Ask clarifying questions** if the braindump is ambiguous or incomplete. It is better to ask than to guess wrong.

### Guidelines

- Prefer many small, focused tasks over few large ones
- Each task should be completable in a single Claude session (1-10 turns)
- **Do NOT write detailed descriptions, acceptance criteria, or pseudocode.** Tasks start as drafts and get enriched with concrete implementation details later, when the codebase state is known.
- Focus on task titles, ordering, and dependencies — the structure of work, not the details
- Create dependencies between tasks when one must complete before another can start".to_owned()
}

#[allow(clippy::unnecessary_wraps)]
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
