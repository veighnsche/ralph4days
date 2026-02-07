use crate::context::PromptContext;
use crate::recipe::Section;

pub fn default_text() -> String {
    r#"## Instructions

You are receiving input from the user about disciplines. Review the existing disciplines and the user's input, then update discipline configurations as needed.

### What to do

1. **Review existing disciplines** listed above to understand current configuration.
2. **Interpret the user's input** about discipline changes.
3. **Update disciplines** using the `update_discipline` MCP tool to modify configurations.
4. **Create new disciplines** using the `create_discipline` MCP tool if the user describes new roles.

### Focus areas

- **system_prompt**: The persona and instructions for agents working in this discipline. Should define the agent's role, expertise, and approach.
- **skills**: A list of specific capabilities the discipline brings (e.g., "TypeScript", "API design", "performance optimization").
- **conventions**: Coding standards, patterns, and practices the discipline enforces (e.g., "use early returns", "prefer composition over inheritance").
- **mcp_servers**: Additional MCP servers the discipline needs for specialized tooling.

### Guidelines

- System prompts should be detailed enough to guide a Claude agent effectively
- Skills should be specific and actionable, not vague
- Conventions should be concrete rules, not aspirational statements
- Keep discipline scope focused -- one discipline should not cover everything"#.to_owned()
}

#[allow(clippy::unnecessary_wraps)]
fn build(ctx: &PromptContext) -> Option<String> {
    if let Some(text) = ctx.instruction_overrides.get("discuss_instructions") {
        return Some(text.clone());
    }
    Some(default_text())
}

pub fn discuss_instructions() -> Section {
    Section {
        name: "discuss_instructions",
        build,
    }
}
