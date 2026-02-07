use crate::context::PromptContext;
use crate::error::PromptError;
use crate::mcp;
use crate::mcp::tools::McpTool;
use crate::output::PromptOutput;

/// A named prompt section. Takes full context, returns None to skip.
pub struct Section {
    pub name: &'static str,
    pub build: fn(&PromptContext) -> Option<String>,
}

/// A complete prompt recipe: which sections in which order + which MCP tools.
pub struct Recipe {
    pub name: &'static str,
    pub sections: Vec<Section>,
    pub mcp_tools: Vec<McpTool>,
}

/// Execute a recipe: run each section, concatenate non-None results.
pub fn execute_recipe(recipe: &Recipe, ctx: &PromptContext) -> Result<PromptOutput, PromptError> {
    let mut prompt = String::new();
    for section in &recipe.sections {
        if let Some(text) = (section.build)(ctx) {
            prompt.push_str(&text);
            prompt.push_str("\n\n");
        }
    }
    // Trim trailing whitespace
    let prompt = prompt.trim_end().to_string();

    let (mcp_scripts, mcp_config_json) = mcp::generate(ctx, &recipe.mcp_tools);
    Ok(PromptOutput {
        prompt,
        mcp_scripts,
        mcp_config_json,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::test_context;

    fn dummy_section_a(_ctx: &PromptContext) -> Option<String> {
        Some("## Section A\nHello".to_string())
    }

    fn dummy_section_b(_ctx: &PromptContext) -> Option<String> {
        None // skipped
    }

    fn dummy_section_c(_ctx: &PromptContext) -> Option<String> {
        Some("## Section C\nWorld".to_string())
    }

    #[test]
    fn execute_recipe_concatenates_and_skips_none() {
        let recipe = Recipe {
            name: "test",
            sections: vec![
                Section {
                    name: "a",
                    build: dummy_section_a,
                },
                Section {
                    name: "b",
                    build: dummy_section_b,
                },
                Section {
                    name: "c",
                    build: dummy_section_c,
                },
            ],
            mcp_tools: vec![],
        };
        let ctx = test_context();
        let output = execute_recipe(&recipe, &ctx).unwrap();
        assert!(output.prompt.contains("## Section A"));
        assert!(output.prompt.contains("## Section C"));
        assert!(!output.prompt.contains("Section B"));
    }
}
