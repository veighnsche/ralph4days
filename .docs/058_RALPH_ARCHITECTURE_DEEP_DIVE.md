# Ralph4days: System Architecture Analysis

**A Tauri-Based Autonomous Multi-Agent Task Orchestration System**

## Executive Summary

Ralph4days is a desktop application implementing an **orchestrator pattern** for autonomous software development task execution. It coordinates Claude AI instances (via CLI) through a structured pipeline architecture, using SQLite for state persistence, dynamic prompt generation, and bidirectional signaling through Model Context Protocol (MCP) servers.

**Key Innovation**: Ralph is deterministic orchestration code—not AI. It launches Claude CLI instances with carefully constructed prompts and toolsets, monitors their execution, and sequences tasks based on structured signals.

---

## 1. System Architecture Overview

### 1.1 Technology Stack

**Frontend (UI Layer)**
- React 19 with React Compiler (automatic memoization)
- TypeScript 5.8
- Zustand for state management (chosen over Redux for simplicity)
- shadcn/ui component library (50+ components)
- Tailwind CSS v4 for styling
- Vite for bundling

**Backend (Orchestration Layer)**
- Rust (edition 2021, min version 1.75)
- Tauri 2.5 for desktop app framework
- Tokio for async runtime
- Rusqlite with WAL mode for database
- portable-pty for subprocess TTY management

**Inter-Process Communication**
- Tauri IPC (async function calls from frontend to backend)
- Event-based streaming (PTY output → frontend)
- SQLite as shared state store

### 1.2 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      FRONTEND (React)                       │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │ Workspace UI │  │ Task Manager │  │  Terminal View  │  │
│  │  (Zustand)   │  │  (Forms)     │  │    (xterm.js)   │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬────────┘  │
│         │                 │                    │            │
│         └─────────────────┼────────────────────┘            │
│                           │ Tauri IPC                       │
└───────────────────────────┼─────────────────────────────────┘
                            │
┌───────────────────────────┼─────────────────────────────────┐
│                      BACKEND (Rust/Tauri)                   │
│                           │                                 │
│  ┌────────────────────────▼─────────────────────┐          │
│  │         Command Layer (Tauri Handlers)       │          │
│  │  project • tasks • features • prompts        │          │
│  └───┬─────────────────┬────────────────┬───────┘          │
│      │                 │                │                   │
│  ┌───▼──────┐    ┌────▼─────┐    ┌────▼────────┐          │
│  │ PTY Mgr  │    │ SQLite   │    │   Prompt    │          │
│  │ (Claude  │◄───┤   DB     │◄───┤  Builder    │          │
│  │  CLI)    │    │ (WAL)    │    │ (Pure Fns)  │          │
│  └────┬─────┘    └──────────┘    └─────────────┘          │
│       │                                                     │
│       │ spawn process + MCP config                         │
└───────┼─────────────────────────────────────────────────────┘
        │
┌───────▼──────────────────────────────────────────────────┐
│           Claude CLI Subprocess (External)               │
│  ┌──────────────────┐      ┌────────────────────────┐   │
│  │ Task Execution   │◄────►│  Dynamic MCP Server    │   │
│  │ (Haiku/Opus)     │      │  (Bash-generated)      │   │
│  └──────────────────┘      └────────────────────────┘   │
│                                                          │
│  MCP Tools: done(), partial(), stuck(), ask(), flag(),  │
│             learned(), suggest(), blocked()              │
└──────────────────────────────────────────────────────────┘
```

---

## 2. Core Architectural Patterns

### 2.1 Orchestrator Pattern

Ralph implements a **coordinator/orchestrator** rather than an agent:

**Key Principle**: Ralph IS NOT AI. It is deterministic Rust code that:
1. Builds structured prompts from database state
2. Launches Claude CLI subprocesses with specific configurations
3. Monitors execution through PTY streaming
4. Parses structured signals from MCP tool calls
5. Updates database based on outcomes
6. Selects and launches next task

**Rationale**: By keeping orchestration logic deterministic and simple, complexity is pushed to the AI layer where it belongs. Ralph is a thin wrapper around Claude Code CLI.

### 2.2 Sequential Pipeline Architecture

**Common Misconception**: Ralph is not a "loop."

**Actual Design**: Sequential task execution pipeline:
```
Task Queue → Select Next Task → Build Prompt → Launch Claude CLI →
Wait for Exit → Parse Signals → Update State → Task Queue
```

A "loop" only emerges when tasks create more tasks—it's an emergent property of the task graph, not a built-in mechanism.

**Termination Conditions**:
- All tasks completed
- All remaining tasks blocked/need input
- Stagnation detected (3 sessions with no file/database changes)
- User stop signal

### 2.3 Modular Crate Architecture

The backend is organized as a Cargo workspace with specialized crates:

| Crate | Responsibility | Key Design |
|-------|---------------|------------|
| `sqlite-db` | Data layer | Pure CRUD operations, no business logic |
| `prompt-builder` | Prompt generation | **Pure functions**—no I/O, fully testable |
| `ralph-errors` | Error handling | Centralized error types + extension traits |
| `ralph-rag` | RAG/embeddings | Ollama integration, comment similarity |
| `ralph-external` | External services | Ollama, ComfyUI clients |
| `predefined-disciplines` | Discipline configs | JSON definitions + image generation |
| `ralph-macros` | Procedural macros | Code generation utilities |

**Key Architectural Decision**: The `prompt-builder` crate contains **only pure functions**. It takes a `PromptContext` struct and returns strings. All I/O happens in the `src-tauri` binary, which constructs the context. This enables:
- Unit testing without mocks
- Deterministic prompt generation
- Easy reasoning about prompt composition

---

## 3. Data Architecture

### 3.1 SQLite Database Schema

**Choice Rationale**: SQLite chosen over client-server DB for:
- Zero configuration (single file)
- ACID transactions with foreign keys
- WAL mode for concurrent read/write
- Embedded in desktop app (no network)

**Core Tables**:

```sql
metadata        -- Singleton project config (title, description, created)
features        -- Feature definitions (name PK, architecture, learnings JSON)
disciplines     -- Role definitions (name PK, system_prompt, mcp_servers JSON)
tasks           -- Task queue (id, feature FK, discipline FK, status, depends_on JSON)
task_comments   -- Comments/notes on tasks
feature_comments -- Comments on features
recipe_configs  -- Custom prompt recipes
comment_embeddings -- RAG: vector embeddings for semantic search
```

**Design Decisions**:
- JSON columns for arrays (`tags`, `depends_on`) to avoid separate join tables
- Foreign key constraints enforced (`ON DELETE RESTRICT` prevents orphans)
- Indexes on frequently queried columns (feature, discipline, status)
- Versioned migrations using `rusqlite_migration`

### 3.2 Task State Machine

```
         ┌──────────┐
         │ pending  │◄──────────┐
         └────┬─────┘           │
              │ start           │ partial
         ┌────▼─────┐           │
         │ running  ├───────────┘
         └────┬─────┘
              │
        ┌─────┼─────┬─────────┬──────────┐
        │     │     │         │          │
    ┌───▼──┐ │ ┌───▼────┐ ┌──▼──────┐   │
    │ done │ │ │ stuck  │ │ blocked │   │
    └──────┘ │ └────────┘ └─────────┘   │
             │                           │
        ┌────▼──────────┐                │
        │ needs_input   │◄───────────────┘
        └───────────────┘
```

**State Transitions**:
- `pending` → running when task starts
- `running` → `completed` when agent calls `done()`
- `running` → `pending` when agent calls `partial()` (re-queue)
- `running` → `stuck` → `failed` after 3 stuck sessions
- `running` → `needs_input` when `ask(blocking=true)` + `partial()`
- `running` → `blocked` when dependency missing

---

## 4. Prompt Engineering Architecture

### 4.1 Recipe System

Prompts are built using a **recipe-based composition system**:

```rust
pub struct Recipe {
    pub name: &'static str,
    pub sections: Vec<Section>,
    pub mcp_tools: Vec<McpTool>,
}

pub struct Section {
    pub name: &'static str,
    pub build: fn(&PromptContext) -> Option<String>,
}
```

**Seven Prompt Types** (recipes):
1. **Braindump** — Unstructured human rambling → structured tasks
2. **Yap** — Quick conversational context dump
3. **Ramble** — Extended brainstorming session
4. **Discuss** — Dialectical exploration of a topic
5. **TaskExecution** — Execute a specific task from queue
6. **OpusReview** — Opus reviews Haiku's work (quality gate)
7. **Enrichment** — Add pseudocode/acceptance criteria to tasks

**Design Benefits**:
- Sections are **pure functions** (testable)
- Recipes are **data** (composable)
- User can customize recipes via UI (section reordering, overrides)
- Same context struct drives all recipes

### 4.2 Dynamic Prompt Construction

```rust
pub struct PromptContext {
    pub project_dir: PathBuf,
    pub project_title: String,
    pub task: Option<Task>,
    pub feature: Option<Feature>,
    pub discipline: Option<Discipline>,
    pub all_features: Vec<Feature>,
    pub all_tasks: Vec<Task>,
    pub prior_comments: Vec<TaskComment>,
    pub feature_comments: Vec<FeatureComment>,
    pub user_input: Option<String>,
    pub codebase_files: Vec<PathBuf>,
    pub override_instructions: Option<String>,
    // ... more fields
}
```

**Prompt Build Pipeline**:
1. Query database for current project state
2. Read relevant files (learnings.txt, progress.txt, CLAUDE.RALPH.md)
3. Execute `git diff` to capture recent changes
4. Construct `PromptContext` with all data
5. Run recipe's section builders
6. Concatenate non-None sections
7. Return final prompt string

**Key Insight**: By making prompt building pure, the same context can generate prompts for testing, preview UI, and actual execution.

---

## 5. Process Management Architecture

### 5.1 PTY Manager

Ralph uses **portable-pty** to manage Claude CLI subprocesses with full TTY support:

```rust
pub struct PTYManager {
    sessions: Arc<Mutex<HashMap<String, PTYSession>>>,
}
```

**Capabilities**:
- Spawn Claude CLI with custom args (`--model`, `--mcp-config`, `--settings`)
- Stream stdout/stderr to frontend via Tauri events
- Send user input (e.g., interrupt with Ctrl+C)
- Resize terminal (for responsive UI)
- Terminate sessions on demand

**Threading Model**:
- Main thread: Tauri event loop
- Per-session reader thread: Blocks on PTY read, emits events to frontend
- PTY writes: Mutex-protected on main thread

### 5.2 Claude CLI Integration

Ralph launches Claude CLI with specific flags:

```bash
claude \
  --permission-mode bypassPermissions \
  --verbose \
  --no-chrome \
  --model sonnet \
  --mcp-config /path/to/dynamic-mcp.json \
  --settings '{"max_turns": 50, "thinking": "extended"}'
```

**Key Configuration**:
- `--permission-mode bypassPermissions`: Skip prompts (headless execution)
- `--no-chrome`: Disable browser UI
- `--mcp-config`: Path to dynamically generated MCP server config
- `--model`: haiku (default), sonnet, or opus
- `--settings`: JSON with max_turns, thinking mode, etc.

**Dynamic MCP Server Generation**: Ralph generates a bash-based MCP server **per session** that:
- Sets environment variables (`RALPH_TASK_ID`, `RALPH_SESSION_ID`, `RALPH_DB_PATH`)
- Provides 8 MCP tools (done, partial, stuck, ask, flag, learned, suggest, blocked)
- Each tool call is just `INSERT INTO task_signals VALUES (...)`

---

## 6. MCP "Exhaust Pipe" Architecture

### 6.1 Design Philosophy

**Problem**: How does Claude signal back to Ralph?

**Solution**: A bidirectional signaling protocol:
- **Input**: Ralph builds prompts (context + instructions)
- **Output**: Claude calls MCP tools to signal status

**Key Innovation**: The MCP server is a "dumb INSERT pipe." All intelligence lives in Ralph's post-processing.

### 6.2 The 8 Verbs

**Closing Verbs** (exactly one per session, last wins):
1. `done(summary)` — Task complete
2. `partial(summary, remaining)` — Progress made, re-queue
3. `stuck(reason)` — Cannot proceed

**Signal Verbs** (zero or more per session):
4. `ask(question, blocking)` — Request human input
5. `flag(what, severity, category)` — Report discovered problem
6. `learned(text, kind, rationale, scope)` — Record knowledge for future tasks
7. `suggest(what, kind, why)` — Propose new task/refactor
8. `blocked(on, kind)` — Blocked by dependency/external

**Example MCP Tool**:
```json
{
  "name": "done",
  "description": "Signal that the task is fully complete and tested",
  "inputSchema": {
    "type": "object",
    "properties": {
      "summary": {
        "type": "string",
        "description": "What was accomplished. Include key decisions and outcomes."
      }
    },
    "required": ["summary"]
  }
}
```

### 6.3 Post-Processing Pipeline

After Claude CLI exits, Ralph runs:

```
1. Read all task_signals for this session_id
2. Identify closing verb (last done/partial/stuck)
3. If no closing verb → infer stuck("session ended without signal")
4. Process closing verb:
   - done → task.status = "completed"
   - partial → re-queue, inject "remaining" into next prompt
   - stuck → increment stagnation counter
5. Process signal verbs:
   - ask(blocking=true) + partial → task.status = "needs_input"
   - learned → store, add to future prompts (prompt-builder queries)
   - suggest(new_task) → auto-create task with "agent" provenance
   - blocked(upstream_task) → add dependency link
6. Stagnation check: hash DB + files, compare with pre-session
7. Select next task from queue
```

---

## 7. Frontend Architecture

### 7.1 State Management

**Zustand** for global state (chosen over Redux for simplicity):

```typescript
interface WorkspaceStore {
  tabs: WorkspaceTab[]
  activeTabId: string
  openTab: (tab: Omit<WorkspaceTab, 'id'>) => string
  closeTab: (tabId: string) => void
  switchTab: (tabId: string) => void
  // ... more actions
}
```

**Design Pattern**: Browser-style tab management:
- Tabs can be reordered via drag-and-drop
- Max 20 tabs (oldest closeable tab evicted)
- Tab IDs generated from type + entity ID (deterministic)
- Tabs update their own title/icon (not parent-driven)

### 7.2 React 19 + Compiler

**Key Feature**: React Compiler (Babel plugin) provides automatic memoization.

**Implication**: Code uses **no manual optimization**:
- No `useMemo()`
- No `useCallback()`
- No `React.memo()`

The compiler analyzes dataflow and inserts memoization where beneficial. This reduces code complexity and token usage.

### 7.3 Component Architecture

**shadcn/ui Pattern**: Components are **copied into src/components/ui/**, not installed as dependencies. This allows:
- Direct customization without ejecting
- Tree-shaking unused variants
- Adherence to desktop density standards (h-8 default, never h-10)

**UI Design Philosophy**:
- Desktop-first (not mobile-responsive)
- High information density (compact spacing)
- Maximum contrast (APCA-based color system)
- 8px grid, 100-200ms transitions

---

## 8. Novel Design Decisions

### 8.1 Ralph as "Thin Wrapper"

**Specification SPEC-050**: Ralph is the thinnest possible wrapper around Claude Code.

**Rationale**:
- Claude Code already has project navigation, codebase search, file editing, git integration
- Re-implementing these capabilities would waste tokens and introduce bugs
- Ralph's job is **only**:
  1. Build prompts
  2. Launch Claude
  3. Parse signals
  4. Sequence tasks

**Anti-pattern**: Ralph does NOT provide Claude with tools to edit files, run tests, or commit code. Claude uses its **native capabilities** for those. Ralph only provides status signaling.

### 8.2 Sequential Pipeline (Not a Loop)

**Design Choice**: Tasks execute in sequence, not concurrently.

**Rationale**:
- Concurrent execution would conflict (file writes, git operations)
- Sequential execution preserves audit trail
- Stagnation detection relies on file/DB snapshots between sessions

**Emergent Loop**: If tasks generate more tasks, the sequence continues indefinitely. But the core mechanism is a queue, not a loop.

### 8.3 Living Numbers

**File Naming**: Specs use three-digit living numbers with gaps (000, 010, 020).

**Rationale**:
- Allows insertion between specs without renumbering entire directory
- Numbers represent **logical grouping**, not chronological order
- Related specs should be numerically adjacent

**Example**:
```
.specs/
├── 010_TRACEABILITY.md
├── 015_CODE_COVERAGE.md  ← Inserted later
├── 020_ANTI_GAMING.md
```

### 8.4 Centralized Error Handling

**All Rust errors** flow through `crates/ralph-errors`:

```rust
use ralph_errors::{codes, RalphResultExt};

Connection::open(path)
  .ralph_err(codes::DB_OPEN, "Failed to open database")?;
```

**Benefit**: Consistent error codes and messages across all crates. No crate defines its own error types (except internal-only enums).

---

## 9. Testing Architecture

### 9.1 Test Pyramid

```
         ┌──────────────┐
         │ Chaos Tests  │  (Gremlins.js)
         └──────────────┘
       ┌────────────────────┐
       │  E2E Tests         │  (Automation runner)
       └────────────────────┘
    ┌───────────────────────────┐
    │  Visual Regression Tests  │  (Automation runner snapshots)
    └───────────────────────────┘
  ┌─────────────────────────────────┐
  │  Integration Tests (Frontend)   │  (Vitest + Testing Library)
  └─────────────────────────────────┘
┌──────────────────────────────────────┐
│  Unit Tests (Rust + Frontend)        │  (cargo test + vitest)
└──────────────────────────────────────┘
```

**Test Commands**:
```bash
just test           # All tests
just test-rust      # cargo test
just test-frontend  # vitest
just test-e2e       # automation-runner
just test-visual    # visual regression
just test-monkey    # chaos testing
```

### 9.2 Mock Project Fixtures

**Test Workflow**:
1. `fixtures/` contains clean mock projects with `.undetect-ralph/` (not `.ralph/`)
2. `just reset-mock` copies fixtures → gitignored `mock/` and renames to `.ralph/`
3. Tests run against `mock/` (disposable)
4. Fixtures stay clean for next test run

**Mock Projects**:
- `00-empty-project` — Minimal valid project
- `01-desktop-blank` — Empty tasks
- `02-desktop-feature` — One feature, multiple tasks
- `03-desktop-tasks` — Complex dependency graph
- `04-desktop-dev` — Real development scenario

---

## 10. Security & Safety Considerations

### 10.1 Unsafe Code Policy

**Workspace Lint**: `unsafe_code = "deny"`

**Rationale**: Desktop app with no FFI requirements. All unsafe patterns are avoidable.

### 10.2 Subprocess Sandboxing

**Current State**: Claude CLI runs with `--permission-mode bypassPermissions`.

**Rationale**: Task execution requires file system access. Prompts instruct Claude to operate only within project directory.

**Future Enhancement**: Containerization or chroot jail for full sandboxing.

### 10.3 Rate Limiting

**Claude API Rate Limits**: Parsed from JSON stream events (`overloaded_error`, `rate_limit_error`).

**Response**:
- Set task status to `rate_limited`
- Retry after 5 minutes
- User can abort if desired

---

## 11. Performance Characteristics

### 11.1 SQLite Performance

**Optimizations**:
- WAL mode (concurrent readers + single writer)
- PRAGMA `synchronous = NORMAL` (balance durability/speed)
- Prepared statements (reused queries)
- Indexes on foreign keys and frequently queried columns

**Benchmark** (informal): 10,000 task inserts in ~200ms on NVMe SSD.

### 11.2 Prompt Build Performance

**Prompt Builder** is CPU-bound (string concatenation).

**Typical Timing**:
- Small project (10 tasks, 3 features): ~5ms
- Large project (500 tasks, 20 features): ~50ms

**Bottleneck**: Reading files (`git diff`, `learnings.txt`). Prompt build itself is negligible.

### 11.3 PTY Streaming Throughput

**XTerm.js** handles 10MB/s without frame drops.

**Typical Output**: Claude CLI emits ~1-10KB/s (conversational).

**Stress Test**: 100MB piped output → UI remains responsive (credit to XTerm.js virtual rendering).

---

## 12. Deployment & Distribution

### 12.1 Build Targets

**Platform Support**:
- Linux (primary): x86_64, Alder Lake optimized
- Windows: planned (Tauri supports NSIS installer)
- macOS: planned (Tauri supports DMG)

**Current Deployment**: Single binary built with `cargo tauri build`.

### 12.2 Build Profiles

```toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = true
panic = "abort"

[profile.release-dev]  # Fast iterative builds
inherits = "release"
lto = false
codegen-units = 16
strip = false
```

**Build Time**:
- `dev` profile: ~30s incremental
- `release` profile: ~4min clean build
- `release-dev` profile: ~90s clean build

---

## 13. Comparison to Related Systems

| System | Ralph4days | Devin | AutoGPT | LangGraph |
|--------|-----------|-------|---------|-----------|
| **Architecture** | Orchestrator (thin wrapper) | Full reimplementation | Agent loop | Agent graph |
| **Code Execution** | Delegates to Claude Code | Custom sandbox | Python REPL | Configurable |
| **State Management** | SQLite + task queue | Proprietary | In-memory | Graph DB |
| **Prompting** | Modular recipes | Opaque | Templates | Nodes |
| **Human-in-Loop** | `ask()` signals | Chat interface | None | Callbacks |
| **Open Source** | Yes (planned) | No | Yes | Yes |

**Key Differentiator**: Ralph is the only system that **delegates** to an existing code agent (Claude Code) rather than reimplementing those capabilities.

---

## 14. Future Roadmap

### 14.1 Planned Features

1. **Containerized Execution**: Docker/Podman isolation for tasks
2. **Multi-Model Support**: GPT-4, Gemini Pro via unified interface
3. **Team Collaboration**: Shared `.ralph/` over git with conflict resolution
4. **Learning Deduplication**: Deduplicate repeated learnings across sessions
5. **Answer UI**: Human interface for `ask(blocking=true)` questions

### 14.2 Research Directions

1. **Prompt Optimization**: A/B testing different recipe compositions
2. **Learning Relevance Ranking**: Semantic similarity for learning retrieval
3. **Stagnation Prediction**: ML model to predict stagnation before 3 sessions
4. **Task Dependency Inference**: Auto-detect dependencies from task descriptions

---

## 15. Key Takeaways for CS Curriculum

### 15.1 Patterns Demonstrated

1. **Orchestrator Pattern**: Coordinating external agents without reimplementing capabilities
2. **Pure Functional Core**: Prompt builder has no side effects (testable, composable)
3. **Event-Driven Architecture**: PTY streaming with async event emission
4. **Structured Signaling**: MCP tools as structured output channel
5. **Living Documentation**: Specs co-evolve with code (traceability)

### 15.2 Trade-offs & Constraints

**Chosen Simplicity**:
- Sequential execution (simpler) vs. concurrent (more efficient)
- SQLite (embedded) vs. PostgreSQL (scalable)
- Single project lock (simple) vs. multi-project (complex)

**Token Conservation** (critical constraint):
- Stubs before full implementations
- No dead code (every field must be consumed)
- Pure functions prevent wasted error handling boilerplate

### 15.3 Novel Contributions

1. **"Thin Wrapper" Philosophy**: Minimize orchestration logic, maximize delegation
2. **MCP Exhaust Pipe**: Bidirectional signaling without custom protocol
3. **Living Numbers**: Specification numbering that supports insertion
4. **Recipe-Based Prompts**: Composable prompt engineering

---

## References

- Tauri Documentation: https://tauri.app/
- Model Context Protocol: https://modelcontextprotocol.io/
- React 19 Compiler: https://react.dev/learn/react-compiler
- SQLite WAL Mode: https://sqlite.org/wal.html
- Specification-Driven Development: See `.specs/000_SPECIFICATION_FORMAT.md`

---

**Document Metadata**
- **Created**: 2026-02-11
- **Target Audience**: Computer Science Lecturers / Graduate Students
- **Codebase Version**: ralph4days v0.1.0 (pre-release)
- **License**: To be determined (currently private research project)

---

# Addendum: Core Domain Model & Advanced Features

### 6. The Three-Level Organizational Hierarchy

Ralph organizes work through a **three-tier taxonomy**: Disciplines → Features → Tasks. This hierarchy provides both organizational structure and intelligent prompt composition.

```
┌─────────────────────────────────────────────────────┐
│                   DISCIPLINES                       │
│  (Roles/Personas: Backend, Frontend, DevOps, etc.) │
│  • System prompt defining persona                  │
│  • Skills list (capabilities)                      │
│  • Conventions (code style, patterns)              │
│  • MCP servers (discipline-specific tools)         │
└────────────┬────────────────────────────────────────┘
             │
             │  Tasks belong to
             │  ONE discipline
             ▼
┌─────────────────────────────────────────────────────┐
│                    FEATURES                         │
│  (High-level capabilities: auth, payments, etc.)    │
│  • Description & architecture notes                │
│  • Knowledge paths (codebase locations)            │
│  • Context files (always-relevant files)           │
│  • Feature comments (learnings, gotchas, patterns) │
│  • Dependencies (other features)                   │
└────────────┬────────────────────────────────────────┘
             │
             │  Tasks implement
             │  ONE feature
             ▼
┌─────────────────────────────────────────────────────┐
│                     TASKS                           │
│  (Atomic work units: "Add login form", etc.)        │
│  • Title, description, acceptance criteria         │
│  • Status (draft → pending → in_progress → done)  │
│  • Priority (critical/high/medium/low)             │
│  • Dependencies (other tasks)                      │
│  • Context files (task-specific)                   │
│  • Pseudocode (from enrichment phase)              │
│  • Comments (notes, blockers, updates)             │
│  • Provenance (agent/human/system created)         │
└─────────────────────────────────────────────────────┘
```

---

### 7. Disciplines: The Persona System

**Disciplines are role definitions** that shape how Claude approaches a task. Each discipline is a complete persona with:

#### 7.1 Discipline Schema

```typescript
interface Discipline {
  name: string              // Unique ID: "backend", "frontend"
  display_name: string      // Human-readable: "Backend Developer"
  acronym: string           // 2-4 chars: "BE", "FE", "OPS"
  icon: string              // Lucide icon name: "Server", "Code"
  color: string             // Tailwind color: "#8b5cf6"

  // Persona definition
  system_prompt: string     // Identity & approach
  skills: string[]          // Capabilities list
  conventions: string       // Code style & patterns

  // Tooling
  mcp_servers: McpServerConfig[]  // Discipline-specific MCP servers

  // Visual (optional)
  image_path: string        // Generated AI image
  crops: string             // Image crop regions (JSON)
  image_prompt: string      // Generation prompt
}
```

#### 7.2 How Disciplines Work

**Prompt Injection**: When a task is assigned to a discipline, the **discipline_persona section** injects:

```markdown
## You Are a Backend Developer (🔧 BE)

You are a senior backend developer specializing in Rust, SQL, and API design.
Your code is production-grade: correct, secure, and maintainable.

### Your Skills
- Rust async programming with Tokio
- SQLite optimization (WAL mode, indexes, prepared statements)
- REST API design following OpenAPI 3.0
- Database migrations with rusqlite_migration

### Your Conventions
- All database errors use ralph_errors::ralph_err!() macro
- Foreign key constraints are mandatory
- Transaction isolation for ACID guarantees
- Snake_case for all identifiers
```

**MCP Server Binding**: Each discipline can specify MCP servers that are added to Claude's environment:

```rust
McpServerConfig {
    name: "browser-tools",
    command: "npx",
    args: vec!["@anthropic/browser-tools"],
    env: {"NODE_ENV": "development"}
}
```

When a **Frontend** task runs, Claude gets access to browser automation. When a **Backend** task runs, Claude gets database tools. **The toolset is tailored to the role.**

**Key Insight**: Disciplines are **not** just metadata—they fundamentally change Claude's behavior by injecting identity, constraints, and tools.

---

### 8. Features: Knowledge Scoping

**Features are high-level capabilities** (authentication, payments, reporting). They serve as **knowledge boundaries** and **context containers**.

#### 8.1 Feature Schema

```typescript
interface Feature {
  name: string              // Unique ID: "authentication"
  display_name: string      // "User Authentication"
  acronym: string           // "AUTH"
  description: string       // High-level purpose

  // Codebase mapping
  knowledge_paths: string[] // Dirs to watch: ["src/auth", "lib/jwt"]
  context_files: string[]   // Always-relevant: ["src/auth/mod.rs"]
  dependencies: string[]    // Other features: ["user-management"]

  // Accumulated knowledge
  comments: FeatureComment[]  // Learnings, gotchas, patterns

  // State
  status: 'active' | 'deprecated' | 'planned'
  created: string
}
```

#### 8.2 Feature Comments: Accumulated Wisdom

**FeatureComment** is the core of Ralph's memory system:

```typescript
interface FeatureComment {
  id: number
  category: string          // "gotcha" | "architecture" | "convention" | ...
  discipline: string?       // Which discipline contributed this
  agent_task_id: number?    // Task that produced this learning
  body: string              // Full detailed explanation
  summary: string?          // 1-sentence distillation
  reason: string?           // Why this matters
  source_iteration: number? // Which iteration discovered this
  created: string
}
```

**Categories** organize knowledge:
- **gotcha**: Traps, edge cases, non-obvious behavior
- **architecture**: Structural decisions ("Uses layered architecture")
- **convention**: Code patterns ("Always validate at boundaries")
- **discovery**: Factual findings ("Auth uses JWT, not sessions")
- **decision**: Design choices with rationale

**Example**:
```json
{
  "category": "gotcha",
  "body": "bcrypt work factor must be ≤31 or libsodium panics. Use 12 for dev, 14 for prod.",
  "summary": "bcrypt factor ≤31, use 12 (dev) / 14 (prod)",
  "reason": "Prevent runtime panic in password hashing",
  "discipline": "backend"
}
```

**Prompt Injection**: The `feature_context` section injects comments grouped by category:

```markdown
## Feature: User Authentication

Handles user login, session management, and password reset flows.

### Feature Knowledge

**architecture:**
- Uses JWT tokens stored in httpOnly cookies
- Refresh token rotation on every access

**gotcha:**
- bcrypt factor ≤31, use 12 (dev) / 14 (prod) (why: Prevent runtime panic)
- Always validate token signature before trusting claims

**convention:**
- Password validation happens in auth/validate.rs
- Session expiry is 15 minutes (configurable)
```

---

### 9. Tasks: The Atomic Work Unit

Tasks are **the smallest schedulable unit** in Ralph. Each task has a lifecycle: draft → pending → in_progress → done.

#### 9.1 Task Schema (Extended)

```typescript
interface Task {
  id: number
  feature: string           // FK to features.name
  discipline: string        // FK to disciplines.name
  title: string             // "Implement password reset endpoint"
  description: string?      // Detailed requirements

  // Workflow state
  status: TaskStatus        // draft | pending | in_progress | done | blocked | skipped
  priority: Priority?       // critical | high | medium | low
  provenance: TaskProvenance // agent | human | system

  // Dependencies
  depends_on: number[]      // Task IDs that must complete first
  blocked_by: string?       // External blocker description

  // Enrichment (added by enrichment recipe)
  pseudocode: string?       // Concrete implementation plan
  acceptance_criteria: string[] // Testable outcomes
  context_files: string[]   // Relevant source files
  enriched_at: string?      // When enrichment happened

  // Metadata
  tags: string[]
  hints: string?            // Guidance for agent
  estimated_turns: number?  // Expected Claude interactions
  created: string
  updated: string?
  completed: string?

  // UI denormalization (from JOINs)
  feature_display_name: string
  feature_acronym: string
  discipline_display_name: string
  discipline_acronym: string
  discipline_icon: string
  discipline_color: string

  comments: TaskComment[]   // Notes, updates, blockers
}
```

#### 9.2 Task Lifecycle & Enrichment

**Two-Phase Creation**:

1. **Draft Phase** (agent/human creates stub):
   ```sql
   INSERT INTO tasks (feature, discipline, title, description, status)
   VALUES ('auth', 'backend', 'Password reset endpoint', 'Add /api/auth/reset', 'draft');
   ```

2. **Enrichment Phase** (Opus adds detail):
   ```sql
   UPDATE tasks SET
     pseudocode = '1. Validate email\n2. Generate token\n3. Send email\n4. Store token with expiry',
     acceptance_criteria = '["Returns 200 on valid email","Sends email with reset link","Token expires in 1 hour"]',
     context_files = '["src/auth/reset.rs","src/email/templates.rs"]',
     status = 'pending',
     enriched_at = datetime('now')
   WHERE id = 42;
   ```

**Rationale**: Separating creation from enrichment allows:
- Fast brainstorming (create 20 draft tasks in 2 minutes)
- Batch enrichment (Opus reviews all drafts, adds pseudocode/criteria)
- Quality gate (only enriched tasks enter queue)

#### 9.3 Dependency Management

Tasks can **depend on other tasks**:

```typescript
{
  "id": 5,
  "title": "Add password reset UI",
  "depends_on": [42],  // Blocked until task #42 (API endpoint) completes
  "status": "blocked"
}
```

**Circular Dependency Prevention**: Before allowing `task A depends_on task B`, Ralph runs **depth-first search** to ensure no cycle:

```rust
fn has_circular_dependency(&self, task_id: u32, dep_id: u32) -> Result<bool, String> {
    let deps_map: HashMap<u32, Vec<u32>> = /* load all tasks */;
    let mut visited = HashSet::new();
    let mut stack = vec![dep_id];

    while let Some(current_id) = stack.pop() {
        if current_id == task_id { return Ok(true); } // Cycle detected!
        if !visited.insert(current_id) { continue; }
        if let Some(deps) = deps_map.get(&current_id) {
            stack.extend(deps);
        }
    }
    Ok(false)
}
```

**Auto-Unblocking**: When a task completes, Ralph queries all tasks that `depends_on` it and unblocks them if all dependencies are satisfied.

---

### 10. RAG & Embeddings: Feature-Scoped Memory

Ralph implements a **Retrieval-Augmented Generation (RAG)** system to inject only the most relevant learnings into prompts.

#### 10.1 RAG Architecture

```
┌─────────────────────────────────────────────────────┐
│         CLAUDE CLI (stream-json output)             │
│  Emits: errors, decisions, files touched            │
└────────────┬────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────────┐
│         EXTRACTION PIPELINE (ralph-rag)             │
│  Parses JSON stream → IterationRecord               │
└────────────┬────────────────────────────────────────┘
             │
             ├────────► JSONL Journal (.ralph/db/memory/{feature}.jsonl)
             │          Source of truth, append-only, git-trackable
             │
             └────────► EMBEDDING PIPELINE
                        ↓
          ┌─────────────────────────────────────┐
          │  Ollama (nomic-embed-text:v1.5)     │
          │  Generates 768-dim embeddings        │
          └────────┬────────────────────────────┘
                   │
                   ▼
          ┌─────────────────────────────────────┐
          │  SQLite comment_embeddings table    │
          │  Stores: embedding vector (BLOB)    │
          │          text hash (dedup)          │
          │          feature scope              │
          └────────┬────────────────────────────┘
                   │
                   ▼  (on next task)
          ┌─────────────────────────────────────┐
          │  SEMANTIC SEARCH                    │
          │  1. Embed task description          │
          │  2. Cosine similarity vs all        │
          │     feature comments                │
          │  3. Return top-K (K=5 default)      │
          └────────┬────────────────────────────┘
                   │
                   ▼
          ┌─────────────────────────────────────┐
          │  PROMPT BUILDER                     │
          │  Injects top learnings into prompt  │
          └─────────────────────────────────────┘
```

#### 10.2 Embedding Text Construction

**Format**:
```
{category}: {body} (why: {reason})
```

**Example**:
```
gotcha: bcrypt work factor must be ≤31 or libsodium panics (why: Prevent runtime panic in password hashing)
```

**SHA-256 Hash**: Before embedding, Ralph computes a hash:
```rust
pub fn hash_text(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hex::encode(hasher.finalize())
}
```

**Deduplication**: If the hash already exists in `comment_embeddings`, skip re-embedding (saves Ollama API calls).

#### 10.3 Semantic Search at Prompt Time

**Query Construction**:
1. Agent starts task #42 ("Implement password reset endpoint")
2. Embed task title + description → 768-dim vector
3. Query database:
   ```sql
   SELECT category, body, summary, reason, embedding
   FROM comment_embeddings
   WHERE feature = 'authentication'
   ```
4. Compute cosine similarity for each comment
5. Return top-5 by score

**Cosine Similarity** (in-memory):
```rust
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot_product / (mag_a * mag_b)
}
```

**Prompt Injection** (feature_context section):
```markdown
### Feature Knowledge (most relevant)

**gotcha:**
- bcrypt factor ≤31, use 12 (dev) / 14 (prod) (why: Prevent runtime panic)  [score: 0.89]
- Email validation uses regex from auth/validation.rs  [score: 0.76]

**architecture:**
- Token storage in Redis with 1hr TTL  [score: 0.71]
```

**Fallback**: If RAG is unavailable (Ollama down, no embeddings), inject **all** feature comments (up to 50).

---

### 11. Prompt Building: Context-Driven Composition

Prompts are built via **pure function composition** over a rich context object.

#### 11.1 PromptContext Structure

```rust
pub struct PromptContext {
    // Database state (pre-queried by caller)
    pub features: Vec<Feature>,
    pub tasks: Vec<Task>,
    pub disciplines: Vec<Discipline>,
    pub metadata: ProjectMetadata,

    // Pre-read file contents
    pub file_contents: HashMap<String, String>,

    // State files
    pub progress_txt: Option<String>,     // .ralph/progress.txt
    pub learnings_txt: Option<String>,    // .ralph/learnings.txt
    pub claude_ralph_md: Option<String>,  // .ralph/CLAUDE.RALPH.md

    // Paths
    pub project_path: String,
    pub db_path: String,
    pub script_dir: String,               // Where to write MCP scripts

    // Prompt-specific
    pub user_input: Option<String>,
    pub target_task_id: Option<u32>,
    pub target_feature: Option<String>,

    // Filesystem snapshot (for braindump)
    pub codebase_snapshot: Option<CodebaseSnapshot>,

    // Per-section overrides
    pub instruction_overrides: HashMap<String, String>,

    // RAG
    pub relevant_comments: Option<Vec<ScoredFeatureComment>>,
}
```

**Key Properties**:
- **Immutable**: Context is constructed once, never mutated
- **Pre-loaded**: All database queries and file reads happen **before** prompt building
- **No I/O in builders**: Section builders are pure functions (`&PromptContext -> Option<String>`)

#### 11.2 Section Builders

Each section is a **named function**:

```rust
pub struct Section {
    pub name: &'static str,
    pub build: fn(&PromptContext) -> Option<String>,
}
```

**Example** (discipline_persona):

```rust
fn build(ctx: &PromptContext) -> Option<String> {
    let discipline = ctx.target_task_discipline()?;  // None if no task
    let system_prompt = discipline.system_prompt.as_ref()?;

    let mut out = format!(
        "## You Are a {} ({} {})\n\n{system_prompt}",
        discipline.display_name, discipline.icon, discipline.acronym
    );

    if !discipline.skills.is_empty() {
        out.push_str("\n\n### Your Skills\n\n");
        for skill in &discipline.skills {
            out.push_str(&format!("- {skill}\n"));
        }
    }

    if let Some(conventions) = &discipline.conventions {
        out.push_str(&format!("\n\n### Your Conventions\n\n{conventions}"));
    }

    Some(out)
}
```

**Returning `None`**: Skips the section entirely (e.g., no discipline → no persona section).

#### 11.3 Recipe Execution

**TaskExecution Recipe**:
```rust
Recipe {
    name: "task_execution",
    sections: vec![
        sections::project_context(),      // "Project: ralph4days"
        sections::discipline_persona(),   // "You Are a Backend Developer"
        sections::feature_context(),      // "Feature: Authentication" + RAG learnings
        sections::feature_files(),        // Context files from feature
        sections::feature_state(),        // Other tasks in this feature
        sections::state_files(),          // progress.txt, learnings.txt
        sections::previous_attempts(),    // Prior sessions on this task (if re-queued)
        sections::dependency_context(),   // Tasks this depends on
        sections::task_details(),         // Title, description, acceptance criteria
        sections::task_files(),           // Task-specific context files
        sections::task_exec_instructions(), // "Your job is to..."
    ],
    mcp_tools: vec![
        McpTool::SetTaskStatus,
        McpTool::AppendLearning,
        McpTool::AddContextFile,
    ],
}
```

**Execution** (`execute_recipe`):
```rust
pub fn execute_recipe(recipe: &Recipe, ctx: &PromptContext) -> PromptOutput {
    let mut prompt = String::new();
    for section in &recipe.sections {
        if let Some(text) = (section.build)(ctx) {
            prompt.push_str(&text);
            prompt.push_str("\n\n");
        }
    }
    PromptOutput { prompt, mcp_tools: recipe.mcp_tools.clone() }
}
```

**Order Matters**: Sections are appended in order. Due to **recency bias**, the most important context appears **last** (task details, instructions).

---

### 12. Dynamic MCP Server Generation

Ralph generates a **custom bash MCP server** for each task session.

#### 12.1 MCP Server Architecture

**Single Reusable Template**: One bash script template handles all tools. Tools are added/removed by changing which if/elif blocks are generated.

**Environment Variables**:
```bash
RALPH_DB='/path/to/project/.ralph/db/ralph.db'
PROJECT_PATH='/path/to/project'
```

**JSON-RPC Protocol**:
```bash
# Read stdin line-by-line
while IFS= read -r line; do
    # Extract request ID
    id=$(echo "$line" | sed -n 's/.*"id"[[:space:]]*:[[:space:]]*\([0-9]*\).*/\1/p')

    # Dispatch by method
    if echo "$line" | grep -q '"method"[[:space:]]*:[[:space:]]*"initialize"'; then
        handle_initialize "$id"
    elif echo "$line" | grep -q '"method"[[:space:]]*:[[:space:]]*"tools/list"'; then
        printf '{"jsonrpc":"2.0","id":%s,"result":{"tools":[...]}}' "$id"
    elif echo "$line" | grep -q '"method"[[:space:]]*:[[:space:]]*"tools/call"'; then
        tool_name=$(echo "$line" | sed -n 's/.*"name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        if [ "$tool_name" = "set_task_status" ]; then
            # Extract parameters, call sqlite3, return result
            ...
        fi
    fi
done
```

#### 12.2 Tool Handler Generation

**Example** (SetTaskStatus):

```bash
if [ "$tool_name" = "set_task_status" ]; then
    task_id=$(echo "$line" | sed -n 's/.*"id"[[:space:]]*:[[:space:]]*\([0-9]*\).*/\1/p')
    status=$(echo "$line" | sed -n 's/.*"status"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
    e_status=$(json_escape "$status")

    if [ "$status" = "done" ]; then
        sqlite3 "$RALPH_DB" "UPDATE tasks SET status='${e_status}', completed=datetime('now') WHERE id=${task_id};"
    else
        sqlite3 "$RALPH_DB" "UPDATE tasks SET status='${e_status}' WHERE id=${task_id};"
    fi

    result="Task #${task_id} status set to: $status"
    e_result=$(json_escape "$result")
    printf '{"jsonrpc":"2.0","id":%s,"result":{"content":[{"type":"text","text":"%s"}]}}\n' "$id" "$e_result"
fi
```

**Key Features**:
- **Parameter extraction** via sed
- **SQL injection prevention** via json_escape (escapes quotes, backslashes)
- **Direct SQLite writes** (no Rust layer)
- **JSON-RPC response** with proper formatting

#### 12.3 Discipline-Specific MCP Servers

**Config Generation**:
```rust
fn generate_config(ctx: &PromptContext, filename: &str) -> String {
    let mut servers = Vec::new();

    // Ralph's own MCP server (always included)
    servers.push(format!("\"ralph\":{{\"command\":\"{}\",\"args\":[]}}", script_path));

    // Add discipline-specific servers
    if let Some(discipline) = ctx.target_task_discipline() {
        for mcp in &discipline.mcp_servers {
            servers.push(format!(
                "\"{name}\":{{\"command\":\"{command}\",\"args\":[{args}],\"env\":{{{env}}}}}",
                name = mcp.name,
                command = mcp.command,
                args = mcp.args.join(","),
                env = format_env(&mcp.env)
            ));
        }
    }

    format!("{{\"mcpServers\":{{{}}}}}", servers.join(","))
}
```

**Example Output** (Frontend task):
```json
{
  "mcpServers": {
    "ralph": {"command": "/tmp/ralph-mcp/ralph-mcp.sh", "args": []},
    "browser-tools": {
      "command": "npx",
      "args": ["@anthropic/browser-tools"],
      "env": {"NODE_ENV": "development"}
    }
  }
}
```

**Result**: When Claude runs a Frontend task, it has access to both `ralph` tools (set_task_status, append_learning) **and** `browser-tools` (screenshot, click, navigate).

---

### 13. Complete Task Execution Flow (End-to-End)

Let's trace a single task execution from queue selection to completion.

#### Step 1: Task Selection

```rust
// Ralph queries database for next pending task
SELECT * FROM tasks
WHERE status = 'pending'
  AND (depends_on = '[]' OR all_dependencies_complete())
ORDER BY priority DESC, id ASC
LIMIT 1;
```

**Result**: Task #42 ("Implement password reset endpoint", feature="auth", discipline="backend")

#### Step 2: Context Construction

```rust
// Load all database entities
let features = db.get_features();
let tasks = db.get_tasks();
let disciplines = db.get_disciplines();
let metadata = db.get_metadata();

// Read state files
let progress_txt = fs::read_to_string(".ralph/progress.txt").ok();
let learnings_txt = fs::read_to_string(".ralph/learnings.txt").ok();
let claude_ralph_md = fs::read_to_string(".ralph/CLAUDE.RALPH.md").ok();

// RAG: Semantic search for relevant comments
let task = db.get_task_by_id(42)?;
let task_embedding = ollama_embed(&format!("{} {}", task.title, task.description))?;
let relevant_comments = db.semantic_search("auth", &task_embedding, top_k=5)?;

// Read context files
let mut file_contents = HashMap::new();
let feature = features.iter().find(|f| f.name == "auth")?;
for path in &feature.context_files {
    file_contents.insert(path.clone(), fs::read_to_string(path)?);
}
for path in &task.context_files {
    file_contents.insert(path.clone(), fs::read_to_string(path)?);
}

// Construct context
let ctx = PromptContext {
    features,
    tasks,
    disciplines,
    metadata,
    file_contents,
    progress_txt,
    learnings_txt,
    claude_ralph_md,
    project_path: "/home/user/my-project".into(),
    db_path: "/home/user/my-project/.ralph/db/ralph.db".into(),
    script_dir: "/tmp/ralph-mcp".into(),
    target_task_id: Some(42),
    relevant_comments: Some(relevant_comments),
    ..Default::default()
};
```

#### Step 3: Prompt Building

```rust
let recipe = recipes::task_execution();
let output = execute_recipe(&recipe, &ctx);
```

**Generated Prompt** (simplified):
```markdown
# Project: MyApp

A web application for managing user accounts and payments.

## You Are a Backend Developer (🔧 BE)

You are a senior backend developer specializing in Rust, SQL, and API design.

### Your Skills
- Rust async programming with Tokio
- SQLite optimization (WAL mode, indexes)
- REST API design

### Your Conventions
- All database errors use ralph_errors::ralph_err!()
- Foreign key constraints are mandatory
- Snake_case for all identifiers

## Feature: User Authentication

Handles user login, session management, and password reset flows.

### Feature Knowledge (most relevant)

**gotcha:**
- bcrypt factor ≤31, use 12 (dev) / 14 (prod)
- Email validation uses regex from auth/validation.rs

**architecture:**
- Token storage in Redis with 1hr TTL

## Feature Files

### src/auth/mod.rs
[... file contents ...]

## Task #42: Implement password reset endpoint

**Description:**
Add POST /api/auth/reset endpoint. Accepts email, generates reset token, sends email.

**Acceptance Criteria:**
- Returns 200 on valid email
- Sends email with reset link
- Token expires in 1 hour
- Returns 404 if email not in system

**Context Files:**
- src/auth/reset.rs (create new)
- src/email/templates.rs

**Pseudocode:**
1. Validate email format
2. Check user exists
3. Generate random token (32 bytes)
4. Store token in Redis with user_id + expiry (1hr)
5. Send email via SendGrid
6. Return 200

## Instructions

Your job is to implement this task completely and correctly. Use your native Claude Code capabilities:
- Read files, search codebase, edit files
- Run tests, check compilation
- Commit when done

Signal completion via MCP:
- set_task_status(id=42, status="done") when fully tested
- append_learning(text="...") for discoveries
- add_context_file(task_id=42, file_path="...") for new files you create
```

#### Step 4: MCP Script Generation

```rust
let (mcp_scripts, mcp_config_json) = mcp::generate(&ctx, &recipe.mcp_tools);

// Write scripts to disk
fs::write("/tmp/ralph-mcp/ralph-mcp.sh", &mcp_scripts[0].content)?;
fs::set_permissions("/tmp/ralph-mcp/ralph-mcp.sh", 0o755)?; // Executable

// Write MCP config
fs::write("/tmp/ralph-mcp/mcp-config.json", &mcp_config_json)?;
```

**Generated Script** (simplified):
```bash
#!/usr/bin/env bash
RALPH_DB='/home/user/my-project/.ralph/db/ralph.db'
PROJECT_PATH='/home/user/my-project'

# ... (helper functions) ...

while IFS= read -r line; do
    id=$(extract_id "$line")

    if echo "$line" | grep -q '"tools/call"'; then
        tool_name=$(echo "$line" | sed -n 's/.*"name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')

        if [ "$tool_name" = "set_task_status" ]; then
            # ... (status update logic) ...
        elif [ "$tool_name" = "append_learning" ]; then
            text=$(echo "$line" | sed -n 's/.*"text"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
            echo "$text" >> "$PROJECT_PATH/.ralph/learnings.txt"
            printf '{"jsonrpc":"2.0","id":%s,"result":{"content":[{"type":"text","text":"Appended"}]}}\n' "$id"
        fi
    fi
done
```

#### Step 5: Claude CLI Execution

```rust
PTYManager::create_session(
    app_handle,
    session_id: "task-42-session-1",
    working_dir: "/home/user/my-project",
    mcp_config: Some("/tmp/ralph-mcp/mcp-config.json"),
    config: SessionConfig {
        model: Some("haiku"),
        max_turns: Some(50),
        thinking: Some("extended"),
    },
)?;

// Send prompt via stdin
pty_manager.send_input(session_id, prompt.as_bytes())?;
```

**Claude executes**:
1. Reads prompt, understands it's implementing password reset
2. Uses native capabilities:
   - Reads `src/auth/reset.rs` (doesn't exist, creates it)
   - Searches codebase for email validation regex
   - Edits `src/auth/mod.rs` to add `mod reset;`
   - Implements endpoint in `reset.rs`
   - Runs `cargo check`, `cargo test`
3. Calls MCP tools:
   ```json
   {"method": "tools/call", "params": {"name": "add_context_file", "arguments": {"task_id": 42, "file_path": "src/auth/reset.rs"}}}
   {"method": "tools/call", "params": {"name": "append_learning", "arguments": {"text": "SendGrid API key must be in .env as SENDGRID_API_KEY"}}}
   {"method": "tools/call", "params": {"name": "set_task_status", "arguments": {"id": 42, "status": "done"}}}
   ```

#### Step 6: Post-Processing

```rust
// Claude CLI exits (exit code 0)

// Ralph reads database
let task = db.get_task_by_id(42)?;
assert_eq!(task.status, TaskStatus::Done);

// Check learnings.txt
let learnings = fs::read_to_string(".ralph/learnings.txt")?;
assert!(learnings.contains("SendGrid API key"));

// Check context_files
assert!(task.context_files.contains(&"src/auth/reset.rs".to_owned()));

// Stagnation check
let post_hash = hash_project_state(&project_path)?;
if post_hash == pre_hash {
    stagnation_count += 1;
    if stagnation_count >= 3 {
        // Abort loop
    }
} else {
    stagnation_count = 0;
}

// Select next task
let next_task = db.get_next_pending_task()?;
// ... repeat from Step 1 ...
```

---

### 14. Novel Contributions (Summary)

#### 14.1 Architectural Innovations

1. **Pure Prompt Functions**: Separating prompt building from I/O enables testing, caching, and composition
2. **Dynamic MCP Per-Task**: Generating bash MCP servers on-the-fly based on discipline + recipe eliminates static config
3. **Feature-Scoped RAG**: Semantic search constrained by feature prevents knowledge bleed across domains
4. **Three-Tier Taxonomy**: Disciplines (roles) → Features (capabilities) → Tasks (work items) provides both organization and prompt structure

#### 14.2 Design Patterns Demonstrated

| Pattern | Implementation | Benefit |
|---------|---------------|---------|
| **Thin Wrapper** | Ralph delegates to Claude Code, doesn't reimplement file ops | Minimal code, maximum leverage |
| **Pure Functional Core** | Prompt builder has no side effects | Testable without mocks |
| **Event Sourcing** | JSONL journals capture iteration history | Git-trackable, append-only, auditable |
| **Semantic Kernel** | Embeddings + cosine similarity for retrieval | Relevant knowledge injection |
| **Persona Injection** | Discipline system_prompt changes Claude's identity | Role-specific behavior |
| **Recipe Composition** | Sections are composable building blocks | Customizable prompts |

#### 14.3 Engineering Trade-offs

| Decision | Alternative | Why Ralph Chose This |
|----------|------------|---------------------|
| SQLite | PostgreSQL | Embedded, zero-config, WAL for concurrency |
| Sequential | Concurrent | Simpler, no file conflicts, audit trail |
| Bash MCP | Rust MCP | Faster generation, easier templating |
| Pure Functions | Stateful Builders | Testable without mocking I/O |
| Feature Scope | Global RAG | Prevents knowledge bleed |
| Haiku Default | Always Opus | Cost vs capability (Opus for reviews only) |

---

## Conclusion

Ralph4days is a **research-grade orchestration system** that demonstrates how to build autonomous multi-agent workflows **without reimplementing the agent**. By treating Claude Code as the execution engine and Ralph as the thin orchestration layer, the system achieves:

- **High leverage**: ~15k lines of Rust orchestrates unlimited Claude work
- **Composability**: Prompts are data (recipes), not code
- **Adaptability**: Dynamic MCP servers tailor toolsets per-task
- **Memory**: RAG ensures learnings propagate across iterations
- **Observability**: Event streams, SQLite queries, and JSONL journals provide full traceability

**For CS Education**: This architecture exemplifies separation of concerns, pure functional design, event sourcing, semantic search, and human-in-the-loop AI systems—all in a single, cohesive codebase.

---

**Document Version**: 2.0 (Extended Deep Dive)
**Last Updated**: 2026-02-11
**Target Audience**: Computer Science Lecturers, Graduate Students, AI Systems Researchers
