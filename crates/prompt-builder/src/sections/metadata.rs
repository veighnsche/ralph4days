use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SectionInfo {
    pub name: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub is_instruction: bool,
}

pub fn all_sections() -> Vec<SectionInfo> {
    vec![
        SectionInfo {
            name: "project_context",
            display_name: "Project Context",
            description: "CLAUDE.RALPH.md and project-level context",
            category: "project",
            is_instruction: false,
        },
        SectionInfo {
            name: "project_metadata",
            display_name: "Project Metadata",
            description: "Project title, description, and creation date",
            category: "project",
            is_instruction: false,
        },
        SectionInfo {
            name: "codebase_state",
            display_name: "Codebase Snapshot",
            description: "Filesystem tree and tech stack analysis",
            category: "project",
            is_instruction: false,
        },
        SectionInfo {
            name: "feature_listing",
            display_name: "Feature Listing",
            description: "All features with descriptions and stats",
            category: "feature",
            is_instruction: false,
        },
        SectionInfo {
            name: "feature_context",
            display_name: "Feature Context",
            description: "Target feature details, architecture, and learnings",
            category: "feature",
            is_instruction: false,
        },
        SectionInfo {
            name: "feature_files",
            display_name: "Feature Files",
            description: "Inlined contents of feature context files",
            category: "feature",
            is_instruction: false,
        },
        SectionInfo {
            name: "feature_state",
            display_name: "Feature State",
            description: "Tasks grouped by status for the target feature",
            category: "feature",
            is_instruction: false,
        },
        SectionInfo {
            name: "task_listing",
            display_name: "Task Listing",
            description: "All tasks with status, priority, and dependencies",
            category: "task",
            is_instruction: false,
        },
        SectionInfo {
            name: "task_details",
            display_name: "Task Details",
            description: "Full details of the target task",
            category: "task",
            is_instruction: false,
        },
        SectionInfo {
            name: "task_files",
            display_name: "Task Files",
            description: "Inlined contents of task context files",
            category: "task",
            is_instruction: false,
        },
        SectionInfo {
            name: "dependency_context",
            display_name: "Dependency Context",
            description: "Details of tasks this task depends on",
            category: "task",
            is_instruction: false,
        },
        SectionInfo {
            name: "previous_attempts",
            display_name: "Previous Attempts",
            description: "Comments from prior iterations on this task",
            category: "task",
            is_instruction: false,
        },
        SectionInfo {
            name: "discipline_listing",
            display_name: "Discipline Listing",
            description: "All disciplines with skills and conventions",
            category: "discipline",
            is_instruction: false,
        },
        SectionInfo {
            name: "discipline_persona",
            display_name: "Discipline Persona",
            description: "System prompt and identity for the target discipline",
            category: "discipline",
            is_instruction: false,
        },
        SectionInfo {
            name: "state_files",
            display_name: "State Files",
            description: "Contents of progress.txt and learnings.txt",
            category: "state",
            is_instruction: false,
        },
        SectionInfo {
            name: "user_input",
            display_name: "User Input",
            description: "Raw text from the user (braindump, yap, etc.)",
            category: "user",
            is_instruction: false,
        },
        SectionInfo {
            name: "braindump_instructions",
            display_name: "Braindump Instructions",
            description: "Instructions for structuring a raw braindump into tasks",
            category: "instructions",
            is_instruction: true,
        },
        SectionInfo {
            name: "yap_instructions",
            display_name: "Yap Instructions",
            description: "Instructions for creating/updating tasks from user input",
            category: "instructions",
            is_instruction: true,
        },
        SectionInfo {
            name: "ramble_instructions",
            display_name: "Ramble Instructions",
            description: "Instructions for creating/updating features from user input",
            category: "instructions",
            is_instruction: true,
        },
        SectionInfo {
            name: "discuss_instructions",
            display_name: "Discuss Instructions",
            description: "Instructions for updating discipline configurations",
            category: "instructions",
            is_instruction: true,
        },
        SectionInfo {
            name: "task_exec_instructions",
            display_name: "Task Exec Instructions",
            description: "Instructions for executing a specific task",
            category: "instructions",
            is_instruction: true,
        },
        SectionInfo {
            name: "opus_review_instructions",
            display_name: "Opus Review Instructions",
            description: "Instructions for reviewing recent work quality",
            category: "instructions",
            is_instruction: true,
        },
    ]
}

pub fn get_info(name: &str) -> Option<SectionInfo> {
    all_sections().into_iter().find(|s| s.name == name)
}
