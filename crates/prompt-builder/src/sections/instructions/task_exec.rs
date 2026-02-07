use crate::context::PromptContext;
use crate::recipe::Section;

pub fn default_text() -> String {
    r#"## Instructions

You are executing a specific task. Complete it thoroughly, following the discipline conventions and acceptance criteria.

### What to do

1. **Read the task details** above carefully. Understand the title, description, acceptance criteria, and hints.
2. **Follow the discipline conventions** specified in the "You Are" section.
3. **Implement the work** described in the task. Use context files and reference documents as guides.
4. **Verify acceptance criteria** are met before marking the task complete.
5. **Update task status** to `done` using the `update_task` MCP tool when complete.
6. **Commit your changes** with a descriptive commit message summarizing what was done.
7. **Append a summary** to `progress.txt` describing what you accomplished in this iteration.

### Rules

- Work on **ONE task only** per iteration. Do not start other tasks.
- If you encounter a blocker, update the task status to `blocked` with a `blocked_by` explanation and stop.
- If ALL tasks in the project are now complete, output `<promise>COMPLETE</promise>` at the end of your response.
- Do not modify files outside the scope of your assigned task unless absolutely necessary.
- If a dependency task is not yet complete, do not proceed -- mark yourself as blocked."#
        .to_string()
}

fn build(ctx: &PromptContext) -> Option<String> {
    if let Some(text) = ctx.instruction_overrides.get("task_exec_instructions") {
        return Some(text.clone());
    }
    Some(default_text())
}

pub fn task_exec_instructions() -> Section {
    Section {
        name: "task_exec_instructions",
        build,
    }
}
