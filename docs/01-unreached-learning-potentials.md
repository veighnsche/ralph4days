# Unreached Learning Potentials

1. Expand test strategy from core libs into UI workflows.
- Evidence: `197` source files, `5` unit test files, `3` e2e specs, and `66` passing tests.
- Why this matters: most user-visible regressions happen in component interaction flows, not utility layers.
- Learning move: add behavior tests for high-interaction components before adding new features.

2. Add direct tests for high-complexity, high-impact components.
- Evidence: large untested components include:
  - `src/components/prompt-builder/PromptBuilderModal.tsx`
  - `src/components/workspace/feature-detail/FeatureCommentsSection.tsx`
  - `src/components/workspace/task-detail/TaskSidebar.tsx`
  - `src/components/workspace/task-detail/SignalSection.tsx`
  - `src/components/workspace/FeatureDetailTabContent.tsx`
  - `src/components/workspace/BraindumpFormTabContent.tsx`
- Why this matters: these files combine orchestration + UI branching and are likely defect hotspots.
- Learning move: require at least one happy-path and one failure-path test per hotspot component.

3. Treat workspace store behavior as a first-class contract.
- Evidence: `src/stores/useWorkspaceStore.ts` is central to navigation/tab state and has very low coverage (~1.33% lines in report).
- Why this matters: tab ordering, focus, and close semantics are foundational and easy to break.
- Learning move: add deterministic state-transition tests for open, close, close-all-except, close-to-right, and reorder edge cases.

4. Add coverage thresholds to stop silent quality drift.
- Evidence: `vitest.config.ts` has coverage enabled in runs but no threshold gates configured.
- Why this matters: coverage can trend down while CI stays green.
- Learning move: add minimum thresholds and ratchet upward over time.

5. Convert TODO clusters into explicit implementation milestones.
- Evidence: `src/components/workspace/BraindumpFormTabContent.tsx:1` has a concentrated TODO block for MCP wiring, lifecycle, invalidation, and progress UX.
- Why this matters: TODO clusters can stay invisible debt unless tied to scoped delivery criteria.
- Learning move: convert TODO set into a tracked checklist with acceptance tests.

6. Remove prototype timing assumptions from production flows.
- Evidence: `src/components/workspace/BraindumpFormTabContent.tsx:49` uses fixed `setTimeout(1000)` before sending terminal input.
- Why this matters: fixed delays are brittle across machine speed and race conditions.
- Learning move: replace timer with readiness/event-based handshake and test for terminal-close race.

7. Centralize and type the Tauri command boundary.
- Evidence: direct `invoke(...)` calls are distributed across UI files such as:
  - `src/App.tsx:40`
  - `src/components/app-shell/ProjectSelector.tsx:147`
  - `src/components/app-shell/ProjectSelector.tsx:148`
  - `src/components/app-shell/ProjectSelector.tsx:161`
  - `src/components/workspace/BraindumpFormTabContent.tsx:55`
- Why this matters: command surface drift and inconsistent error handling become harder to control.
- Learning move: create a typed `src/lib/tauri/commands.ts` facade and route all `invoke` calls through it.

8. Improve determinism of generated IDs for state reproducibility.
- Evidence: `src/stores/useWorkspaceStore.ts:58` uses `Date.now()` fallback IDs.
- Why this matters: replay/debug/test determinism improves when IDs are stable or injectable.
- Learning move: use a monotonic store counter or injectable id generator in store creation.

9. Grow e2e coverage around critical user journeys.
- Evidence: current e2e set is `e2e/controls.spec.ts`, `e2e/monkey.spec.ts`, `e2e/visual/states.spec.ts` with Chromium-only config.
- Why this matters: orchestration-heavy desktop flows need explicit journey tests (project init, tab workflows, terminal lifecycle).
- Learning move: add scenario e2e specs focused on full workflow outcomes and failure recovery.

10. Formalize orchestration quality gates for AI-produced code.
- Evidence: code quality is strong, but guardrails are mostly implicit in current process.
- Why this matters: consistency under multi-agent contribution scales when expectations are executable.
- Learning move: define a lightweight PR checklist (test added, risk notes, touched commands typed, no unresolved TODOs in changed files).
