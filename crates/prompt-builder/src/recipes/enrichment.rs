use crate::mcp::tools::McpTool;
use crate::recipe::Recipe;
use crate::sections;

pub fn recipe() -> Recipe {
    Recipe {
        name: "enrichment",
        sections: vec![
            sections::project_context(),
            sections::codebase_state(),
            sections::feature_context(),
            sections::feature_state(),
            sections::state_files(),
            sections::dependency_context(),
            sections::task_details(),
            sections::task_listing(),
            sections::enrichment_instructions(),
        ],
        mcp_tools: vec![
            McpTool::EnrichTask,
            McpTool::UpdateTask,
            McpTool::CreateTask,
            McpTool::ListTasks,
        ],
    }
}
