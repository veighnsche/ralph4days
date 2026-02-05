# CLAUDE.md

## Project Overview

**Ralph Loop** — Tauri 2.5 desktop application for running autonomous, multi-agent build loops. Runs Claude Haiku in a loop to complete tasks from a PRD, with periodic Opus reviews for quality control.

**Not a general-purpose AI chat app.** This is a build automation tool that orchestrates Claude CLI sessions.

## Development Device

| Component | Specification |
|-----------|---------------|
| **Device** | Intel NUC12WSKi5 |
| **CPU** | Intel Core i5-1240P (12C/16T, Alder Lake) |
| **RAM** | 64 GB |
| **GPU** | NVIDIA RTX 3090 (24GB) + Intel Iris Xe |
| **Storage** | Samsung 980 PRO 1TB NVMe |
| **Display** | 1920x1080 @ 60Hz |
| **OS** | Ultramarine Linux 43 (Wayland/KDE) |

### Platform Requirements

- **Wayland required** — KDE Plasma on Wayland is the development environment
- **RTX 3090 available** — Not currently used by Ralph, but available for future local LLM integration
- Builds target x86_64 with Alder Lake optimizations

## Commands

```bash
# Development
bun dev                    # Vite dev server (port 1420)
bun tauri dev              # Full Tauri app with hot reload

# Build
bun build                  # Frontend production build
bun tauri build            # Full production app (release optimized)
bun tauri build --debug    # Debug build

# Run
ralph                      # Launch with ProjectPicker modal
ralph --project /path      # Launch with locked project (skips picker)

# Test
bun test                   # Vitest watch mode
bun test:run               # Vitest single run
bun test:e2e               # Playwright E2E tests
bun test:visual            # Visual regression tests
bun test:monkey            # Chaos testing (Gremlins.js)
bun test:all               # Unit + E2E

# Rust
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Frontend (React 19 + Zustand)            │
│  LoopControls │ OutputPanel │ StatusBadge                   │
│                         │                                   │
│                    useLoopStore (Zustand)                   │
└─────────────────────────┬───────────────────────────────────┘
                          │ IPC (invoke + events)
┌─────────────────────────┴───────────────────────────────────┐
│                    Backend (Tauri/Rust)                     │
│  loop_engine.rs   │ State machine, iteration control        │
│  claude_client.rs │ Subprocess execution, JSON streaming    │
│  prompt_builder.rs│ Inline PRD/progress/learnings           │
│  commands.rs      │ Tauri IPC handlers                      │
└─────────────────────────────────────────────────────────────┘
                          │
                          │ subprocess (timeout + claude CLI)
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    Claude CLI                               │
│  --model haiku/opus │ --output-format stream-json           │
│  --dangerously-skip-permissions │ --max-turns 50            │
└─────────────────────────────────────────────────────────────┘
```

## Project Structure

```
ralph4days/
├── .specs/                     # Specifications (meta, testing)
├── src/                        # React frontend
│   ├── components/             # UI components
│   ├── stores/                 # Zustand stores
│   ├── hooks/                  # Custom hooks
│   └── test/                   # Test setup
├── src-tauri/                  # Rust backend
│   └── src/
│       ├── lib.rs              # Entry, command registration
│       ├── commands.rs         # IPC handlers
│       ├── loop_engine.rs      # Loop state machine
│       ├── claude_client.rs    # Claude subprocess
│       ├── prompt_builder.rs   # Prompt construction
│       └── types.rs            # Shared types
├── e2e/                        # Playwright tests
│   ├── controls.spec.ts        # E2E tests
│   ├── visual/                 # Visual regression
│   └── monkey.spec.ts          # Chaos tests
└── vitest.config.ts            # Unit test config
```

## Loop Engine States

```
┌───────┐
│ Idle  │◄────────────────────────────────────┐
└───┬───┘                                     │
    │ start_loop()                            │
    ▼                                         │
┌───────────┐    pause()    ┌────────┐        │
│  Running  │──────────────►│ Paused │        │
└─────┬─────┘               └────┬───┘        │
      │                          │ resume()   │
      │◄─────────────────────────┘            │
      │                                       │
      │ rate limit detected                   │
      ▼                                       │
┌─────────────┐                               │
│ RateLimited │───── retry after 5min ───────►│ (back to Running)
└─────────────┘                               │
      │                                       │
      │ max retries exceeded                  │
      ▼                                       │
┌─────────┐     ┌──────────┐                  │
│ Aborted │     │ Complete │──────────────────┘
└─────────┘     └──────────┘
      ▲               ▲
      │               │
      │ stop() or     │ all tasks done
      │ stagnation    │ (<promise>COMPLETE</promise>)
      └───────────────┘
```

## Target Project Structure

Ralph expects projects to have a `.ralph/` directory:

```
target-project/
├── .ralph/
│   ├── prd.md              # Task list with [ ] / [x] checkboxes (REQUIRED)
│   ├── CLAUDE.RALPH.md     # Ralph-specific context for Claude (RECOMMENDED)
│   ├── progress.txt        # Iteration log (appended after each)
│   └── learnings.txt       # Patterns and gotchas (optional)
├── CLAUDE.md               # Original project context (if exists)
└── ... (project files)
```

**Context File Management**:
- Ralph-specific context uses `CLAUDE.RALPH.md` (note the `.RALPH.` infix)
- When loop starts: backs up existing `CLAUDE.md`, copies `CLAUDE.RALPH.md` to `CLAUDE.md`
- When loop stops: restores original `CLAUDE.md` from backup
- This prevents conflicts with projects that already have `CLAUDE.md`
- See [SPEC-030](./.specs/030_RALPH_PROJECT_STANDARDS.md) for full details

## Project Locking (Session Management)

Ralph locks ONE project per session. User picks project at startup, cannot change during runtime.

### Two Startup Modes

**CLI Argument Mode**:
```bash
ralph --project /path/to/project
```
- Validates and locks project immediately
- If invalid: prints error to stderr, exits with code 1
- If valid: skips ProjectPicker, main UI loads directly

**Interactive Mode**:
```bash
ralph
```
- ProjectPicker modal appears (cannot be dismissed)
- Scans home directory for `.ralph/` folders (5 levels, max 100 projects)
- Dropdown if multiple found, auto-selects if only one
- Manual path input + folder browser
- Real-time validation (debounced 500ms)
- "Lock Project" button enabled when valid

### Backend State

`AppState` in `src-tauri/src/commands.rs`:
```rust
pub struct AppState {
    pub engine: Mutex<LoopEngine>,
    pub locked_project: Mutex<Option<PathBuf>>,
}
```

**Commands**:
- `validate_project_path(path)` - checks path exists, `.ralph/` exists, `prd.md` exists
- `set_locked_project(path)` - validates and locks (one-time operation, errors if already locked)
- `get_locked_project()` - returns `Option<String>` of locked project
- `start_loop(max_iterations)` - reads locked project from state (no path parameter)

### Validation Rules

Path must:
1. Exist and be a directory
2. Contain `.ralph/` subdirectory
3. Contain `.ralph/prd.md` file
4. Be canonicalized (symlinks resolved)

Errors:
- "Directory not found: {path}"
- "Not a directory: {path}"
- "No .ralph folder found. Is this a Ralph project?"
- ".ralph/prd.md not found. Create PRD first."
- "Project already locked for this session"

### Frontend Flow

`src/App.tsx`:
1. Queries `get_locked_project()` on mount
2. If null: renders `<ProjectPicker />`
3. If set: renders main UI with `<LoopControls lockedProject={path} />`

`src/components/LoopControls.tsx`:
- Displays locked project as read-only (path + name/.ralph)
- No project selection UI
- `start_loop` call omits `projectPath` parameter

## Key Implementation Details

### Subprocess Timeout (Critical)
Uses system `timeout` command to avoid Python-style timeout bugs:
```rust
Command::new("timeout")
    .arg(format!("{}s", timeout_secs))  // 900s default
    .arg("claude")
    // ... args
```

### Rate Limit Detection
Parses JSON stream for error types, not log grepping:
```rust
if event.event_type == "error" {
    if err.error_type == "overloaded_error" || err.error_type == "rate_limit_error" {
        // Handle rate limit
    }
}
```

### Inline Prompts
Full file contents embedded in prompt (no @file syntax):
```rust
format!("PRD:\n{prd}\n\nProgress:\n{progress}\n\nLearnings:\n{learnings}")
```

### Stagnation Detection
Compares SHA256 hash of progress.txt + prd.md before/after each iteration. Aborts after 3 consecutive iterations with no change.

## Specifications

| Location | Contains |
|----------|----------|
| `/.specs/` | Meta specs (format, traceability, anti-gaming, testing) |
| `/src/.specs/` | Frontend specs (UI components) — TBD |
| `/src-tauri/.specs/` | Backend specs (loop engine) — TBD |

Read `.specs/000_SPECIFICATION_FORMAT.md` before writing specs.

## Testing Stack

| Category | Tool | Command |
|----------|------|---------|
| **Rust Unit** | cargo test | `cargo test --manifest-path src-tauri/Cargo.toml` |
| **Frontend Unit** | Vitest | `bun test:run` |
| **E2E** | Playwright | `bun test:e2e` |
| **Visual** | Playwright Visual | `bun test:visual` |
| **Chaos** | Gremlins.js | `bun test:monkey` |

See `.specs/060_TESTING_STANDARDS.md` for full testing requirements.

## Tech Stack

- **Frontend:** React 19, TypeScript, Vite, Tailwind v4, Zustand, Lucide Icons
- **Backend:** Tauri 2.5, Rust, Tokio
- **Testing:** Vitest, Playwright, Gremlins.js
- **Build:** bun, Cargo

## Environment Notes

- Claude CLI must be installed and authenticated (`claude --version`)
- Projects must have `.ralph/prd.md` with checkbox tasks
- Loop runs in project directory (working dir = target project)
- Commits happen inside Claude CLI sessions (not managed by Ralph)
