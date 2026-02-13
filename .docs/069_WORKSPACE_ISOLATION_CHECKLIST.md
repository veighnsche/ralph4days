# Workspace Isolation Checklist

Date: 2026-02-13
Owner: Frontend workspace infrastructure

## Goal
- Treat workspace as a standalone embedded browser runtime.
- Workspace should own tab lifecycle, mount policy, and resource scheduling.
- Non-workspace app areas should not drive workspace render/effect churn.

## Current Status Snapshot
- [x] Workspace has its own tab registry/module system.
- [x] Workspace has explicit tab lifecycle hooks: `onMount`, `onUnmount`, `onActivate`, `onDeactivate`.
- [x] Workspace has its own tab store (`useWorkspaceStore`).
- [x] Workspace mount policy is centralized and active-tab-first.
- [x] Keep-alive policy is declared per tab type (`keepAliveOnDeactivate`).
- [x] Terminal tab is explicitly keep-alive.
- [x] Tab active state is available via context (`useWorkspaceTabIsActive`).
- [x] Active/inactive gating is enforced for terminal keep-alive runtime work.
- [x] Workspace query/cache domain is isolated from non-workspace surfaces.
- [x] Workspace kernel contract exists as a single orchestration boundary.
- [x] Left-side app pages are mounted-on-demand (not all mounted and hidden).

## Verified Bugs And Gaps (Audit 2026-02-13)
- [x] `P0` Lifecycle contract drift for non-keepalive tabs fixed.
  - Lifecycle dispatch now tracks mounted-tab transitions and active-tab transitions in one kernel pass.
  - References: `src/components/workspace/kernel.ts`, `src/components/workspace/WorkspacePanel.tsx`.
- [x] `P0` Stale `activeTabId` after `closeToRight` fixed.
  - Store now guarantees `activeTabId` always points to an existing tab (or empty when no tabs).
  - References: `src/stores/useWorkspaceStore.ts`, `src/stores/useWorkspaceStore.test.ts`.
- [x] `P0` Additional active-id invariants fixed.
  - `switchTab` now no-ops on unknown tab ids; `closeAllExcept` normalizes active tab on unknown target.
  - Reference: `src/stores/useWorkspaceStore.ts`.
- [x] `P1` Keep-alive runtime gating wired for terminal.
  - Terminal session runtime uses active/inactive stream mode with backend buffered replay.
  - References: `src/components/workspace/tabs/terminal/content.tsx`, `src/lib/terminal/session.ts`, `src-tauri/src/terminal/manager.rs`.
- [x] `P1` Mount policy + lifecycle transition coverage added.
  - References: `src/components/workspace/kernel.test.ts`, `src/stores/useWorkspaceStore.test.ts`.
- [x] `P2` App-side hidden mounts removed.
  - Left panel now renders only the current page component.
  - Reference: `src/App.tsx`.
- [x] `P2` Non-keyed tab id collision risk addressed.
  - Store now uses UUID-backed ids for non-keyed tabs.
  - Reference: `src/stores/useWorkspaceStore.ts`.

## Isolation Requirements Checklist
- [x] Define workspace kernel responsibilities in code/docs:
  - tab scheduling
  - lifecycle dispatch
  - mount/unmount policy
  - keep-alive policy
  - background work budget
- [x] Enforce module-level tab policy ownership:
  - tab type declares `keepAliveOnDeactivate`
  - workspace panel is the only place that decides mount strategy
- [x] Require active-state gating for background work:
  - terminal output stream switches to buffered mode on deactivate and replays on activate.
  - workspace queries are namespaced to a workspace cache domain.
- [x] Ensure deactivation does not terminate sessions unless policy says so.
- [x] Add tests for lifecycle + mount policy behavior:
  - active tab mounted
  - non-keepalive tab unmounted on deactivate
  - keepalive tab remains mounted but marked inactive
  - hooks fire in expected order on tab switches
- [ ] Add instrumentation (follow-up):
  - workspace-level render count sampling
  - hidden-tab active effects warning logs in dev

## Anti-Patterns To Block
- [x] Hidden-but-mounted tab trees by default for all tab types.
- [ ] Tab content directly deciding global mount policy.
- [x] Background polling/listeners continuing for inactive tabs without explicit need.
- [x] Cross-surface coupling where non-workspace pages force workspace recomputation.

## Immediate Next Steps
- [ ] Add dev instrumentation for hidden tab workload and workspace render sampling.
- [ ] Evaluate whether any non-terminal keepalive modules need explicit inactive budgets.
- [ ] Add a focused integration test for terminal replay truncation user messaging.
