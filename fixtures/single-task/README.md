# Single Task Fixture

Minimal smoke test - one simple file creation task.

## Task

Create `hello.txt` containing "Hello, World!"

## Purpose

Quick sanity check that Ralph can complete a basic task in one iteration.

## Expected Behavior

1. Ralph reads `prd.md` and finds one unchecked task
2. Claude creates `hello.txt` with the correct content
3. Claude marks the task as complete: `[x]` in `prd.md`
4. Claude appends to `progress.txt`
5. Ralph detects completion via `<promise>COMPLETE</promise>` marker

## Files

- `.ralph/prd.md` - Task definition (checkbox format)
- `.ralph/CLAUDE.md` - Context for Claude
- `.ralph/progress.txt` - Created during execution
- `hello.txt` - Created by Claude (output)
