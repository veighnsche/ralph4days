# 062 Internal Engineering Capabilities (Verified From Code)

Date: 2026-02-12
Scope: backend/runtime/tooling architecture capabilities.
Method: claims below are checked against current source code.

Status labels:
- `Implemented`: present and wired
- `Partial`: present with active caveats
- `Broken/Drift`: mismatch between intended and wired behavior
- `Aspirational`: scaffold/placeholder not implemented

## Implemented

### Tauri backend command architecture
- Commands are organized into project/tasks/features/prompts/terminal modules.
- Invoke handler registration is centralized.

Evidence:
- `src-tauri/src/lib.rs`
- `src-tauri/src/commands/*.rs`

### App state and project lock model
- Global app state tracks locked project, DB handle, PTY manager, MCP temp dir, API port.
- Session-level lock flow exists.

Evidence:
- `src-tauri/src/commands/state.rs`
- `src-tauri/src/commands/project.rs`

### SQLite normalized persistence
- Migration-driven normalized schema (metadata/features/disciplines/tasks/signals/comments/recipes).
- Typed DB methods for domain operations.

Evidence:
- `crates/sqlite-db/src/migrations/001_initial.sql`
- `crates/sqlite-db/src/lib.rs`
- `crates/sqlite-db/src/*.rs`

### Prompt/recipe engine
- Prompt type enum + recipe lookup.
- Section execution pipeline and custom section build support.
- Prompt context object model.

Evidence:
- `crates/prompt-builder/src/lib.rs`
- `crates/prompt-builder/src/recipe.rs`
- `crates/prompt-builder/src/recipes/*`
- `crates/prompt-builder/src/sections/*`
- `crates/prompt-builder/src/context.rs`

### MCP generation and task signal server path
- MCP generation supports bash tools mode and signal-server mode.
- Task execution recipe uses signal-server mode.
- Generated config points task sessions to `task_signals_server.ts`.

Evidence:
- `crates/prompt-builder/src/mcp/mod.rs`
- `crates/prompt-builder/src/recipes/task_exec.rs`
- `crates/prompt-builder/src/mcp/task_signals_server.ts`

### Local API server for signal ingestion
- Axum server started in app setup.
- Endpoints:
  - `POST /api/set-db-path`
  - `POST /api/task-signal`
- Maps verbs to typed signal insert methods.

Evidence:
- `src-tauri/src/api_server.rs`
- `crates/sqlite-db/src/signals.rs`

### PTY runtime and event bridge
- PTY session manager uses `portable-pty`.
- Spawns and supervises `claude` subprocesses.
- Emits output/closed events to frontend.

Evidence:
- `src-tauri/src/terminal/manager.rs`
- `src-tauri/src/terminal/session.rs`
- `src-tauri/src/terminal/events.rs`

### Type contract export pipeline
- `#[ipc_type]` macro for ts-rs export behavior.
- Generated TS wire types present.

Evidence:
- `crates/ralph-macros/src/lib.rs`
- `src/types/generated.ts`
- `justfile` (`types` target)

### Test/dev tooling surfaces
- Vitest unit setup.
- Automation runner e2e/visual harness.
- Storybook support.
- Dev bridge tooling (`mcp-dev-server.ts` + browser bridge).

Evidence:
- `vitest.config.ts`
- `automation-runner.config.ts`
- `src/test/setup.ts`
- `mcp-dev-server.ts`
- `src/lib/dev-bridge.ts`

## Partial

### Feature comment embedding pipeline
- End-to-end embedding write path exists.
- Runtime behavior depends on external service configuration availability.

Evidence:
- `src-tauri/src/commands/features.rs`
- `crates/ralph-external/src/comment_embeddings.rs`
- `crates/ralph-external/src/config.rs`

### Predefined discipline assets/toolchain
- Stack metadata and discipline presets are wired.
- Image-generation helper scripts/binaries exist, but asset generation workflow is environment-dependent and iterative.

Evidence:
- `crates/predefined-disciplines/src/lib.rs`
- `crates/predefined-disciplines/src/bin/generate_discipline_image.rs`
- `crates/predefined-disciplines/*.py`

## Broken/Drift

### Task signal frontend/backend command drift
- Frontend mutation hook command names do not match invoke-exposed backend command names.
- This creates integration risk for signal comment edits/creates/replies.

Evidence:
- `src/hooks/tasks/useSignalMutations.ts`
- `src-tauri/src/lib.rs`
- `src-tauri/src/commands/tasks.rs`

## Aspirational

### Task execution orchestration backend
- Execution command API surface currently exists under execution-sequence names (`start_execution_sequence`, `pause_execution_sequence`, etc.) as placeholders.
- These commands currently return `Not implemented`.
- Intended model (per product direction): orchestrate task execution in parallel/ordered progression until no runnable tasks remain, with support for tasks creating new tasks.

Evidence:
- `src-tauri/src/commands/project.rs`
