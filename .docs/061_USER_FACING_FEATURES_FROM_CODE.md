# 061 User-Facing Features (Verified From Code)

Date: 2026-02-12
Scope: End-user visible behavior only.
Method: claims below are checked against current source code.

Status labels:
- `Implemented`: exists and is wired in current code
- `Partial`: exists but has TODOs, limitations, or incomplete wiring
- `Broken/Drift`: code paths exist but current wiring appears inconsistent
- `Aspirational`: explicitly present as TODO/placeholder/not implemented

## Implemented

### Desktop shell and navigation
- Native desktop app (Tauri), splash + main window startup flow.
- Split layout: page area (left) + workspace tabs (right).
- Window title updates to selected project name.
- App menu supports page switching and opening a new window.
- Settings dialog supports dark/light theme toggle.

Evidence:
- `src-tauri/src/lib.rs`
- `src/App.tsx`
- `src/components/app-shell/NavigationMenu.tsx`
- `src/components/app-shell/Settings.tsx`

### Project selection and initialization
- Open recent projects.
- Discover projects by scanning filesystem for `.ralph/`.
- Validate project directories.
- Initialize a new Ralph project with stack preset.

Evidence:
- `src/components/app-shell/ProjectSelector.tsx`
- `src-tauri/src/commands/project.rs`
- `src-tauri/src/recent_projects.rs`

### Core pages
- Tasks page, Features page, Disciplines page.
- Page registry and menu navigation are wired.

Evidence:
- `src/pages/pageRegistry.ts`
- `src/pages/TasksPage.tsx`
- `src/pages/FeaturesPage.tsx`
- `src/pages/DisciplinesPage.tsx`

### Tasks (major user flows)
- Task listing and progress UI.
- Task creation form.
- Task detail tabs.
- Status update command usage from task sidebar (`set_task_status`).
- Execute Task button opens task-bound terminal tab.

Evidence:
- `src/components/workspace/TaskFormTabContent.tsx`
- `src/components/workspace/TaskDetailTabContent.tsx`
- `src/components/workspace/task-detail/TaskSidebar.tsx`
- `src-tauri/src/commands/tasks.rs`

### Features (major user flows)
- Feature list and progress UI.
- Feature creation form.
- Feature detail tabs.
- Feature comments add/edit/delete flows in UI hooks and backend commands.

Evidence:
- `src/pages/FeaturesPage.tsx`
- `src/components/workspace/FeatureFormTabContent.tsx`
- `src/components/workspace/FeatureDetailTabContent.tsx`
- `src/hooks/features/useFeatureCommentMutations.ts`
- `src-tauri/src/commands/features.rs`

### Disciplines (major user flows)
- Discipline roster/grid with stats.
- Discipline detail tabs.
- Discipline create/update/delete flows.
- Discipline forms include prompt, skills, conventions, MCP server JSON.
- Discipline image crop rendering in list/detail when image data exists.

Evidence:
- `src/pages/DisciplinesPage.tsx`
- `src/components/forms/DisciplineForm.tsx`
- `src/components/workspace/DisciplineDetailTabContent.tsx`
- `src/hooks/disciplines/useDisciplineMutations.ts`

### Workspace tabs
- Open/switch/close tabs.
- Close others, close to right, close all.
- Drag-and-drop tab reorder.
- New tab to right.

Evidence:
- `src/stores/useWorkspaceStore.ts`
- `src/components/workspace/BrowserTabs.tsx`
- `src/hooks/workspace/useBrowserTabsActions.ts`

### Claude terminal UX
- Open terminal tabs, stream output, send input, resize PTY.
- Model picker (haiku/sonnet/opus) and thinking toggle exposed in UI.

Evidence:
- `src/components/workspace/TerminalTabContent.tsx`
- `src/lib/terminal/session.ts`
- `src/components/model-thinking/ModelThinkingPicker.tsx`
- `src-tauri/src/commands/terminal.rs`

## Partial

### Braindump flow
- Braindump tab exists and sends text to a new terminal session.
- File contains explicit TODOs for MCP generation, DB sync, and lifecycle handling.

Evidence:
- `src/components/workspace/BraindumpFormTabContent.tsx`

### Prompt Builder UX
- Modal, section toggles/reorder, preview, save/load/delete recipe config are implemented.
- Completeness depends on backend section coverage and recipe assumptions; still an actively evolving area.

Evidence:
- `src/components/prompt-builder/PromptBuilderModal.tsx`
- `src/hooks/prompt-builder/*`
- `src-tauri/src/commands/prompts.rs`

## Broken/Drift

### Task signal mutation command names
- Frontend hook calls `add_task_signal`, `update_task_signal`, `delete_task_signal`, `add_reply_to_signal`.
- Backend exposes `add_task_comment`, `update_task_comment`, `delete_task_comment`, `add_reply_to_comment`.
- This is a direct command-name mismatch in current code.

Evidence:
- `src/hooks/tasks/useSignalMutations.ts`
- `src-tauri/src/lib.rs`
- `src-tauri/src/commands/tasks.rs`

## Aspirational

### Task execution orchestration controls
- Task execution controls are intentionally disabled in UI for now.
- The backend currently exposes execution-sequence command names (`start_execution_sequence`, `pause_execution_sequence`, etc.), but these are placeholders and return `Not implemented`.
- Intended behavior (per project direction): execute tasks in ordered/parallel orchestration until no runnable tasks remain, while allowing tasks to generate new tasks.

Evidence:
- `src/components/app-shell/BottomBar.tsx`
- `src/components/app-shell/ExecutionToggle.tsx`
- `src-tauri/src/commands/project.rs`
