use crate::mcp::tools::McpTool;
use crate::mcp::McpMode;
use crate::recipe::Recipe;
use crate::sections;

pub fn recipe() -> Recipe {
    Recipe {
        name: "opus_review",
        sections: vec![
            sections::project_context(),
            sections::feature_context(),
            sections::feature_files(),
            sections::feature_state(),
            sections::task_listing(),
            sections::state_files(),
            sections::opus_review_instructions(),
        ],
        mcp_mode: McpMode::BashTools,
        mcp_tools: vec![
            McpTool::SetTaskStatus,
            McpTool::AppendLearning,
            McpTool::UpdateFeature,
            McpTool::ListTasks,
        ],
    }
}
