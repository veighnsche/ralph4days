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
pnpm dev                    # Vite dev server (port 1420)
pnpm tauri dev              # Full Tauri app with hot reload

# Build
pnpm build                  # Frontend production build
pnpm tauri build            # Full production app (release optimized)
pnpm tauri build --debug    # Debug build

# Test
pnpm test                   # Vitest watch mode
pnpm test:run               # Vitest single run
pnpm test:e2e               # Playwright E2E tests
pnpm test:visual            # Visual regression tests
pnpm test:monkey            # Chaos testing (Gremlins.js)
pnpm test:all               # Unit + E2E

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
│   ├── prd.md           # Task list with [ ] / [x] checkboxes (REQUIRED)
│   ├── progress.txt     # Iteration log (appended after each)
│   ├── learnings.txt    # Patterns and gotchas (optional)
│   └── CLAUDE.md        # Context injected into Claude (optional)
└── ... (project files)
```

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
| **Frontend Unit** | Vitest | `pnpm test:run` |
| **E2E** | Playwright | `pnpm test:e2e` |
| **Visual** | Playwright Visual | `pnpm test:visual` |
| **Chaos** | Gremlins.js | `pnpm test:monkey` |

See `.specs/060_TESTING_STANDARDS.md` for full testing requirements.

## Tech Stack

- **Frontend:** React 19, TypeScript, Vite, Tailwind v4, Zustand, Lucide Icons
- **Backend:** Tauri 2.5, Rust, Tokio
- **Testing:** Vitest, Playwright, Gremlins.js
- **Build:** pnpm, Cargo

## Environment Notes

- Claude CLI must be installed and authenticated (`claude --version`)
- Projects must have `.ralph/prd.md` with checkbox tasks
- Loop runs in project directory (working dir = target project)
- Commits happen inside Claude CLI sessions (not managed by Ralph)
