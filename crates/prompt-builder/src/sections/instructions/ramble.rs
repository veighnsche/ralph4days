use crate::context::PromptContext;
use crate::recipe::Section;

pub fn default_text() -> String {
    "## Instructions

You are receiving input from the user about features. Review the existing features and the user's input, then create or update features as needed.

### What to do

1. **Review existing features** listed above to understand current project structure.
2. **Interpret the user's input** about new or changed features.
3. **Create new features** using the `create_feature` MCP tool where the user describes new areas of work.
4. **Update existing features** using the `update_feature` MCP tool where the user wants changes.
5. **Consider dependencies** between features and how tasks should be organized.

### Guidelines

- Each feature should represent a cohesive area of work
- Use clear, descriptive names that convey the feature's purpose
- Set `knowledge_paths` to point at reference documents (specs, designs, docs)
- Set `context_files` to point at the key source files for the feature
- If a feature is being split or merged, update associated tasks accordingly
- Keep feature descriptions concise but informative".to_owned()
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
