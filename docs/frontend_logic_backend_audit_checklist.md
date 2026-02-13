# Frontend -> Backend Boundary Audit Checklist

Scope: frontend logic outside `src/components/ui/**` that should be backend-owned or backend-authoritative.

## Prompt Builder Domain Ownership

- [ ] Move canonical prompt-builder section metadata out of frontend.
  - Current frontend source: `src/lib/prompt-builder-registry.ts`
  - Current backend source: `crates/prompt-builder/src/sections/metadata.rs`
  - Reason: domain metadata exists in two places and can drift.

- [ ] Move canonical prompt recipe definitions out of frontend.
  - Current frontend source: `src/lib/prompt-builder-registry.ts`
  - Current backend source: `crates/prompt-builder/src/recipes/mod.rs`
  - Reason: recipe ordering/availability is backend domain behavior, not UI behavior.

- [ ] Eliminate duplicated default instruction text in frontend.
  - Current frontend source: `src/lib/prompt-builder-registry.ts`
  - Current backend source: `crates/prompt-builder/src/sections/instructions/*.rs`
  - Reason: instruction content is product policy and should have one source of truth.

- [ ] Fix already-observed drift: frontend missing enrichment instruction surface.
  - Backend has: `enrichment_instructions` in `crates/prompt-builder/src/sections/metadata.rs`
  - Frontend does not expose equivalent metadata in `src/lib/prompt-builder-registry.ts`
  - Reason: confirms duplication has already created behavioral mismatch.

## Prompt Preview Assembly

- [ ] Move full prompt preview composition to backend (including user input insertion).
  - Current frontend behavior: `src/hooks/prompt-builder/usePromptPreview.ts`
  - Current backend entrypoint: `src-tauri/src/commands/prompts.rs` (`preview_custom_prompt_builder`)
  - Reason: prompt assembly/order should be authoritative in one place.

- [ ] Stop client-side section splicing for `user_input`.
  - Current frontend behavior: `rebuildPreviewWithUserInput` in `src/hooks/prompt-builder/usePromptPreview.ts`
  - Expected: backend returns final `sections` + `fullPrompt` exactly as used.
  - Reason: prevents preview/runtime divergence.

## Naming and Canonicalization Invariants

- [ ] Move feature-name normalization from frontend-only transform to backend validation/normalization.
  - Current frontend transform: `src/lib/acronym.ts` + `src/lib/schemas/featureSchema.ts` + `src/lib/schemas/taskSchema.ts`
  - Current backend create/update pass-through: `src-tauri/src/commands/features.rs`
  - Reason: invariants must hold for all clients, not only this UI.

- [ ] Define and enforce one canonical naming rule in backend for feature references from task create/update.
  - Related frontend assumption: task form sends normalized feature names.
  - Related backend logic: `crates/sqlite-db/src/tasks.rs` resolves by exact `name`.
  - Reason: avoid hidden coupling to frontend transform behavior.

## Session Launch Resolution Policy

- [ ] Move launch precedence policy (task -> discipline -> user default) to backend.
  - Current frontend resolution: `src/components/workspace/task-detail/hooks/useResolvedTaskLaunch.ts`
  - Backend already validates model/effort compatibility: `src-tauri/src/terminal/providers/mod.rs`
  - Reason: precedence is domain policy, not view logic.

- [ ] Move fallback model/effort coercion policy to backend command layer.
  - Current frontend fallback: `src/components/agent-session-launch/resolveLaunchConfig.ts`
  - Related backend launch: `src-tauri/src/commands/terminal_bridge.rs`
  - Reason: keep launch behavior deterministic across all invocation surfaces.

- [ ] Keep frontend as display-only for launch-source labels.
  - Current frontend labels: `LaunchSource` in `src/components/workspace/task-detail/hooks/useResolvedTaskLaunch.ts`
  - Expected: backend returns resolved config and source metadata.
  - Reason: UI can still explain provenance without owning policy.

## Stack Catalog Source of Truth

- [ ] Remove hardcoded stack list from project selector.
  - Current frontend hardcoded list: `src/components/app-shell/ProjectSelector.tsx`
  - Backend provider: `get_stack_metadata` in `src-tauri/src/commands/features.rs`
  - Reason: stack definitions are backend/domain data and should not be duplicated.

## Nice-to-Have Boundary Tightening

- [ ] Consider backend-provided stats payloads for feature/discipline/project progress if these become shared product semantics.
  - Current frontend derived stats: `src/lib/stats.ts`, `src/hooks/features/useFeatureStats.ts`, `src/hooks/disciplines/useDisciplineStats.ts`
  - Existing backend types indicate domain intent: `crates/sqlite-db/src/types.rs` (`GroupStats`, `ProjectProgress`)
  - Reason: not strictly required today, but useful if non-UI consumers need identical metrics.

## Acceptance Checklist

- [ ] No canonical prompt-builder metadata or instruction text is duplicated in frontend.
- [ ] Prompt preview returned from backend is final and not reassembled client-side.
- [ ] Feature/task naming invariants are backend-enforced and documented.
- [ ] Launch resolution policy is backend-owned; frontend only displays and invokes.
- [ ] Project stack options are fetched from backend metadata, not hardcoded.
- [ ] Existing behavior is covered by tests at command/provider layer for moved logic.
