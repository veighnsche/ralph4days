use crate::mcp::tools::McpTool;
use crate::mcp::McpMode;
use crate::recipe::Recipe;
use crate::sections;

pub fn recipe() -> Recipe {
    Recipe {
        name: "braindump",
        sections: vec![
            sections::project_context(),
            sections::project_metadata(),
            sections::codebase_state(),
            sections::feature_listing(),
            sections::discipline_listing(),
            sections::user_input(),
            sections::braindump_instructions(),
        ],
        mcp_mode: McpMode::BashTools,
        mcp_tools: vec![
            McpTool::CreateFeature,
            McpTool::CreateDiscipline,
            McpTool::CreateTask,
            McpTool::ListFeatures,
            McpTool::ListDisciplines,
        ],
    }
}
