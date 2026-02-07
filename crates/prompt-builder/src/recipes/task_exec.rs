use crate::mcp::tools::McpTool;
use crate::recipe::Recipe;
use crate::sections;

pub fn recipe() -> Recipe {
    Recipe {
        name: "task_execution",
        sections: vec![
            // Order matters for recency bias — most important last
            sections::project_context(),
            sections::discipline_persona(),
            sections::feature_context(),
            sections::feature_files(),
            sections::feature_state(),
            sections::state_files(),
            sections::previous_attempts(),
            sections::dependency_context(),
            sections::task_details(),
            sections::task_files(),
            sections::task_exec_instructions(),
        ],
        mcp_tools: vec![
            McpTool::SetTaskStatus,
            McpTool::AppendLearning,
            McpTool::AddContextFile,
        ],
    }
}
