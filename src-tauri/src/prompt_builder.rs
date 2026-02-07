// TODO: Prompt Builder - Generate context-rich prompts for Claude
//
// This module builds prompts for the autonomous loop. Need to add new prompt types:
//
// TODO LIST:
// 1. build_braindump_prompt(braindump_text: &str, ralph_db: &Path) -> Result<String>
//    - System instructions: "You are helping structure a project braindump..."
//    - Explain available MCP tools (create_task, create_feature, etc.)
//    - Tell Claude to ask clarifying questions if needed
//    - Include braindump text
//    - Request structured output (features, disciplines, tasks)
//
// 2. build_task_execution_prompt(task_id: &str, ralph_db: &Path) -> Result<String>
//    - Read task from SQLite database
//    - Include task title, description, acceptance criteria
//    - Include feature context from database
//    - Include discipline context from database
//    - List completed dependencies
//    - Include project files overview (ls -R or similar)
//    - Tell Claude to use update_task_status tool when done
//
// 3. build_yap_prompt(user_rambling: &str, ralph_db: &Path) -> Result<String>
//    - List existing tasks from database
//    - Include user's thoughts about what to change
//    - Explain update_task and create_task tools
//    - Ask Claude to clarify ambiguities
//
// 4. build_ramble_prompt(user_rambling: &str, ralph_db: &Path) -> Result<String>
//    - List existing features from database
//    - Include user's thoughts about features
//    - Explain update_feature and create_feature tools
//
// 5. Include CLAUDE.RALPH.md in all prompts
//    - Read .ralph/CLAUDE.RALPH.md if it exists
//    - Inject as project-specific context
//    - Gives Claude project-specific knowledge

use crate::types::RalphError;
use sqlite_db::SqliteDb;
use std::path::Path;

const COMPLETION_MARKER: &str = "<promise>COMPLETE</promise>";

pub struct PromptBuilder;

impl PromptBuilder {
    pub fn build_haiku_prompt(project_path: &Path) -> Result<String, RalphError> {
        let ralph_dir = project_path.join(".ralph");

        if !ralph_dir.exists() {
            return Err(RalphError::MissingRalphDir);
        }

        // Read PRD from SQLite database
        let prd = Self::read_prd_content(&ralph_dir)?;
        let progress = Self::read_file_optional(&ralph_dir.join("progress.txt"));
        let learnings = Self::read_file_optional(&ralph_dir.join("learnings.txt"));
        let claude_md = Self::read_file_optional(&ralph_dir.join("CLAUDE.md"));

        let mut prompt = String::new();

        if let Some(context) = claude_md {
            prompt.push_str("## Project Context\n\n");
            prompt.push_str(&context);
            prompt.push_str("\n\n");
        }

        prompt.push_str("## PRD (Task List)\n\n");
        prompt.push_str(&prd);
        prompt.push_str("\n\n");

        if let Some(prog) = progress {
            prompt.push_str("## Progress Log\n\n");
            prompt.push_str(&prog);
            prompt.push_str("\n\n");
        }

        if let Some(learn) = learnings {
            prompt.push_str("## Learnings & Patterns\n\n");
            prompt.push_str(&learn);
            prompt.push_str("\n\n");
        }

        prompt.push_str("## Instructions\n\n");
        prompt.push_str("You are working on tasks from the PRD above. ");
        prompt.push_str(
            "Pick ONE incomplete task (status: todo or in-progress) and complete it.\n\n",
        );
        prompt.push_str("After completing the task:\n");
        prompt.push_str("1. Update its status to 'done' in the project database\n");
        prompt.push_str("2. Commit your changes with a descriptive message\n");
        prompt.push_str("3. Append a brief summary to .ralph/progress.txt\n\n");
        prompt.push_str("If ALL tasks are complete, output exactly: ");
        prompt.push_str(COMPLETION_MARKER);
        prompt.push_str(
            "\n\nIMPORTANT: Only work on ONE task per iteration. Be thorough but focused.",
        );

        Ok(prompt)
    }

    pub fn build_opus_review_prompt(project_path: &Path) -> Result<String, RalphError> {
        let ralph_dir = project_path.join(".ralph");

        if !ralph_dir.exists() {
            return Err(RalphError::MissingRalphDir);
        }

        // Read PRD from SQLite database
        let prd = Self::read_prd_content(&ralph_dir)?;
        let progress = Self::read_file_optional(&ralph_dir.join("progress.txt"));
        let learnings = Self::read_file_optional(&ralph_dir.join("learnings.txt"));

        let mut prompt = String::new();

        prompt.push_str("## Code Review Task\n\n");
        prompt.push_str("You are reviewing progress on a project. Review the recent work and:\n\n");
        prompt.push_str("1. Check for any bugs, issues, or code quality problems\n");
        prompt.push_str("2. Verify the completed tasks actually work correctly\n");
        prompt.push_str("3. Add any important patterns or gotchas to learnings.txt\n");
        prompt.push_str("4. Fix any issues you find\n\n");

        prompt.push_str("## PRD (Task List)\n\n");
        prompt.push_str(&prd);
        prompt.push_str("\n\n");

        if let Some(prog) = progress {
            prompt.push_str("## Recent Progress\n\n");
            prompt.push_str(&prog);
            prompt.push_str("\n\n");
        }

        if let Some(learn) = learnings {
            prompt.push_str("## Current Learnings\n\n");
            prompt.push_str(&learn);
            prompt.push_str("\n\n");
        }

        prompt.push_str("Focus on quality over speed. Fix any issues before they compound.");

        Ok(prompt)
    }

    pub fn check_completion(output: &str) -> bool {
        output.contains(COMPLETION_MARKER)
    }

    /// Read PRD content from SQLite database at .ralph/db/ralph.db
    fn read_prd_content(ralph_dir: &Path) -> Result<String, RalphError> {
        let db_path = ralph_dir.join("db").join("ralph.db");
        let db = SqliteDb::open(&db_path)
            .map_err(|e| RalphError::MissingFile(format!("db/ralph.db: {}", e)))?;
        db.export_prd_yaml()
            .map_err(|e| RalphError::MissingFile(format!("export failed: {}", e)))
    }

    fn read_file_optional(path: &Path) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }
}

pub fn hash_content(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}
