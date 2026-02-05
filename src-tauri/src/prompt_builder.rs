use crate::types::RalphError;
use std::path::Path;

const COMPLETION_MARKER: &str = "<promise>COMPLETE</promise>";

pub struct PromptBuilder;

impl PromptBuilder {
    pub fn build_haiku_prompt(project_path: &Path) -> Result<String, RalphError> {
        let ralph_dir = project_path.join(".ralph");

        if !ralph_dir.exists() {
            return Err(RalphError::MissingRalphDir);
        }

        // Read PRD from database
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
        prompt.push_str("Pick ONE incomplete task (marked with [ ]) and complete it.\n\n");
        prompt.push_str("After completing the task:\n");
        prompt.push_str("1. Mark it as done by changing [ ] to [x] in prd.yaml\n");
        prompt.push_str("2. Commit your changes with a descriptive message\n");
        prompt.push_str("3. Append a brief summary to progress.txt\n\n");
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

        // Read PRD from database
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

    /// Read PRD content from .ralph/db/ files
    /// Raw YAML is fine since Claude reads it as text context
    fn read_prd_content(ralph_dir: &Path) -> Result<String, RalphError> {
        let db_path = ralph_dir.join("db");

        let metadata = Self::read_file(&db_path.join("metadata.yaml"), "db/metadata.yaml")?;
        let tasks = Self::read_file(&db_path.join("tasks.yaml"), "db/tasks.yaml")?;

        Ok(format!("{}\n{}", metadata, tasks))
    }

    fn read_file(path: &Path, name: &str) -> Result<String, RalphError> {
        std::fs::read_to_string(path).map_err(|_| RalphError::MissingFile(name.to_string()))
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
