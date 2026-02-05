# Ralph Project Standards

| Field | Value |
|-------|-------|
| **Spec ID** | SPEC-030 |
| **Title** | Ralph Project Standards |
| **Status** | Active |
| **Version** | 1.0.0 |
| **Created** | 2026-02-05 |
| **Author** | Vince Liem |
| **Co-Author** | Claude Sonnet 4.5 |

---

## 1. Purpose

This specification defines the standard structure and conventions for target projects managed by Ralph Loop. It establishes:

- Required directory structure (`.ralph/` folder contents)
- File naming conventions to avoid conflicts
- Context file management during loop execution
- Backup and restoration procedures

## 2. Scope

This specification applies to all projects that will be managed by Ralph Loop. It does NOT apply to the Ralph application itself, but to the target projects Ralph operates on.

## 3. Definitions

| Term | Definition |
|------|------------|
| **Target Project** | A software project that Ralph Loop operates on (not Ralph itself) |
| **Ralph Directory** | The `.ralph/` folder within a target project containing Ralph-specific files |
| **Context File** | A file providing context to Claude during loop execution |
| **CLAUDE.md** | The standard filename that Claude CLI reads for project context |
| **CLAUDE.RALPH.md** | Ralph-specific context file that avoids naming conflicts |

## 4. Required Directory Structure

### 4.1 Minimal Valid Project

A valid Ralph project MUST have the following structure:

```
target-project/
├── .ralph/
│   └── prd.yaml              # REQUIRED: Product Requirements Document
└── ... (project files)
```

### 4.2 Complete Project Structure

A fully-featured Ralph project SHOULD have:

```
target-project/
├── .ralph/
│   ├── prd.yaml              # REQUIRED: Task list with [ ] / [x] checkboxes
│   ├── CLAUDE.RALPH.md     # RECOMMENDED: Ralph-specific context for Claude
│   ├── progress.txt        # AUTO-GENERATED: Iteration log (appended after each)
│   └── learnings.txt       # OPTIONAL: Patterns, gotchas, accumulated wisdom
├── CLAUDE.md               # OPTIONAL: Project's original context file (if exists)
└── ... (project files)
```

## 5. File Naming Standards

### 5.1 Context File Convention

| Rule ID | Requirement |
|---------|-------------|
| FILE-01 | Ralph-specific context MUST be named `CLAUDE.RALPH.md` (not `CLAUDE.md`) |
| FILE-02 | This prevents conflicts with projects that already have `CLAUDE.md` |
| FILE-03 | The `.RALPH.` infix clearly indicates Ralph-specific content |
| FILE-04 | All Ralph context files MUST live in `.ralph/CLAUDE.RALPH.md` |

### 5.2 Rationale

Many projects (including Ralph itself) already use `CLAUDE.md` in their project root for Claude Code context. Using `.ralph/CLAUDE.md` would create:

1. **Confusion**: Which `CLAUDE.md` takes precedence?
2. **Conflicts**: Same filename in different locations
3. **Overwrites**: Risk of losing original project context

Using `CLAUDE.RALPH.md` makes the purpose explicit and avoids all conflicts.

## 6. Context File Management During Loops

### 6.1 Backup and Inject Procedure

When Ralph Loop **starts**, it MUST perform the following sequence:

| Step | Action | Command/Logic |
|------|--------|---------------|
| 1 | Check if `CLAUDE.md` exists in project root | `if exists(CLAUDE.md)` |
| 2 | If yes, create backup | `cp CLAUDE.md CLAUDE.md.ralph-backup` |
| 3 | Check if `CLAUDE.RALPH.md` exists in `.ralph/` | `if exists(.ralph/CLAUDE.RALPH.md)` |
| 4 | If yes, copy to project root | `cp .ralph/CLAUDE.RALPH.md CLAUDE.md` |
| 5 | If no, create empty placeholder | `touch CLAUDE.md` (optional) |
| 6 | Proceed with loop execution | Start Claude CLI subprocess |

**Implementation Location**: `src-tauri/src/loop_engine.rs` in `start()` method, before first Claude CLI invocation.

### 6.2 Restore Procedure

When Ralph Loop **completes** (success, abort, stop, or error), it MUST restore the original state:

| Step | Action | Command/Logic |
|------|--------|---------------|
| 1 | Check if backup exists | `if exists(CLAUDE.md.ralph-backup)` |
| 2 | If yes, restore original | `mv CLAUDE.md.ralph-backup CLAUDE.md` |
| 3 | If no backup, check if we created new file | Track creation in loop state |
| 4 | If we created it, remove it | `rm CLAUDE.md` (only if Ralph created it) |
| 5 | If existed before Ralph, leave as-is | No action |

**Implementation Location**: `src-tauri/src/loop_engine.rs` in cleanup logic, called from `stop()`, `abort()`, and error handlers.

### 6.3 Error Handling

| Rule ID | Requirement |
|---------|-------------|
| ERR-01 | If backup creation fails, Ralph MUST abort before starting loop |
| ERR-02 | If restore fails, Ralph MUST log error and alert user |
| ERR-03 | Backup file MUST NOT be deleted until after successful restore |
| ERR-04 | If Ralph crashes, backup MUST remain so user can manually restore |

## 7. File Content Standards

### 7.1 prd.yaml Format

See [SPEC-035: PRD Format Standard](./035_PRD_FORMAT.md) for complete specification.

The `prd.yaml` file MUST:

- Use YAML 1.2 format
- Include `schema_version: "1.0"`
- Include at least one task with valid status
- Be parseable by serde_yaml

Minimal example:
```yaml
schema_version: "1.0"
project:
  title: "My Project"

tasks:
  - id: "task-001"
    title: "Implement user authentication"
    status: "pending"
```

### 7.2 CLAUDE.RALPH.md Format

The `CLAUDE.RALPH.md` file SHOULD:

- Provide project-specific context Claude needs
- Include architectural decisions
- Reference coding standards
- Link to relevant documentation

This file is freeform markdown. Ralph injects it into `CLAUDE.md` so Claude CLI can read it.

### 7.3 progress.txt Format

Auto-generated by Ralph after each iteration:

```
=== Iteration 1 ===
Timestamp: 2026-02-05 10:30:15
Status: Success
Duration: 45s
Changes: Added authentication module

=== Iteration 2 ===
...
```

### 7.4 learnings.txt Format

Freeform text. Teams can add notes about:
- Patterns discovered
- Things that didn't work
- Gotchas to remember
- Best practices

## 8. Implementation Checklist

### 8.1 Validation Changes

Update `validate_project_path()` in `src-tauri/src/commands.rs`:

- [x] Check `.ralph/` directory exists (already done)
- [x] Check `.ralph/prd.yaml` exists (already done)
- [ ] Do NOT require `CLAUDE.RALPH.md` (optional file)

### 8.2 Loop Engine Changes

Update `src-tauri/src/loop_engine.rs`:

- [ ] Add `original_claude_md_existed: bool` field to `LoopEngine` struct
- [ ] In `start()`: Implement backup procedure (section 6.1)
- [ ] In `stop()`, `abort()`, error handlers: Implement restore procedure (section 6.2)
- [ ] Add logging for backup/restore operations
- [ ] Handle backup failure as abort condition

### 8.3 Prompt Builder Changes

Update `src-tauri/src/prompt_builder.rs`:

- [ ] Change file path from `.ralph/CLAUDE.md` to `.ralph/CLAUDE.RALPH.md`
- [ ] Make context injection optional (don't error if file missing)
- [ ] Log when context file is found vs. not found

## 9. Migration Path

### 9.1 Existing Projects

Projects that already use `.ralph/CLAUDE.md` should:

1. Rename `.ralph/CLAUDE.md` → `.ralph/CLAUDE.RALPH.md`
2. Update any documentation referencing the old name
3. Delete any backup files created by old Ralph versions

### 9.2 Fixture Updates

Update test fixtures:

- [ ] Rename `fixtures/single-task/.ralph/CLAUDE.md` → `CLAUDE.RALPH.md`
- [ ] Update fixture documentation
- [ ] Add test fixture with existing `CLAUDE.md` to verify backup/restore

## 10. Examples

### 10.1 Example: Project Without Existing CLAUDE.md

**Before Ralph Start:**
```
my-project/
├── .ralph/
│   ├── prd.yaml
│   └── CLAUDE.RALPH.md
└── src/
```

**During Ralph Loop:**
```
my-project/
├── .ralph/
│   ├── prd.yaml
│   └── CLAUDE.RALPH.md
├── CLAUDE.md              # Created by Ralph (copy of CLAUDE.RALPH.md)
└── src/
```

**After Ralph Stop:**
```
my-project/
├── .ralph/
│   ├── prd.yaml
│   └── CLAUDE.RALPH.md
└── src/                   # CLAUDE.md removed (Ralph created it)
```

### 10.2 Example: Project With Existing CLAUDE.md

**Before Ralph Start:**
```
my-project/
├── .ralph/
│   ├── prd.yaml
│   └── CLAUDE.RALPH.md
├── CLAUDE.md              # Original project context
└── src/
```

**During Ralph Loop:**
```
my-project/
├── .ralph/
│   ├── prd.yaml
│   └── CLAUDE.RALPH.md
├── CLAUDE.md              # Overwritten with CLAUDE.RALPH.md content
├── CLAUDE.md.ralph-backup # Original backed up
└── src/
```

**After Ralph Stop:**
```
my-project/
├── .ralph/
│   ├── prd.yaml
│   └── CLAUDE.RALPH.md
├── CLAUDE.md              # Restored from backup
└── src/                   # .ralph-backup removed after restore
```

## 11. Testing Requirements

### 11.1 Unit Tests

- [ ] Test backup creation when `CLAUDE.md` exists
- [ ] Test no backup when `CLAUDE.md` does not exist
- [ ] Test restore from backup on successful completion
- [ ] Test restore from backup on error/abort
- [ ] Test removal of created file (no backup case)

### 11.2 Integration Tests

- [ ] E2E test: Ralph loop with existing `CLAUDE.md`, verify restore
- [ ] E2E test: Ralph loop without `CLAUDE.md`, verify cleanup
- [ ] E2E test: Simulated crash, verify backup remains

### 11.3 Manual Tests

- [ ] Start Ralph on project with `CLAUDE.md`, verify backup created
- [ ] Check during loop that context from `CLAUDE.RALPH.md` is active
- [ ] Stop loop, verify original `CLAUDE.md` restored
- [ ] Check backup file was removed after restore

## 12. Backwards Compatibility

### 12.1 Breaking Change Notice

This is a **breaking change** for projects using `.ralph/CLAUDE.md`:

- Old filename: `.ralph/CLAUDE.md`
- New filename: `.ralph/CLAUDE.RALPH.md`

Projects MUST rename their context files or Ralph will not load the context.

### 12.2 Detection and Warning

Ralph SHOULD:

- Detect if `.ralph/CLAUDE.md` exists (old naming)
- Emit warning: "Found .ralph/CLAUDE.md (deprecated). Please rename to CLAUDE.RALPH.md"
- Continue without loading context (don't silently use old file)

Implementation location: `src-tauri/src/prompt_builder.rs`

## 13. Documentation Updates Required

- [ ] Update `CLAUDE.md` (Ralph's own) to document this standard
- [ ] Update "Target Project Structure" section
- [ ] Add migration instructions for existing users
- [ ] Update `PROJECT_LOCK_IMPLEMENTATION.md` if it references context files
- [ ] Update fixture README files

## 14. Traceability

| Requirement | Implementation File | Test File |
|-------------|---------------------|-----------|
| FILE-01 | `src-tauri/src/prompt_builder.rs` | TBD |
| Backup (6.1) | `src-tauri/src/loop_engine.rs::start()` | TBD |
| Restore (6.2) | `src-tauri/src/loop_engine.rs::stop()` | TBD |
| ERR-01 | `src-tauri/src/loop_engine.rs::start()` | TBD |

## 15. Open Questions

None. Specification is complete and ready for implementation.

## 16. References

- [SPEC-000: Specification Format](./000_SPECIFICATION_FORMAT.md)
- [SPEC-010: Traceability Standard](./010_TRACEABILITY.md)
- [SPEC-060: Testing Standards](./060_TESTING_STANDARDS.md)
- Ralph Loop CLAUDE.md (project instructions)
