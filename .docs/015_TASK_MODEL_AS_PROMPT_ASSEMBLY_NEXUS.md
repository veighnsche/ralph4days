# Task Model as Prompt Assembly Nexus

**Created:** 2026-02-06
**Status:** Design Proposal

## The Problem

Currently, Discipline and Feature are **display metadata only** — names, icons, colors, acronyms. The prompt builder dumps all 4 raw YAML files into a single prompt and tells Haiku "pick ONE task." Haiku gets a wall of undifferentiated text with no surgical guidance.

The task model needs to become a **prompt assembly nexus**: a task references a Discipline (HOW) and a Feature (ABOUT WHAT), and Ralph deterministically assembles a focused, self-contained execution environment from all three.

## Core Mental Model

```
TASK ──references──→ DISCIPLINE = HOW to do it (persona + tools + conventions)
  │
  └──references──→ FEATURE = WHAT DOMAIN (knowledge base + relevant files)
```

Each Ralph iteration launches a **fresh Claude instance with no memory**. The task model must therefore produce a **self-contained context packet** that answers:

| Question | Source |
|----------|--------|
| WHO am I? | Discipline.system_prompt |
| WHAT coding standards? | Discipline.conventions |
| WHAT tools do I have? | Discipline.mcp_servers |
| WHAT domain am I in? | Feature.description |
| WHAT docs are relevant? | Feature.knowledge_paths |
| WHAT code matters? | Feature.context_files + Task.context_files |
| WHAT exactly do I build? | Task.title + Task.description |
| WHAT counts as done? | Task.acceptance_criteria |
| WHAT should I produce? | Task.output_artifacts |
| HOW should I approach it? | Task.hints |
| WHAT came before? | Task.depends_on (completed tasks) |
| WHAT went wrong last time? | Task.comments |
| WHO created this task? | Task.provenance |

## Proposed Schema

### Discipline (the HOW — persona, tools, conventions)

```yaml
# .ralph/db/disciplines.yaml
disciplines:
  - name: frontend
    display_name: Frontend
    acronym: FRNT
    icon: Monitor
    color: "#3b82f6"
    # --- execution context ---
    system_prompt: |
      You are a frontend developer. You work with React 19, TypeScript,
      and Tailwind v4. Use components from src/components/ui/.
    skills:
      - react
      - typescript
      - tailwindcss
      - accessibility
    conventions: |
      - Functional components with hooks only
      - Desktop-density: h-8 default, h-6 sm
      - shadow-sm max, 8px spacing grid
      - Always check src/components/ui/ before creating custom markup
    mcp_servers:
      - name: shadcn-ui
        command: npx
        args: ["-y", "@anthropic-ai/shadcn-ui-mcp"]
      - name: tailwindcss
        command: npx
        args: ["-y", "@anthropic-ai/tailwindcss-mcp"]
```

**New fields on Discipline:**

| Field | Type | Required | Purpose |
|-------|------|----------|---------|
| `system_prompt` | string | no | Persona and general approach for this discipline |
| `skills` | Vec<string> | no | Capabilities (for prompt context, not enforcement) |
| `conventions` | string | no | Coding standards and patterns to follow |
| `mcp_servers` | Vec<McpServer> | no | MCP servers to include when executing tasks of this discipline |

**McpServer sub-struct:**

| Field | Type | Required | Purpose |
|-------|------|----------|---------|
| `name` | string | yes | MCP server identifier |
| `command` | string | yes | Command to run (e.g., "npx", "bash", "node") |
| `args` | Vec<string> | no | Command arguments |
| `env` | Map<string,string> | no | Environment variables |

### Feature (the DOMAIN — knowledge, relevant files)

```yaml
# .ralph/db/features.yaml
features:
  - name: authentication
    display_name: Authentication
    acronym: AUTH
    description: JWT-based user authentication with refresh tokens
    # --- knowledge context ---
    knowledge_paths:
      - docs/auth-flow.md
      - .specs/AUTH_SPEC.md
    context_files:
      - src/lib/auth.ts
      - src/hooks/useAuth.ts
      - src-tauri/src/auth/mod.rs
```

**New fields on Feature:**

| Field | Type | Required | Purpose |
|-------|------|----------|---------|
| `knowledge_paths` | Vec<string> | no | Docs/specs to read and inject into prompt (relative to project root) |
| `context_files` | Vec<string> | no | Source files always relevant to this feature (relative to project root) |

**"RAG" here means:** Ralph reads these files with `fs::read_to_string` and injects their contents into the prompt. No embeddings, no vector DB. Just deterministic file inclusion. Ralph is NOT AI.

### Task (the WHAT — work, deliverables, hints)

```yaml
# .ralph/db/tasks.yaml
tasks:
  - id: 1
    feature: authentication
    discipline: frontend
    title: Build login form component
    status: pending
    priority: high
    description: |
      Create a login form with email/password fields, client-side validation,
      and integration with the auth API from task #3.
    acceptance_criteria:
      - Email field with format validation
      - Password field with min 8 chars
      - Error states for invalid credentials
      - Loading state during API call
      - Redirect to dashboard on success
    depends_on: [3]
    tags: [ui, forms]
    # --- execution context ---
    context_files:
      - src/components/auth/RegisterForm.tsx
    output_artifacts:
      - src/components/auth/LoginForm.tsx
      - src/components/auth/LoginForm.test.tsx
    hints: |
      Use the existing Form component from shadcn/ui.
      Follow the pattern established in RegisterForm.tsx.
    estimated_turns: 10
    # --- provenance & history ---
    provenance: agent
    comments:
      - author: agent
        agent_task_id: 1
        body: "2026-02-05: Failed — auth middleware wasn't returning user object in expected format. Check src/middleware/auth.ts response shape."
        created: '2026-02-05T18:30:00Z'
```

**New fields on Task:**

| Field | Type | Required | Purpose |
|-------|------|----------|---------|
| `context_files` | Vec<string> | no | Task-specific files to include in prompt (on top of feature's context_files) |
| `output_artifacts` | Vec<string> | no | Expected files to create or modify |
| `hints` | string | no | Implementation guidance — things a senior dev would tell a junior |
| `estimated_turns` | u32 | no | Override default --max-turns for this task |
| `provenance` | enum | no | Who created this task: `agent`, `human`, or `system` |
| `comments` | Vec<TaskComment> | no | Structured comments with author (human/agent), body, and timestamp |

## Prompt Assembly

When Ralph executes task #1, it deterministically builds:

### 1. The Prompt

```markdown
## You Are
You are a frontend developer. You work with React 19, TypeScript,
and Tailwind v4. Use components from src/components/ui/.

## Your Skills
react, typescript, tailwindcss, accessibility

## Conventions
- Functional components with hooks only
- Desktop-density: h-8 default, h-6 sm
- shadow-sm max, 8px spacing grid
- Always check src/components/ui/ before creating custom markup

## Feature: Authentication
JWT-based user authentication with refresh tokens

## Reference Documents
### docs/auth-flow.md
[file contents injected]

### .specs/AUTH_SPEC.md
[file contents injected]

## Relevant Source Files
### src/lib/auth.ts
[file contents injected — from feature.context_files]

### src/hooks/useAuth.ts
[file contents injected — from feature.context_files]

### src/components/auth/RegisterForm.tsx
[file contents injected — from task.context_files]

## Your Task
**Build login form component**

Create a login form with email/password fields, client-side validation,
and integration with the auth API from task #3.

## Acceptance Criteria
- [ ] Email field with format validation
- [ ] Password field with min 8 chars
- [ ] Error states for invalid credentials
- [ ] Loading state during API call
- [ ] Redirect to dashboard on success

## Expected Output Files
- src/components/auth/LoginForm.tsx
- src/components/auth/LoginForm.test.tsx

## Implementation Hints
Use the existing Form component from shadcn/ui.
Follow the pattern established in RegisterForm.tsx.

## Previous Attempts (IMPORTANT)
- 2026-02-05: Failed — auth middleware wasn't returning user object
  in expected format. Check src/middleware/auth.ts response shape.

## Completed Prerequisites
Task #3 (done): "Implement auth API endpoints"

## Instructions
Complete this task. When done:
1. Update task #1 status to 'done' in .ralph/db/tasks.yaml
2. Commit your changes
3. Append a brief summary to .ralph/progress.txt
```

### 2. The MCP Config

Assembled from discipline.mcp_servers + ralph-db (always included):

```json
{
  "mcpServers": {
    "shadcn-ui": {
      "command": "npx",
      "args": ["-y", "@anthropic-ai/shadcn-ui-mcp"]
    },
    "tailwindcss": {
      "command": "npx",
      "args": ["-y", "@anthropic-ai/tailwindcss-mcp"]
    },
    "ralph-db": {
      "command": "bash",
      "args": ["/tmp/ralph-mcp/ralph-db.sh"]
    }
  }
}
```

### 3. The CLI Invocation

```bash
claude \
  --max-turns 10 \
  --output-format stream-json \
  --mcp-config /tmp/ralph-mcp-config.json \
  --project /path/to/project \
  < /tmp/ralph-prompt.txt
```

## Comparison: Before vs After

| Aspect | Current (dump everything) | Proposed (surgical assembly) |
|--------|--------------------------|------------------------------|
| Prompt content | All 4 YAML files raw | Only task-relevant context |
| Discipline role | Display metadata | Persona + tools + standards |
| Feature role | Display metadata | Knowledge base + context files |
| MCP servers | Global/static | Per-discipline, per-task |
| Task guidance | "pick ONE task" | Focused on exactly one task |
| Context size | Entire project state | Only what this task needs |
| Haiku effectiveness | Must parse everything | Gets exactly what it needs |

## Backward Compatibility

All new fields use `#[serde(default, skip_serializing_if = ...)]`. Existing YAML files work unchanged:

- Discipline without `system_prompt`/`skills`/etc → prompt section omitted, no discipline-specific MCP servers
- Feature without `knowledge_paths`/`context_files` → no extra files injected
- Task without `hints`/`output_artifacts`/etc → simpler prompt, default max-turns

The system degrades gracefully to current behavior when new fields aren't populated.

## Rust Struct Changes

### Discipline

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discipline {
    pub name: String,
    pub display_name: String,
    pub icon: String,
    pub color: String,
    #[serde(default)]
    pub acronym: String,
    // --- execution context ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conventions: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_servers: Vec<McpServerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
}
```

### Feature

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub name: String,
    pub display_name: String,
    #[serde(default)]
    pub acronym: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    // --- knowledge context ---
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub knowledge_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_files: Vec<String>,
}
```

### Task

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    // ... existing fields unchanged ...
    // --- execution context ---
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_files: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub output_artifacts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hints: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_turns: Option<u32>,
    // --- provenance & history ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<TaskProvenance>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub comments: Vec<TaskComment>,
}

/// Who created this task and how
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskProvenance {
    /// Created by an AI agent (structured, from braindump/loop)
    Agent,
    /// Created by a human via the UI form
    Human,
    /// Generated programmatically by Ralph (verification tasks, hooks, etc.)
    System,
}
```

## Resolved Questions

0. **Verification** — NOT a model field. Verification is either handled by Claude Code hooks (built-in) or by Ralph generating verification tasks in the task list with `provenance: system`. This keeps the model clean — a verification step is just another task, not a special field.

1. **File size limits** — Not now. Collect and gather everything first. Prompt refinement (truncation, prioritization, token budgets) is a separate later concern. The model should capture what's *available*, not what fits.

2. **Glob patterns** — Undecided. Park it. Start with explicit paths. Can revisit if explicit becomes painful.

3. **Dependency summaries** — Undecided. Collect first, filter later. Both task descriptions and progress.txt entries are available; the prompt builder decides what to use when we build it.

4. ~~**MCP server deduplication**~~ — N/A. One task = one discipline. Moot.

5. **Feature knowledge delivery** — Feature knowledge will eventually be exposed as **MCP resources** that Claude can pull on-demand, not injected into the prompt. This means `knowledge_paths` and `context_files` on Feature are **transitional fields** — they capture *what* knowledge exists, but the *delivery mechanism* will evolve from prompt injection to MCP resources.

## Design Philosophy

**Phase 1 (now): Collect.** Get the model right. Capture everything a task needs — the persona, the tools, the domain knowledge, the hints, the deliverables. All fields optional, all backward compatible.

**Phase 2 (later): Assemble.** Build the surgical prompt builder that assembles from task + discipline + feature. Start with simple file injection.

**Phase 3 (later): Refine.** Replace brute-force file injection with MCP resources. Add token budgets. Optimize what goes into the prompt vs what Claude pulls on demand.
