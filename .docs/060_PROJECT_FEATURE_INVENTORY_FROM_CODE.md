# 060 Project Feature Inventory (From Code)

Date: 2026-02-11
Scope: `ralph4days` feature inventory derived from source code paths, not historical design docs.

## 1. Desktop Runtime and App Shell

- Tauri desktop app with split-pane workspace (left app pages + right tabbed workspace).
- Splash window + main window startup flow.
- Dynamic window title set to current project name.
- New window spawning from the app menu.
- Dark/light theme toggle in Settings (stored in localStorage).

Primary code:
- `src-tauri/src/lib.rs`
- `src/App.tsx`
- `src/components/app-shell/BottomBar.tsx`
- `src/components/app-shell/NavigationMenu.tsx`
- `src/components/app-shell/Settings.tsx`

## 2. Project Onboarding and Selection

- Scan filesystem for projects containing `.ralph/`.
- Recent projects list stored in XDG data dir.
- Validate project path (`.ralph/db/ralph.db` checks).
- Project lock for current desktop session.
- Initialize new Ralph project with selected stack preset.
- Project initialization creates:
  - `.ralph/` and sqlite DB
  - seeded disciplines for selected stack
  - `CLAUDE.RALPH.md` template

Primary code:
- `src/components/app-shell/ProjectSelector.tsx`
- `src-tauri/src/commands/project.rs`
- `src-tauri/src/recent_projects.rs`
- `src-tauri/src/xdg.rs`

## 3. Page Model (Tasks / Features / Disciplines)

- Three main pages with menu switching:
  - Tasks
  - Features
  - Disciplines
- Shared page layout and statistics/progress presentation.

Primary code:
- `src/pages/pageRegistry.ts`
- `src/pages/TasksPage.tsx`
- `src/pages/FeaturesPage.tsx`
- `src/pages/DisciplinesPage.tsx`

## 4. Tasks: Planning, Tracking, and Detail

- Task list loading from backend.
- Task filtering and progress bars.
- Create task form (feature, discipline, title, description, priority, tags, acceptance criteria, dependencies).
- Task status updates (`draft`, `pending`, `in_progress`, `done`, `blocked`, `skipped`).
- Task detail tab with discipline imagery and inferred status context.
- Task signal summaries (pending asks, flags, session counts, etc.)

Primary code:
- `src/components/workspace/TaskFormTabContent.tsx`
- `src/components/workspace/TaskDetailTabContent.tsx`
- `src/hooks/tasks/usePRDData.ts`
- `src/hooks/tasks/usePRDFilters.ts`
- `src/hooks/tasks/useSignalSummaries.ts`
- `src-tauri/src/commands/tasks.rs`
- `crates/sqlite-db/src/tasks.rs`
- `crates/sqlite-db/src/types.rs`

## 5. Task Signals and Reply Threading

- Signal/comment UI on tasks.
- Structured signal verbs represented in data model:
  - `signal`, `done`, `partial`, `stuck`, `ask`, `flag`, `learned`, `suggest`, `blocked`
- Reply support to top-level signals (2-layer depth).
- Signal payload rendering by verb.

Primary code:
- `src/components/workspace/task-detail/SignalSection.tsx`
- `src/components/workspace/task-detail/SignalPayloadDisplay.tsx`
- `src/components/workspace/task-detail/ReplyForm.tsx`
- `src-tauri/src/api_server.rs`
- `crates/sqlite-db/src/signals.rs`
- `crates/sqlite-db/src/migrations/001_initial.sql`

## 6. Features: CRUD + Knowledge Comments

- Feature list and progress view.
- Create/update/delete features.
- Feature detail tabs.
- Feature comments with categories and metadata.
- Embedding generation for feature comments (Ollama-backed via `ralph-external`).
- Comment embedding storage and update pipeline.

Primary code:
- `src/pages/FeaturesPage.tsx`
- `src/components/workspace/FeatureDetailTabContent.tsx`
- `src/hooks/features/useFeatureStats.ts`
- `src/hooks/features/useFeatureCommentMutations.ts`
- `src-tauri/src/commands/features.rs`
- `crates/sqlite-db/src/features.rs`
- `crates/sqlite-db/src/feature_comments.rs`
- `crates/sqlite-db/src/comment_embeddings.rs`
- `crates/ralph-external/src/comment_embeddings.rs`

## 7. Disciplines: Stack Presets, CRUD, Images

- Discipline list with card visuals and task stats.
- Create/update/delete custom disciplines.
- Discipline fields include:
  - identity (name/display/acronym/icon/color)
  - system prompt
  - skills
  - conventions
  - MCP server configs
- Stack metadata retrieval.
- Discipline portraits and cached crop generation for UI slots (`face`, `card`, `strip`, etc.).

Primary code:
- `src/pages/DisciplinesPage.tsx`
- `src/components/workspace/DisciplineDetailTabContent.tsx`
- `src/hooks/disciplines/useDisciplines.ts`
- `src/hooks/disciplines/useDisciplineMutations.ts`
- `src-tauri/src/commands/features.rs`
- `crates/predefined-disciplines/src/lib.rs`

## 8. Workspace Tabs and Browser-like Tab UX

- Multi-tab workspace for terminals, forms, and detail pages.
- Open/switch/close tab operations.
- Close all/close others/close right context actions.
- Drag reorder tabs.
- New tab to the right.
- Enforced max tab count.

Primary code:
- `src/stores/useWorkspaceStore.ts`
- `src/components/workspace/BrowserTabs.tsx`
- `src/hooks/workspace/useBrowserTabsActions.ts`
- `src/hooks/workspace/useWorkspaceActions.ts`

## 9. Integrated Claude Terminal Sessions

- PTY-backed terminal tabs via xterm.js.
- Spawns `claude` CLI in project working directory.
- Session lifecycle:
  - create
  - send input
  - resize
  - terminate
  - stream output events to UI
- Model + thinking preferences per launch (haiku/sonnet/opus and extended thinking).
- Per-task terminal session path available (task-bound MCP config).

Primary code:
- `src/components/workspace/TerminalTabContent.tsx`
- `src/lib/terminal/session.ts`
- `src-tauri/src/commands/terminal.rs`
- `src-tauri/src/terminal/manager.rs`
- `src-tauri/src/terminal/session.rs`

## 10. Prompt Builder / Recipe System

- Prompt Builder modal for section-level prompt composition.
- Drag-and-drop section ordering.
- Enable/disable sections.
- Per-section instruction overrides.
- Prompt preview generation.
- Copy prompt to clipboard.
- Save/list/load/delete custom recipe configs in sqlite.

Primary code:
- `src/components/prompt-builder/PromptBuilderModal.tsx`
- `src/hooks/prompt-builder/useSectionConfiguration.ts`
- `src/hooks/prompt-builder/usePromptPreview.ts`
- `src/hooks/prompt-builder/useRecipeManagement.ts`
- `src/lib/recipe-registry.ts`
- `src-tauri/src/commands/prompts.rs`
- `crates/prompt-builder/src/recipes/*`
- `crates/sqlite-db/src/recipe_configs.rs`

## 11. MCP Integration Modes

- MCP generation for interactive/task creation modes (bash tools mode).
- MCP generation for task execution mode (signal server mode).
- Task signal MCP server (`task_signals_server.ts`) forwards verbs to local API server.
- Discipline-specific MCP servers can be included for task execution contexts.

Primary code:
- `src-tauri/src/commands/state.rs`
- `crates/prompt-builder/src/mcp/mod.rs`
- `crates/prompt-builder/src/mcp/tools.rs`
- `crates/prompt-builder/src/mcp/task_signals_server.ts`
- `crates/prompt-builder/src/recipes/task_exec.rs`

## 12. Local API Server for Signal Ingestion

- Axum server launched on random localhost port.
- Endpoints:
  - `POST /api/set-db-path`
  - `POST /api/task-signal`
- Writes typed signal records into sqlite.
- Emits frontend event after insert.

Primary code:
- `src-tauri/src/api_server.rs`

## 13. Database and Type-Safe IPC Surface

- SQLite migrations with normalized schema and constraints.
- Rust typed DB API for tasks/features/disciplines/signals/recipes.
- IPC types exported to TypeScript via `ts-rs` + custom `#[ipc_type]` macro.

Primary code:
- `crates/sqlite-db/src/migrations/001_initial.sql`
- `crates/sqlite-db/src/lib.rs`
- `crates/sqlite-db/src/types.rs`
- `crates/ralph-macros/src/lib.rs`
- `src/types/generated.ts`

## 14. Test and Dev Infrastructure

- Frontend unit tests (Vitest + jsdom).
- Automation runner e2e and visual suites.
- Storybook support.
- Dev bridge / MCP dev server tooling for automating Tauri interactions in development.

Primary code:
- `vitest.config.ts`
- `automation-runner.config.ts`
- `e2e/*.spec.ts`
- `src/test/setup.ts`
- `mcp-dev-server.ts`
- `src/lib/dev-bridge.ts`

## 15. Current Gaps / Partially Implemented Features

- Task execution orchestration commands currently exist under execution-sequence naming and return not implemented:
  - `start_execution_sequence`, `pause_execution_sequence`, `resume_execution_sequence`, `stop_execution_sequence`, `get_execution_sequence_state`
- Bottom-bar task execution controls are intentionally disabled pending execution integration.
- Signal mutation hook currently references command names that do not match current backend command names (`add_task_signal` vs `add_task_comment` etc.), indicating wiring drift to resolve.

Primary code:
- `src-tauri/src/commands/project.rs`
- `src/components/app-shell/BottomBar.tsx`
- `src/components/app-shell/ExecutionToggle.tsx`
- `src/hooks/tasks/useSignalMutations.ts`
- `src-tauri/src/commands/tasks.rs`
