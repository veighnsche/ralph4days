use crate::context::PromptContext;
use crate::recipe::Section;

pub fn default_text() -> String {
    r#"## Instructions

You are reviewing recent work for quality. Focus on correctness, code quality, and completeness.

### What to do

1. **Review recently completed tasks.** Check that the work actually implements what the task describes.
2. **Verify the code works.** Run tests, check for compilation errors, and look for obvious bugs.
3. **Check code quality.** Look for:
   - Code that does not follow project conventions
   - Missing error handling
   - Hardcoded values that should be configurable
   - Dead code or unused imports
   - Poor naming or unclear logic
4. **Fix issues you find.** Make corrections directly rather than just noting them.
5. **Update learnings.txt.** Add patterns, gotchas, or useful discoveries to `learnings.txt` so future iterations benefit.
6. **Update task statuses** if you discover a "done" task that is not actually complete -- set it back to `in_progress` or `blocked` as appropriate.

### Guidelines

- Quality over speed. It is better to fix one thing well than to skim many things.
- Be specific in learnings -- "the X pattern causes Y problem, use Z instead" is useful; "be careful with X" is not.
- If you find systemic issues (e.g., a pattern repeated across many files), note it in learnings and fix the instances you find.
- Do not re-do completed work that is correct. Focus on finding and fixing actual problems.
- Commit fixes with clear messages explaining what was wrong and how it was fixed."#.to_owned()
}

#[allow(clippy::unnecessary_wraps)]
fn build(ctx: &PromptContext) -> Option<String> {
    if let Some(text) = ctx.instruction_overrides.get("opus_review_instructions") {
        return Some(text.clone());
    }
    Some(default_text())
}

pub fn opus_review_instructions() -> Section {
    Section {
        name: "opus_review_instructions",
        build,
    }
}
