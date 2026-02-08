use serde::Serialize;

use crate::context::PromptContext;
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

/// A rendered prompt section with its name and content.
#[derive(Debug, Clone, Serialize)]
pub struct PromptSection {
    pub name: String,
    pub content: String,
}

/// Build sections from an arbitrary list of section names.
pub fn build_sections_from_names(
    section_names: &[&str],
    ctx: &PromptContext,
) -> Vec<PromptSection> {
    section_names
        .iter()
        .filter_map(|name| {
            let section = crate::sections::get_section(name)?;
            (section.build)(ctx).map(|c| PromptSection {
                name: (*name).to_owned(),
                content: c,
            })
        })
        .collect()
}

/// Build each section individually, returning name+content pairs.
pub fn build_sections(recipe: &Recipe, ctx: &PromptContext) -> Vec<PromptSection> {
    recipe
        .sections
        .iter()
        .filter_map(|s| {
            (s.build)(ctx).map(|c| PromptSection {
                name: s.name.to_owned(),
                content: c,
            })
        })
        .collect()
}

/// Execute a recipe: run each section, concatenate non-None results.
#[tracing::instrument(skip(recipe, ctx), fields(recipe_name = recipe.name))]
pub fn execute_recipe(recipe: &Recipe, ctx: &PromptContext) -> PromptOutput {
    tracing::debug!(
        section_count = recipe.sections.len(),
        mcp_tool_count = recipe.mcp_tools.len(),
        "Executing prompt recipe"
    );

    let mut prompt = String::new();
    let mut sections_built = 0;
    for section in &recipe.sections {
        if let Some(text) = (section.build)(ctx) {
            tracing::trace!(
                section_name = section.name,
                content_len = text.len(),
                "Section built"
            );
            prompt.push_str(&text);
            prompt.push_str("\n\n");
            sections_built += 1;
        } else {
            tracing::trace!(
                section_name = section.name,
                "Section skipped (returned None)"
            );
        }
    }
    // Trim trailing whitespace
    let prompt = prompt.trim_end().to_owned();

    tracing::debug!(
        sections_built,
        total_sections = recipe.sections.len(),
        prompt_length = prompt.len(),
        "Prompt sections built"
    );

    let (mcp_scripts, mcp_config_json) = mcp::generate(ctx, &recipe.mcp_tools);

    tracing::info!(
        recipe_name = recipe.name,
        prompt_length = prompt.len(),
        mcp_scripts_count = mcp_scripts.len(),
        "Prompt recipe executed successfully"
    );

    PromptOutput {
        prompt,
        mcp_scripts,
        mcp_config_json,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::test_context;

    #[allow(clippy::unnecessary_wraps)]
    fn dummy_section_a(_ctx: &PromptContext) -> Option<String> {
        Some("## Section A\nHello".to_owned())
    }

    fn dummy_section_b(_ctx: &PromptContext) -> Option<String> {
        None
    }

    #[allow(clippy::unnecessary_wraps)]
    fn dummy_section_c(_ctx: &PromptContext) -> Option<String> {
        Some("## Section C\nWorld".to_owned())
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
        let output = execute_recipe(&recipe, &ctx);
        assert!(output.prompt.contains("## Section A"));
        assert!(output.prompt.contains("## Section C"));
        assert!(!output.prompt.contains("Section B"));
    }
}
