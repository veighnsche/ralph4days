use crate::mcp::tools::McpTool;
use crate::recipe::Recipe;
use crate::sections;

pub fn recipe() -> Recipe {
    Recipe {
        name: "yap",
        sections: vec![
            sections::project_context(),
            sections::project_metadata(),
            sections::feature_listing(),
            sections::task_listing(),
            sections::discipline_listing(),
            sections::user_input(),
            sections::yap_instructions(),
        ],
        mcp_tools: vec![
            McpTool::CreateTask,
            McpTool::UpdateTask,
            McpTool::SetTaskStatus,
            McpTool::ListTasks,
            McpTool::ListFeatures,
        ],
    }
}
