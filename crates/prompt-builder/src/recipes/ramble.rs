use crate::mcp::tools::McpTool;
use crate::mcp::McpMode;
use crate::recipe::Recipe;
use crate::sections;

pub fn recipe() -> Recipe {
    Recipe {
        name: "ramble",
        sections: vec![
            sections::project_context(),
            sections::project_metadata(),
            sections::feature_listing(),
            sections::feature_state(),
            sections::user_input(),
            sections::ramble_instructions(),
        ],
        mcp_mode: McpMode::BashTools,
        mcp_tools: vec![
            McpTool::CreateFeature,
            McpTool::UpdateFeature,
            McpTool::ListFeatures,
            McpTool::ListTasks,
        ],
    }
}
