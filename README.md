# Ralph4days

**Autonomous multi-agent task orchestrator for Claude Code.** A Tauri 2.5 desktop application that plays tasks sequentially from a YAML database using Claude Haiku, with periodic Opus reviews for quality assurance.

## Overview

Ralph4days is a desktop application that orchestrates Claude Code instances to autonomously complete software development tasks. It acts as a thin wrapper around the Claude CLI -- launching instances, generating context-rich prompts, monitoring output, and detecting stagnation or completion.

The core execution is simple: Ralph picks the next pending task from the project's YAML database, assembles a prompt with full project context, runs Claude Haiku to execute the task, and checks for progress. Every few Haiku sessions, an Opus review cycle inspects recent work for bugs, code quality issues, and architectural problems. Execution continues until all tasks are complete, stagnation is detected, or the user intervenes. A loop emerges naturally if tasks create more tasks.

Ralph is deliberately not AI. It implements only deterministic, coded behavior: project structure enforcement, prompt generation from YAML data, subprocess lifecycle management, and state persistence. All intelligence comes from the Claude instances it orchestrates.

## Features

- **Task execution** -- Start, pause, resume, and stop task execution that picks and completes tasks sequentially
- **Haiku + Opus review cycle** -- Claude Haiku executes tasks for speed; periodic Opus reviews catch bugs and quality issues before they compound
- **Stagnation detection** -- SHA256 hashing of six project files across sessions detects when execution stops making progress, auto-aborting after three unchanged sessions
- **Rate limit handling** -- Parses Claude's JSON stream for `overloaded_error` and `rate_limit_error` events, backs off for five minutes, and retries automatically
- **Multi-file YAML database** -- Thread-safe, file-locked YAML storage in `.ralph/db/` with atomic writes (temp file + rename pattern) and circular dependency detection
- **Project locking** -- One project per session, chosen at startup via interactive picker or CLI flag, preventing cross-project confusion
- **Project scanner** -- Recursive home directory scan (5 levels deep, 100 project max) discovers existing Ralph-enabled projects automatically
- **Project initialization** -- Creates `.ralph/db/` structure with default disciplines, empty task/feature files, metadata, and a `CLAUDE.RALPH.md` template
- **Embedded terminal** -- Full PTY-backed Claude Code sessions inside the app via xterm.js, with model and thinking mode selection per tab
- **Dynamic MCP servers** -- Generates bash-based MCP server scripts on the fly, giving Claude instances access to `.ralph/db/` as tools and resources
- **Inferred task status** -- Computes display status from raw status + dependency graph (Ready, Waiting on Deps, Externally Blocked, In Progress, Done, Skipped)
- **10 default disciplines** -- Frontend, Backend, Wiring, Database, Testing, Infrastructure, Security, Documentation, Design, and API -- each with icon, color, and acronym
- **Task filtering** -- Filter tasks by status, feature, discipline, priority, and tags with a combined header control bar
- **Workspace tabs** -- Browser-style tabbed workspace panel for terminals, task details, forms, and braindump sessions
- **Task provenance tracking** -- Records whether tasks were created by an agent, human, or system, with structured comments for context and retry history
- **Execution context per task** -- Context files, output artifacts, hints, and estimated turns give Claude precise guidance for each task

## Architecture

```
Frontend (React 19 / Zustand / Tailwind v4)
    |
    | Tauri IPC
    v
Backend (Rust / Tauri 2.5 / Tokio)
    |
    |-- yaml_db      Thread-safe YAML database (fs2 file locking)
    |-- loop_engine   Start/stop/pause/resume, stagnation detection
    |-- claude_client  Subprocess management, JSON stream parsing
    |-- prompt_builder PRD aggregation, Haiku/Opus prompt generation
    |-- mcp_generator  Dynamic bash MCP server creation
    |-- terminal       PTY session management (portable-pty)
    |
    | subprocess (timeout + claude CLI)
    v
Claude Code (--output-format stream-json, --max-turns 50)
```

The frontend renders a split-panel layout: task/feature/discipline pages on the left, workspace tabs (terminals, forms, detail views) on the right. All state flows through Tauri IPC commands; the frontend never touches the filesystem directly.

The backend manages task execution on a dedicated thread, streaming output events (`ralph://state_changed`, `ralph://output_chunk`, `ralph://task_complete`, `ralph://rate_limited`, `ralph://error`) to the frontend via Tauri's event system.

## Prerequisites

- **Rust** (1.75+) and **Cargo**
- **Bun** (JavaScript runtime and package manager)
- **Claude CLI** -- `claude` must be on your PATH ([Claude Code](https://docs.anthropic.com/en/docs/claude-code))
- **just** -- Command runner (`cargo install just`)
- **Linux** with WebKitGTK (Tauri requirement): `libwebkit2gtk-4.1-0` / `webkit2gtk4.1`

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/ralph4days.git
cd ralph4days

# Install frontend dependencies
bun install

# Build release binary (optimized, stripped, LTO)
just build

# The binary is at src-tauri/target/release/ralph
```

### Linux Packages

```bash
# Build .deb, .rpm, and .appimage packages
just release-linux
```

## Quick Start

### 1. Initialize a project

Point Ralph at an existing codebase to create the `.ralph/` structure:

```bash
# Interactive mode -- opens the project picker
ralph

# Or lock directly to a project via CLI
ralph --project /path/to/your-project
```

In the project picker, choose **Initialize Existing Project**, browse to your project directory, and click **Initialize Ralph**. This creates:

```
your-project/
  .ralph/
    db/
      tasks.yaml          # Task records
      features.yaml       # Feature definitions
      disciplines.yaml    # Discipline definitions (10 defaults)
      metadata.yaml       # Project metadata and ID counters
    CLAUDE.RALPH.md       # Project context template for Claude
```

### 2. Define tasks

Create features and tasks through the UI forms, or let Claude structure them from a braindump. Each task belongs to a feature and a discipline:

```yaml
# .ralph/db/tasks.yaml
tasks:
- id: 1
  feature: authentication
  discipline: backend
  title: Implement login API
  description: Create REST API endpoints for user authentication
  status: pending
  priority: high
  tags: [api, security]
  acceptance_criteria:
  - POST /login endpoint works
  - Returns JWT token
  context_files:
  - src/auth/mod.rs
  output_artifacts:
  - src/routes/auth.rs
  hints: Use bcrypt for password hashing
  estimated_turns: 3
  provenance: agent
```

### 3. Run the loop

Click the loop toggle in the bottom bar to start the autonomous build cycle. Ralph generates a prompt containing the full PRD (all four YAML files), progress log, learnings, and project context, then launches Claude Haiku to pick and complete one task per iteration.

## Usage

### Interactive Mode

```bash
ralph
```

Opens the project picker modal. Scans your home directory for projects containing `.ralph/` folders. Select one or initialize a new project from any directory.

### CLI Mode

```bash
ralph --project /path/to/your-project
```

Validates the project path, locks it for the session, and opens directly to the workspace -- skipping the project picker.

### Workspace Layout

The application uses a resizable split-panel layout:

**Left panel** -- Three navigable pages:
- **Tasks** -- Filterable task list with progress bar, grouped by feature/discipline. Click a task to open its detail view.
- **Features** -- Feature definitions with task count statistics.
- **Disciplines** -- Discipline configuration with icons and colors.

**Right panel** -- Tabbed workspace:
- **Terminal tabs** -- Full PTY Claude Code sessions with model selection (Haiku, Sonnet, Opus) and thinking mode toggle.
- **Task detail** -- Read-only view of task metadata, acceptance criteria, dependencies, and execution context.
- **Forms** -- Task creation, feature creation, discipline creation, and braindump forms.

### Loop States

| State | Description |
|-------|-------------|
| **Idle** | No loop running. Ready to start. |
| **Running** | Actively executing Claude iterations. |
| **Paused** | Loop suspended. Resume to continue. |
| **Rate Limited** | Backed off due to API rate limits. Auto-retries after 5 minutes. |
| **Complete** | All tasks marked as done. |
| **Aborted** | Stopped manually, stagnation detected, or max retries exceeded. |

### Loop Configuration

The loop engine uses these defaults:

| Parameter | Default | Description |
|-----------|---------|-------------|
| `haiku_iterations_before_opus` | 3 | Haiku runs between each Opus review |
| `max_stagnant_iterations` | 3 | Unchanged iterations before auto-abort |
| `iteration_timeout_secs` | 900 | Per-iteration timeout (15 minutes) |
| `rate_limit_retry_secs` | 300 | Backoff duration on rate limit (5 minutes) |
| `max_rate_limit_retries` | 5 | Rate limit retries before abort |

## Database Schema

Ralph stores project data in `.ralph/db/` as four YAML files:

### `tasks.yaml`

Task records with rich metadata:
- **Core fields:** `id`, `feature`, `discipline`, `title`, `description`, `status`, `priority`
- **Organization:** `tags`, `depends_on` (dependency graph), `blocked_by` (external blocker text)
- **Execution context:** `context_files`, `output_artifacts`, `hints`, `estimated_turns`
- **History:** `provenance` (agent/human/system), `comments` (structured with author/body/timestamp), `created`/`updated`/`completed` timestamps
- **Acceptance criteria:** List of success conditions

Task statuses: `pending`, `in_progress`, `done`, `blocked`, `skipped`

Inferred statuses (computed from raw status + dependency graph): `ready`, `waiting_on_deps`, `externally_blocked`, `in_progress`, `done`, `skipped`

### `features.yaml`

Feature groupings that organize related tasks:
- `name` (internal slug), `display_name`, `acronym` (unique 3-4 char identifier)
- `description`, `created` timestamp
- `knowledge_paths`, `context_files` (feature-level context for prompts)

### `disciplines.yaml`

Engineering disciplines that categorize task type:
- `name`, `display_name`, `acronym`, `icon` (Lucide icon name), `color` (hex)
- `system_prompt`, `skills`, `conventions` (execution context for discipline-specific prompts)
- `mcp_servers` (discipline-specific MCP server configurations)

### `metadata.yaml`

Project-level metadata and internal counters:
- `project.title`, `project.description`, `project.created`
- `_counters` -- Tracks highest task ID per feature+discipline pair for safe concurrent ID assignment

## Dynamic MCP Servers

Ralph generates bash MCP server scripts at runtime that implement the MCP protocol (JSON-RPC 2.0 over stdio). These scripts give Claude Code instances direct access to the `.ralph/db/` files as MCP tools and resources.

Two modes are supported:

- **Task creation mode** -- Exposes `list_features`, `list_disciplines`, and `read_tasks` tools
- **Interactive mode** -- Exposes `read_db_file` and `list_db_files` tools, plus resource URIs (`ralph://db/tasks.yaml`, etc.)

Scripts are generated to a PID-scoped temp directory (`/tmp/ralph-mcp-<pid>/`) and cleaned up on shutdown.

## Development

### Setup

```bash
# Install dependencies
bun install

# Start dev server (frontend hot reload + Rust backend)
just dev

# Start with a mock project (skips project picker)
just reset-mock          # Create mock/ from fixtures/
just dev-mock 03         # Opens mock project by prefix
```

### Commands

```bash
just dev                 # Start development server
just build               # Build optimized release binary
just test                # Run all tests (Rust + frontend)
just test-rust           # Run Rust tests only
just test-frontend       # Run Vitest frontend tests
just test-e2e            # Run Playwright E2E tests
just test-visual         # Run visual regression tests
just test-monkey         # Run Gremlins.js chaos tests
just lint                # Run clippy + oxlint + biome
just fmt                 # Format all code (rustfmt + biome)
just check               # Run cargo check (fast compile check)
just clean               # Remove build artifacts
```

### Test Fixtures

The `fixtures/` directory contains read-only reference projects at various stages:

| Fixture | Description |
|---------|-------------|
| `00-empty-project` | Directory with no `.ralph/` folder |
| `01-desktop-blank` | Fresh `.ralph/db/` with Desktop stack disciplines |
| `02-desktop-feature` | Has features defined |
| `03-desktop-tasks` | Has tasks with dependencies |
| `04-desktop-dev` | Development-ready project with full task graph |

Use `just reset-mock` to copy fixtures into a disposable `mock/` directory for testing. The mock copy renames `.undetect-ralph/` to `.ralph/` (fixtures use the alternate name to avoid being detected by the project scanner).

### Project Structure

```
ralph4days/
  src/                          # Frontend (React 19 + TypeScript)
    components/
      ui/                       # 50+ shadcn/ui components
      prd/                      # Task list views (PRDHeader, PRDBody, PlaylistItem)
      workspace/                # Workspace tab contents (terminal, forms, details)
      layout/                   # Page layout components
    pages/                      # TasksPage, FeaturesPage, DisciplinesPage
    stores/                     # Zustand stores (useWorkspaceStore)
    hooks/                      # Custom hooks (useInvoke, usePRDData, usePRDFilters)
    types/                      # TypeScript type definitions
  src-tauri/                    # Backend (Rust + Tauri 2.5)
    src/
      lib.rs                    # Tauri app setup, CLI parsing, IPC handler registration
      commands.rs               # All Tauri IPC command handlers
      loop_engine.rs            # Build loop state machine
      claude_client.rs          # Claude CLI subprocess management
      prompt_builder.rs         # Haiku/Opus prompt generation
      mcp_generator.rs          # Dynamic bash MCP server creation
      types.rs                  # Core types (LoopState, LoopConfig, RalphEvent, etc.)
      terminal/                 # PTY session management
        manager.rs              # Session lifecycle (create, resize, terminate)
        session.rs              # Claude CLI settings, PTY session struct
        events.rs               # PTY output/closed event types
  crates/
    yaml-db/                    # YAML database crate (thread-safe, file-locked)
      src/
        database.rs             # YamlDatabase coordinator, CRUD operations
        tasks.rs                # TasksFile (tasks.yaml)
        features.rs             # FeaturesFile (features.yaml)
        disciplines.rs          # DisciplinesFile (disciplines.yaml, 10 defaults)
        metadata.rs             # MetadataFile (counters, project info)
        entity.rs               # Generic EntityFile<T> with atomic save
        migration.rs            # Acronym migration utilities
        acronym.rs              # Acronym generation and validation
  fixtures/                     # Read-only test fixture projects
  .specs/                       # Project specifications
  .docs/                        # Implementation notes and decision records
  justfile                      # Task runner commands
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| **Desktop framework** | Tauri 2.5 |
| **Frontend** | React 19, TypeScript, Vite 7, Tailwind CSS v4 |
| **State management** | Zustand 5 |
| **UI components** | shadcn/ui (50+ components), Lucide Icons |
| **Backend** | Rust (edition 2021), Tokio |
| **Database** | Custom YAML (serde_yaml) with fs2 file locking |
| **Terminal** | portable-pty + xterm.js |
| **Testing** | Vitest, Playwright, Gremlins.js |
| **Linting** | Clippy, oxlint, Biome |
| **Package manager** | Bun |
| **Task runner** | just |

## License

MIT
