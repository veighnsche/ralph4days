/// What the prompt builder produces.
pub struct PromptOutput {
    pub prompt: String,
    pub mcp_scripts: Vec<McpScript>,
    pub mcp_config_json: String,
}

/// A generated bash MCP server script.
pub struct McpScript {
    pub filename: String,
    pub content: String,
}
