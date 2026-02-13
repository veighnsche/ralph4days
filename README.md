# Ralph4days

Desktop workspace for orchestrating Claude-driven software execution on top of a project-local task graph.

## Current Status (February 12, 2026)

Ralph4days is in active development. The app already provides a strong interactive workflow (project setup, task/feature/discipline management, terminal sessions, prompt builder), while autonomous task-sequence orchestration is still being wired.

## What Works Today

- Tauri desktop app with split layout:
  - left: Tasks / Features / Disciplines pages
  - right: tabbed workspace (terminal, detail views, forms)
- Project picker:
  - discover existing `.ralph` projects
  - open recent projects
  - initialize a new `.ralph` project with stack presets
- SQLite-backed local data model for:
  - tasks
  - features
  - disciplines
  - signals/comments
  - recipe configs
- Task workflow:
  - create tasks
  - open task detail
  - update task status
  - run task-bound terminal session
- Feature workflow:
  - create/update/delete features
  - feature comments with embedding pipeline support
- Discipline workflow:
  - create/update/delete disciplines
  - prompt/skills/conventions/MCP server config fields
  - stack metadata and discipline image crop rendering
- Workspace tabs:
  - open/switch/close
  - close others / close right / close all
  - drag-reorder
- Integrated Claude terminal tabs via PTY
  - model selection (haiku/sonnet/opus)
  - thinking toggle
- Prompt Builder:
  - reorder/toggle sections
  - override instructions
  - preview and save recipe configs

## In Progress

- Autonomous task execution sequence controls are not yet wired end-to-end in UI.
- Backend execution-sequence commands currently exist as placeholders.
- Some signal mutation command wiring is being reconciled between frontend and backend names.

## Architecture

### Frontend

- React 19 + TypeScript + Vite
- TanStack Query for data fetching
- Zustand for workspace/tab state
- Tailwind CSS + component library in `src/components/ui`

### Backend

- Tauri 2 app runtime
- Rust command modules for project/tasks/features/prompts/terminal
- Local Axum API server for task-signal ingestion
- PTY process management for Claude CLI sessions

### Data + Core Crates

- `crates/sqlite-db`: normalized schema + typed DB API
- `crates/prompt-builder`: recipe/section prompt assembly + MCP config generation
- `crates/predefined-disciplines`: stack presets + discipline assets/metadata
- `crates/ralph-external`: external integrations (including embeddings)

## Prerequisites

- Rust 1.75+
- Bun
- Claude CLI available on PATH (`claude`)
- Linux desktop dependencies required by Tauri/WebKitGTK

## Development

```bash
# install deps
bun install

# run app in dev mode
just dev

# run with mock fixture
just reset-mock
just dev-mock 03
# optional override for location (default: /tmp/ralph4days-mock)
RALPH_MOCK_DIR=/path/outside/repo just reset-mock

# checks
just check
just lint
just fmt

# tests
just test
just test-rust
just test-frontend
```

## Build

```bash
# release desktop build
just build

# linux bundles (.deb/.rpm/.appimage)
just release-linux
```

## Project Layout

```text
src/                 React frontend
src-tauri/           Tauri app + Rust backend commands
crates/              Workspace crates (sqlite-db, prompt-builder, etc.)
fixtures/            Read-only fixture projects
.docs/               Internal docs and code-truth inventories
mock data            Disposable fixture copies in /tmp/ralph4days-mock (or $RALPH_MOCK_DIR)
justfile             Task runner commands
```

## Docs Freshness

For current doc freshness status, see:

- `.docs/DOC_FRESHNESS_STATUS.md`

## License

MIT
