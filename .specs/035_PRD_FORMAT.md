# PRD Format Standard

| Field | Value |
|-------|-------|
| **Spec ID** | SPEC-035 |
| **Title** | PRD Format Standard |
| **Status** | Active |
| **Version** | 1.0.0 |
| **Created** | 2026-02-05 |
| **Author** | Vince Liem |
| **Co-Author** | Claude Sonnet 4.5 |

---

## 1. Purpose

This specification defines the Product Requirements Document (PRD) format for Ralph Loop projects. It establishes:

- A structured, machine-parseable format (YAML) instead of markdown
- Task schema with status tracking
- Programmatic update capabilities
- Human readability and editability
- Backwards compatibility considerations

## 2. Scope

This specification applies to all `.ralph/prd.yaml` files in target projects managed by Ralph Loop.

## 3. Problem Statement

The current `prd.md` format has significant limitations:

| Issue | Impact |
|-------|--------|
| **Unstructured** | Must parse markdown to find checkboxes |
| **No validation** | Invalid syntax silently breaks Ralph |
| **Hard to update** | String manipulation to toggle checkboxes is fragile |
| **No metadata** | Can't track priority, dependencies, tags, etc. |
| **No IDs** | Can't reference specific tasks programmatically |
| **Ambiguous completion** | What counts as "done"? Just checkboxes? |

## 4. Solution: YAML-Based PRD

### 4.1 File Naming

| Rule ID | Requirement |
|---------|-------------|
| FILE-01 | PRD files MUST be named `prd.yaml` (not `prd.md`) |
| FILE-02 | Ralph MUST check for `prd.yaml` during project validation |
| FILE-03 | Ralph SHOULD warn if `prd.md` exists (deprecated format) |
| FILE-04 | Ralph MAY provide migration tool: `prd.md` → `prd.yaml` |

### 4.2 Schema Version 1.0

```yaml
# .ralph/prd.yaml
schema_version: "1.0"
project:
  title: "My Project"
  description: "Optional project description"
  created: "2026-02-05"

tasks:
  - id: "task-001"
    title: "Implement user authentication"
    description: "Add JWT-based auth with refresh tokens"
    status: "pending"      # pending | in_progress | done | blocked | skipped
    priority: "high"       # low | medium | high | critical
    tags: ["backend", "security"]
    acceptance_criteria:
      - "User can register with email and password"
      - "User can log in and receive JWT token"
      - "Token refresh endpoint works correctly"
    created: "2026-02-05"
    updated: "2026-02-05"

  - id: "task-002"
    title: "Add database schema"
    description: "PostgreSQL schema with migrations"
    status: "done"
    priority: "medium"
    tags: ["backend", "database"]
    depends_on: ["task-001"]
    created: "2026-02-05"
    updated: "2026-02-05"
    completed: "2026-02-05T10:30:00Z"

  - id: "task-003"
    title: "Set up CI/CD pipeline"
    status: "blocked"
    priority: "low"
    blocked_by: "Waiting for DevOps team access"
    tags: ["infrastructure"]
    created: "2026-02-05"
    updated: "2026-02-05"
```

### 4.3 Field Definitions

#### Project Metadata

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `schema_version` | string | **YES** | Format version (currently "1.0") |
| `project.title` | string | **YES** | Human-readable project name |
| `project.description` | string | no | Optional project description |
| `project.created` | date | no | When PRD was created (YYYY-MM-DD) |

#### Task Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | **YES** | Unique task identifier (e.g., "task-001") |
| `title` | string | **YES** | Short task description (single line) |
| `description` | string | no | Detailed description (can be multi-line) |
| `status` | enum | **YES** | One of: pending, in_progress, done, blocked, skipped |
| `priority` | enum | no | One of: low, medium, high, critical (default: medium) |
| `tags` | array[string] | no | Categorization tags |
| `depends_on` | array[string] | no | Task IDs this task depends on |
| `blocked_by` | string | no | Reason for blocked status |
| `acceptance_criteria` | array[string] | no | List of criteria that define task completion |
| `created` | date | no | When task was created |
| `updated` | date | no | Last update timestamp |
| `completed` | datetime | no | When status changed to "done" (ISO 8601) |

### 4.4 Status State Machine

```
┌─────────┐
│ pending │◄─────────────────┐
└────┬────┘                  │
     │                       │
     ▼                       │
┌─────────────┐              │
│ in_progress │              │
└─────┬───────┘              │
      │                      │
      ├──────►┌────────┐     │
      │       │ blocked│─────┘ (can resume)
      │       └────────┘
      │
      ├──────►┌─────────┐
      │       │ skipped │
      │       └─────────┘
      │
      ▼
┌──────┐
│ done │
└──────┘
```

| Transition | Allowed | Notes |
|------------|---------|-------|
| pending → in_progress | Yes | Ralph starts working on task |
| in_progress → done | Yes | Task completed successfully |
| in_progress → blocked | Yes | Must set `blocked_by` field |
| blocked → pending | Yes | Unblock and return to queue |
| in_progress → skipped | Yes | Explicitly skipped (optional tasks) |
| done → * | No | Completed tasks don't revert |

## 5. Completion Detection

Ralph Loop completes when:

```
ALL tasks have status IN (done, skipped)
AND
AT LEAST ONE task has status = done
```

Alternative completion signal (backwards compatible):
```yaml
completion_marker: true  # Explicit completion flag
```

## 6. Rust Implementation

### 6.1 Serde Structures

```rust
// src-tauri/src/prd.rs (NEW FILE)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prd {
    pub schema_version: String,
    pub project: ProjectMetadata,
    pub tasks: Vec<Task>,
    #[serde(default)]
    pub completion_marker: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: TaskStatus,
    #[serde(default = "default_priority")]
    pub priority: Priority,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_by: Option<String>,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Done,
    Blocked,
    Skipped,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

fn default_priority() -> Priority {
    Priority::Medium
}

impl Prd {
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read prd.yaml: {}", e))?;
        serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse prd.yaml: {}", e))
    }

    pub fn to_file(&self, path: impl AsRef<std::path::Path>) -> Result<(), String> {
        let content = serde_yaml::to_string(self)
            .map_err(|e| format!("Failed to serialize PRD: {}", e))?;
        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write prd.yaml: {}", e))
    }

    pub fn is_complete(&self) -> bool {
        if self.completion_marker {
            return true;
        }

        let all_done_or_skipped = self.tasks.iter().all(|t| {
            matches!(t.status, TaskStatus::Done | TaskStatus::Skipped)
        });

        let has_done = self.tasks.iter().any(|t| t.status == TaskStatus::Done);

        all_done_or_skipped && has_done
    }

    pub fn pending_tasks(&self) -> Vec<&Task> {
        self.tasks.iter()
            .filter(|t| t.status == TaskStatus::Pending)
            .collect()
    }

    pub fn update_task_status(&mut self, task_id: &str, status: TaskStatus) -> Result<(), String> {
        let task = self.tasks.iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| format!("Task not found: {}", task_id))?;

        task.status = status;
        task.updated = Some(chrono::Utc::now().format("%Y-%m-%d").to_string());

        if status == TaskStatus::Done {
            task.completed = Some(chrono::Utc::now().to_rfc3339());
        }

        Ok(())
    }
}
```

### 6.2 Cargo Dependencies

```toml
# src-tauri/Cargo.toml
[dependencies]
serde_yaml = "0.9"
chrono = { version = "0.4", features = ["serde"] }
```

## 7. Validation Rules

Ralph MUST validate PRD on load:

| Rule ID | Check | Error Message |
|---------|-------|---------------|
| VAL-01 | `schema_version` present | "Missing schema_version in prd.yaml" |
| VAL-02 | `schema_version` is "1.0" | "Unsupported schema_version: {version}" |
| VAL-03 | `project.title` present | "Missing project.title in prd.yaml" |
| VAL-04 | `tasks` is array with ≥1 item | "prd.yaml must have at least one task" |
| VAL-05 | Each task has unique `id` | "Duplicate task ID: {id}" |
| VAL-06 | Each task has `id`, `title`, `status` | "Task missing required field: {field}" |
| VAL-07 | `status` is valid enum value | "Invalid status '{status}' for task {id}" |
| VAL-08 | `depends_on` references existing IDs | "Task {id} depends on unknown task {dep}" |
| VAL-09 | No circular dependencies | "Circular dependency detected: {path}" |

Implementation location: `src-tauri/src/prd.rs` with `impl Prd::validate()`

## 8. Prompt Builder Integration

Ralph injects PRD content into Claude prompt:

```rust
// src-tauri/src/prompt_builder.rs

pub fn build_prompt(prd_path: &Path) -> Result<String, String> {
    let prd = Prd::from_file(prd_path)?;

    let mut prompt = String::new();
    prompt.push_str(&format!("# Project: {}\n\n", prd.project.title));

    if let Some(desc) = &prd.project.description {
        prompt.push_str(&format!("{}\n\n", desc));
    }

    prompt.push_str("## Tasks\n\n");

    for task in &prd.tasks {
        let status_icon = match task.status {
            TaskStatus::Done => "✓",
            TaskStatus::InProgress => "▶",
            TaskStatus::Blocked => "⊗",
            TaskStatus::Skipped => "⊘",
            TaskStatus::Pending => "○",
        };

        prompt.push_str(&format!(
            "{} [{}] {}\n",
            status_icon,
            task.id,
            task.title
        ));

        if let Some(desc) = &task.description {
            prompt.push_str(&format!("   {}\n", desc));
        }

        if !task.tags.is_empty() {
            prompt.push_str(&format!("   Tags: {}\n", task.tags.join(", ")));
        }

        if let Some(blocked) = &task.blocked_by {
            prompt.push_str(&format!("   Blocked: {}\n", blocked));
        }

        prompt.push_str("\n");
    }

    Ok(prompt)
}
```

## 9. UI Display

In `src/components/LoopControls.tsx`, display PRD status:

```typescript
interface PrdStatus {
  totalTasks: number;
  done: number;
  inProgress: number;
  pending: number;
  blocked: number;
  skipped: number;
}

// New IPC command
invoke<PrdStatus>("get_prd_status")
```

Example UI:
```
Project: My Project
─────────────────────────
Tasks: 15 total
  ✓ Done:        8
  ▶ In Progress: 2
  ○ Pending:     4
  ⊗ Blocked:     1
  ⊘ Skipped:     0
─────────────────────────
Progress: 53%
```

## 10. Migration from prd.md

### 10.1 Detection

Ralph SHOULD detect deprecated format:

```rust
// In validate_project_path()
let prd_yaml = path.join(".ralph/prd.yaml");
let prd_md = path.join(".ralph/prd.md");

if !prd_yaml.exists() && prd_md.exists() {
    return Err(
        "Found prd.md (deprecated). Please migrate to prd.yaml. \
         See: https://docs.ralph.dev/migration/prd-yaml"
            .to_string()
    );
}
```

### 10.2 Manual Migration

User must manually convert:

```markdown
# My Project PRD

## Tasks
- [ ] Implement user authentication
- [x] Add database schema
- [ ] Set up CI/CD pipeline
```

↓

```yaml
schema_version: "1.0"
project:
  title: "My Project"

tasks:
  - id: "task-001"
    title: "Implement user authentication"
    status: "pending"

  - id: "task-002"
    title: "Add database schema"
    status: "done"

  - id: "task-003"
    title: "Set up CI/CD pipeline"
    status: "pending"
```

### 10.3 Automated Migration Tool (Future)

```bash
ralph migrate prd /path/to/project
```

Reads `prd.md`, generates `prd.yaml`, prompts for confirmation.

## 11. Example PRD Files

### 11.1 Minimal Valid PRD

```yaml
schema_version: "1.0"
project:
  title: "Quick Script"

tasks:
  - id: "task-001"
    title: "Write the script"
    status: "pending"
```

### 11.2 Complex PRD with Dependencies

```yaml
schema_version: "1.0"
project:
  title: "E-commerce Platform"
  description: "Full-stack shopping site with payment integration"
  created: "2026-02-01"

tasks:
  - id: "backend-001"
    title: "Set up database schema"
    status: "done"
    priority: "critical"
    tags: ["backend", "database"]
    created: "2026-02-01"
    completed: "2026-02-01T14:30:00Z"

  - id: "backend-002"
    title: "Implement REST API"
    status: "done"
    priority: "high"
    tags: ["backend", "api"]
    depends_on: ["backend-001"]
    completed: "2026-02-02T09:15:00Z"

  - id: "backend-003"
    title: "Add Stripe payment processing"
    status: "in_progress"
    priority: "critical"
    tags: ["backend", "payments"]
    depends_on: ["backend-002"]
    acceptance_criteria:
      - "Stripe SDK integrated and configured"
      - "Payment intent creation endpoint works"
      - "Webhook handlers for payment events implemented"
      - "Error handling for failed payments"

  - id: "frontend-001"
    title: "Build product catalog UI"
    status: "done"
    priority: "high"
    tags: ["frontend", "ui"]
    completed: "2026-02-03T11:00:00Z"

  - id: "frontend-002"
    title: "Implement shopping cart"
    status: "pending"
    priority: "high"
    tags: ["frontend", "ui"]
    depends_on: ["frontend-001", "backend-002"]

  - id: "infra-001"
    title: "Set up CI/CD pipeline"
    status: "blocked"
    priority: "medium"
    tags: ["infrastructure"]
    blocked_by: "Waiting for AWS credentials from DevOps"

  - id: "docs-001"
    title: "Write API documentation"
    status: "skipped"
    priority: "low"
    tags: ["documentation"]
```

## 12. Implementation Checklist

### 12.1 Backend (Rust)

- [ ] Add `serde_yaml` and `chrono` dependencies to `Cargo.toml`
- [ ] Create `src-tauri/src/prd.rs` with structs and methods
- [ ] Update `validate_project_path()` to check for `prd.yaml`
- [ ] Add deprecation warning for `prd.md`
- [ ] Update `prompt_builder.rs` to read and format `prd.yaml`
- [ ] Add IPC command `get_prd_status()` for UI display
- [ ] Add IPC command `update_task_status(task_id, status)` (future)
- [ ] Add validation method `Prd::validate()`
- [ ] Update stagnation detection to use `Prd::is_complete()`

### 12.2 Frontend (React)

- [ ] Update validation error messages to reference `prd.yaml`
- [ ] Add PRD status display in `LoopControls`
- [ ] Add progress bar based on task completion percentage
- [ ] (Future) Add task list viewer component

### 12.3 Documentation

- [ ] Update `CLAUDE.md` Target Project Structure section
- [ ] Update `SPEC-030` to reference `prd.yaml` not `prd.md`
- [ ] Create migration guide: `docs/migration/prd-yaml.md`
- [ ] Update fixture: `fixtures/single-task/.ralph/prd.md` → `prd.yaml`

### 12.4 Testing

- [ ] Unit test: Parse valid `prd.yaml`
- [ ] Unit test: Reject invalid schema versions
- [ ] Unit test: Validate task dependencies
- [ ] Unit test: Detect circular dependencies
- [ ] Unit test: `is_complete()` logic
- [ ] Unit test: Task status updates
- [ ] E2E test: Ralph task execution with `prd.yaml`
- [ ] E2E test: Validation rejects `prd.md` (deprecated)

## 13. Backwards Compatibility

**This is a breaking change.**

Projects using `prd.md` MUST migrate to `prd.yaml`. Ralph will:

1. Detect `prd.md` during validation
2. Return clear error message with migration instructions
3. NOT attempt to parse markdown (too fragile)

Users have two options:
1. Manual migration (copy/paste into YAML format)
2. Wait for automated migration tool (future)

## 14. Future Enhancements

### 14.1 Schema Evolution

Future schema versions might add:

- Subtasks/nested tasks
- Assignees
- Time estimates
- Links to issues/PRs
- Custom fields

All additions MUST maintain backwards compatibility within major version.

### 14.2 Task Management UI

Future Ralph UI might include:

- Interactive task viewer/editor
- Drag-and-drop task reordering
- Dependency graph visualization
- Task filtering by status/priority/tags

### 14.3 Alternative Formats

Could support TOML as alternative:

```toml
schema_version = "1.0"

[project]
title = "My Project"

[[tasks]]
id = "task-001"
title = "Do something"
status = "pending"
```

But YAML is preferred for:
- Better multi-line string support
- More familiar to developers
- Better tooling ecosystem

## 15. Traceability

| Requirement | Implementation File | Test File |
|-------------|---------------------|-----------|
| FILE-01 | `src-tauri/src/commands.rs::validate_project_path()` | TBD |
| VAL-01..09 | `src-tauri/src/prd.rs::Prd::validate()` | TBD |
| Completion | `src-tauri/src/prd.rs::Prd::is_complete()` | TBD |
| Status updates | `src-tauri/src/prd.rs::Prd::update_task_status()` | TBD |

## 16. References

- [SPEC-030: Ralph Project Standards](./030_RALPH_PROJECT_STANDARDS.md)
- [YAML 1.2 Specification](https://yaml.org/spec/1.2/spec.html)
- [serde_yaml Documentation](https://docs.rs/serde_yaml/)
