use crate::context::PromptContext;
use crate::recipe::Section;

pub fn default_text() -> String {
    "## Instructions

You are receiving input from the user about subsystems. Review the existing subsystems and the user's input, then create or update subsystems as needed.

### What to do

1. **Review existing subsystems** listed above to understand current project structure.
2. **Interpret the user's input** about new or changed subsystems.
3. **Create new subsystems** using the available subsystem-management MCP tools where the user describes new areas of work.
4. **Update existing subsystems** using the available subsystem-management MCP tools where the user wants changes.
5. **Consider dependencies** between subsystems and how tasks should be organized.

### Guidelines

- Each subsystem should represent a cohesive area of work
- Use clear, descriptive names that convey the subsystem's purpose
- Set `knowledge_paths` to point at reference documents (specs, designs, docs)
- Set `context_files` to point at the key source files for the subsystem
- If a subsystem is being split or merged, update associated tasks accordingly
- Keep subsystem descriptions concise but informative".to_owned()
}

#[allow(clippy::unnecessary_wraps)]
fn build(ctx: &PromptContext) -> Option<String> {
    if let Some(text) = ctx.instruction_overrides.get("ramble_instructions") {
        return Some(text.clone());
    }
    Some(default_text())
}

pub fn ramble_instructions() -> Section {
    Section {
        name: "ramble_instructions",
        build,
    }
}
