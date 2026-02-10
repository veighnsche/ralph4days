# Prompt Assembly Nexus — How Tasks Drive Everything

**Created:** 2026-02-10
**Status:** Implemented (elaborates on 015_TASK_MODEL_AS_PROMPT_ASSEMBLY_NEXUS.md)

## Why This Matters

Every Claude instance Ralph launches is **amnesiac** — zero memory of previous sessions. The only thing connecting it to the project is the prompt Ralph assembles. If the prompt is wrong, vague, or bloated, the session is wasted (burned tokens, no progress, stagnation counter ticks up).

The task model is the nexus because **a single task record references everything needed to build a perfect prompt**: who the agent should be (discipline), what domain it's working in (feature), and exactly what to build (the task itself). Ralph walks these references and assembles a self-contained context packet. No guessing. No "pick ONE task from this wall of text."

## The Question Table

This is the core design insight. Every field in the data model exists to answer a specific question that an amnesiac Claude instance would otherwise have to figure out on its own (burning tokens and risking hallucination):

| Question | Source | Prompt Section | Why It Matters |
|----------|--------|----------------|----------------|
| **WHO am I?** | `Discipline.system_prompt` | `discipline_persona` | Sets the persona. A "frontend developer" writes different code than a "security engineer" given the same task. Without this, Claude defaults to generic assistant behavior. |
| **WHAT coding standards?** | `Discipline.conventions` | `discipline_persona` | Prevents style violations that fail pre-commit hooks. A frontend discipline knows about biome/oxlint rules; a backend discipline knows about clippy lints. Without this, Claude writes code that gets rejected. |
| **WHAT tools do I have?** | `Discipline.mcp_servers` | MCP config (JSON) | Controls which MCP servers are available. A frontend discipline gets shadcn-ui and tailwindcss MCPs; a backend discipline might get none. Without this, Claude either lacks tools it needs or wastes turns trying tools it doesn't have. |
| **WHAT domain am I in?** | `Feature.description` | `feature_context` | Scopes the work. "Authentication" vs "Dashboard" vs "CLI" — each implies different patterns, files, and concerns. Without this, Claude doesn't know what part of the codebase it's operating in. |
| **WHAT docs are relevant?** | `Feature.knowledge_paths` | `feature_files` | Injects specs, design docs, and architecture notes directly into the prompt. Without this, Claude would need to discover and read these files itself (burning turns) or worse, make assumptions. |
| **WHAT code matters?** | `Feature.context_files` + `Task.context_files` | `feature_files` + `task_files` | Pre-reads source files so Claude sees existing patterns immediately. Feature-level files are always relevant (e.g., `src/hooks/useAuth.ts` for any auth task). Task-level files are specific (e.g., `RegisterForm.tsx` as a pattern to follow for building `LoginForm.tsx`). |
| **WHAT exactly do I build?** | `Task.title` + `Task.description` | `task_details` | The actual work order. Unambiguous, scoped, actionable. Without this, Claude is just exploring. |
| **WHAT counts as done?** | `Task.acceptance_criteria` | `task_details` | Concrete success conditions. Claude can self-verify against these. Without this, "done" is subjective and Claude may stop too early or gold-plate. |
| **WHAT should I produce?** | `Task.output_artifacts` | `task_details` | Expected output files. Tells Claude exactly what to create or modify. Without this, Claude might put code in the wrong place. |
| **HOW should I approach it?** | `Task.hints` | `task_details` | Senior-dev-to-junior-dev guidance. "Use the existing Form component from shadcn/ui" or "Follow the pattern in RegisterForm.tsx." Without this, Claude reinvents patterns that already exist in the codebase. |
| **WHAT came before?** | `Task.depends_on` → completed task summaries | `dependency_context` | Shows what prerequisite tasks produced. If task #1 depends on task #3 (auth API), Claude sees task #3's title and status so it knows the API exists. Without this, Claude might try to build both. |
| **WHAT went wrong last time?** | `Task.comments` | `previous_attempts` | Structured retry history. If a previous session failed because "auth middleware wasn't returning user object in expected format," the next session knows exactly where to look. Without this, the same failure repeats (stagnation). |
| **WHO created this task?** | `Task.provenance` | `task_details` | Agent-created tasks (from braindump) tend to be well-structured. Human-created tasks (from UI forms) may need more interpretation. System tasks (verification, hooks) have specific constraints. The prompt builder can adjust tone accordingly. |

## How It's Implemented: Recipes and Sections

The prompt builder (`crates/prompt-builder/`) uses a **recipe system**. Each prompt type (braindump, task execution, opus review, etc.) is a recipe — an ordered list of sections.

### The Task Execution Recipe

```
project_context        → Project title, description, CLAUDE.RALPH.md
discipline_persona     → WHO am I + coding standards
feature_context        → WHAT domain
feature_files          → knowledge_paths + feature-level context_files (injected)
feature_state          → Other tasks in this feature (for awareness)
state_files            → progress.txt + learnings.txt
previous_attempts      → Task comments (retry history)
dependency_context     → Completed prerequisite task summaries
task_details           → Title, description, acceptance criteria, output artifacts, hints
task_files             → Task-level context_files (injected)
task_exec_instructions → "Complete this task. When done: [update status, commit, etc.]"
```

**Order matters for recency bias.** LLMs pay more attention to what they read last. The recipe puts the most actionable content (task details, instructions) at the end, and background context (project metadata, discipline persona) at the beginning.

### Section → Question Mapping

Each section is a pure function: `fn render(ctx: &PromptContext) -> Option<String>`. It reads from `PromptContext` (pre-queried database state + pre-read file contents) and returns markdown text or `None` if there's nothing to render. No I/O inside sections.

The `PromptContext` struct is the caller's job to populate — it queries the SQLite database and reads all referenced files before calling `build()`. This keeps the prompt builder pure and testable.

### MCP Tools Per Recipe

Each recipe also declares which MCP tools to include. Task execution gets:
- `SetTaskStatus` — so Claude can mark the task done
- `AppendLearning` — so Claude can record what it learned (feeds back into future prompts)
- `AddContextFile` — so Claude can register files it discovers as relevant (improves future sessions)

These are generated as bash MCP server scripts at runtime, giving the amnesiac Claude instance write access to the project database through a clean tool interface.

## The Data Flow

```
1. Ralph picks next pending task (id=7)
2. Queries SQLite: task #7, its feature, its discipline
3. Reads all referenced files:
   - Feature.knowledge_paths → .specs/AUTH_SPEC.md, docs/auth-flow.md
   - Feature.context_files → src/lib/auth.ts, src/hooks/useAuth.ts
   - Task.context_files → src/components/auth/RegisterForm.tsx
   - progress.txt, learnings.txt, CLAUDE.RALPH.md
4. Builds PromptContext (all data, all file contents, no I/O pending)
5. Calls prompt_builder::build(TaskExecution, &ctx)
6. Recipe iterates sections, each renders markdown from context
7. Output: assembled prompt text + MCP server scripts + CLI flags
8. Ralph launches: claude --max-turns N --mcp-config ... < prompt.txt
```

## Why Not Just Dump Everything?

The original approach (doc 015's "before" column) dumped all 4 raw database files into a single prompt. Problems:

1. **Token waste** — Haiku reads descriptions of tasks in completely unrelated features and disciplines. Every irrelevant token is money burned.
2. **Attention dilution** — "Pick ONE task" forces Claude to parse, evaluate, and select from potentially hundreds of tasks. It often picks wrong or gets confused by dependencies.
3. **No persona** — Without discipline context, Claude writes generic code that doesn't match project conventions. Fails linting, wastes a retry.
4. **No knowledge injection** — Without feature files pre-read, Claude spends turns discovering what files exist and reading them. Turns are expensive.
5. **No retry intelligence** — Without comments from previous attempts, the same failure mode repeats identically.

The surgical approach means a task execution prompt contains **only what that specific task needs**. A frontend auth task gets React conventions, shadcn MCP, auth specs, and the specific component to follow as a pattern. Nothing else.

## Feature-Level Fields Beyond the Original Proposal

Since doc 015, Feature has grown additional fields that feed into prompts:

| Field | Purpose | Used In |
|-------|---------|---------|
| `architecture` | How this feature is structured (layers, patterns) | `feature_context` section |
| `boundaries` | What this feature does NOT touch | `feature_context` section |
| `learnings` | Append-only lessons from completed tasks | `feature_state` section |
| `dependencies` | Other features this one depends on | `feature_context` section |

These evolved from the RAG work (docs 017-019) and the feature entity redesign (doc 018). The pattern is consistent: every field exists to answer a question that would otherwise require Claude to discover the answer itself.

## The Virtuous Cycle

The MCP tools create a feedback loop:

1. Claude executes task #7
2. During execution, Claude calls `AppendLearning("The auth middleware returns { user, token } not just { token }")`
3. This learning is stored in the feature record
4. Next time any auth task runs, `feature_state` includes this learning
5. Claude doesn't repeat the same mistake

Similarly with `AddContextFile` — Claude discovers that `src/middleware/auth.ts` is relevant, registers it, and future auth tasks automatically include it in their prompt.

The task model isn't just a work order. It's a **knowledge accumulation system** where each session makes future sessions smarter, without any session having memory of the past.
