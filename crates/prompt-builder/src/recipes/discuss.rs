use crate::mcp::tools::McpTool;
use crate::recipe::Recipe;
use crate::sections;

pub fn recipe() -> Recipe {
    Recipe {
        name: "discuss",
        sections: vec![
            sections::project_context(),
            sections::project_metadata(),
            sections::discipline_listing(),
            sections::user_input(),
            sections::discuss_instructions(),
        ],
        mcp_tools: vec![
            McpTool::UpdateDiscipline,
            McpTool::ListDisciplines,
        ],
    }
}
