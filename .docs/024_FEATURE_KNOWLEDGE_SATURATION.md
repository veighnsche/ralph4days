# 024 — Feature Knowledge Saturation & Auto-Split Signal

**Date:** 2026-02-07
**Status:** Design (loop engine stub — implementation blocked until post-iteration hooks exist)

## Problem

Each feature caps at 50 learnings (`max_learnings_per_feature` in `RagConfig`). As agents work on a feature, learnings accumulate. Once all 50 slots are filled with protected learnings (reviewed by Opus, written by humans, or re-observed 3+ times), `select_for_pruning` returns an empty vec — new knowledge literally cannot be stored. The feature's institutional memory is full and new learnings are silently lost.

This is a natural signal that the feature scope has grown too large and should be decomposed.

## Design: Saturation Metric

A per-feature metric computed from its learnings array:

| Field | Type | Description |
|-------|------|-------------|
| `learnings_count` | usize | Current number of learnings |
| `max_learnings` | usize | Cap (default 50) |
| `protected_count` | usize | Learnings that can't be auto-pruned (`!is_auto_prunable()`) |
| `prunable_count` | usize | Learnings that can be auto-pruned |
| `saturation_pct` | u32 | `(learnings_count * 100) / max_learnings` |
| `level` | enum | Low (0-59%), Medium (60-79%), High (80-89%), Critical (90%+) |

The primary signal is `learnings_count / max_learnings`. The `protected_count` provides secondary context — a feature at 80% with all protected learnings is more urgent than one at 80% with half prunable.

## Design: Deterministic Auto-Split Task

When saturation reaches **Critical (90%+)**, Ralph deterministically creates a system task to decompose the feature. No human or agent decision — pure signal-to-task.

### Trigger Point

After every successful `append_feature_learning` call, check saturation. If critical AND no active split task already exists for this feature, create one.

**Currently blocked:** The loop engine is a stub (`src-tauri/src/loop_engine.rs`). There are no post-iteration hooks. The `append_feature_learning` Tauri command exists (`src-tauri/src/commands.rs:716`) but is only called via IPC — not from within the loop. Once the loop engine runs iterations and extracts learnings, the saturation check hooks into the learning-append path.

### Idempotency

Before creating a split task, query for existing active ones:

```sql
SELECT id FROM tasks
WHERE feature = ?
  AND provenance = 'system'
  AND title LIKE 'Decompose feature%'
  AND status IN ('pending', 'in_progress')
```

If one exists, skip. This prevents duplicate tasks when multiple learnings push a feature further past the threshold.

### Task Shape

```
provenance: system
priority: high
tags: ["auto-split", "system"]
title: "Decompose feature '{display_name}' — knowledge saturation at {pct}%"
discipline: <feature's most-used discipline by task count>
```

Description instructs Claude to:
1. Analyze the feature's tasks and learnings for natural sub-boundaries
2. Split into 2-3 focused sub-features using `create_feature` tool
3. Reassign tasks to new sub-features using `update_task` tool
4. Carry relevant learnings based on `task_id` associations (learnings already have `task_id` fields)
5. New sub-features start with fresh knowledge accumulation

### What Happens to Old Data

- **Old feature:** Stays in DB with its learnings as historical record. Tasks get reassigned away by the agent executing the split task.
- **Old JSONL journal:** `{feature}.jsonl` stays as-is in `.ralph/db/memory/`. Historical, append-only.
- **Old Qdrant collection:** Gets rebuilt from new journal state when features change.
- **New sub-features:** Get fresh empty `learnings: []`, fresh empty JSONL journals, fresh Qdrant collections.

The `task_id` field on `FeatureLearning` is key — when splitting, learnings follow their associated tasks. A learning with `task_id: 42` moves to whichever sub-feature task 42 gets reassigned to.

## Design: Frontend Progress Bar

On the Features page (`src/pages/FeaturesPage.tsx`), each feature card shows a thin knowledge saturation bar.

- Only visible if the feature has any learnings (don't clutter empty features)
- Color coded: muted (low), amber (medium), orange (high), red (critical)
- Uses existing `<Progress>` component (already imported in the file)
- Tooltip shows: "{count}/{max} learnings — {protected} protected"

Data comes from a `get_feature_saturations` Tauri command that computes saturation for all features in one pass.

## Implementation Sequence

When the loop engine and post-iteration hooks are built:

1. Add `FeatureSaturation` and `SaturationLevel` types to `crates/sqlite-db/src/types.rs`
2. Add `compute_feature_saturation` and `get_all_feature_saturations` to `crates/sqlite-db/src/stats.rs`
3. Add `saturation.rs` module to `crates/sqlite-db/` with `maybe_create_split_task`
4. Wire saturation check into `append_feature_learning` in `src-tauri/src/commands.rs`
5. Add `get_feature_saturations` Tauri command
6. Add TypeScript types to `src/types/prd.ts`
7. Add progress bar to `src/pages/FeaturesPage.tsx`
8. Tests in `crates/sqlite-db/tests/crud_operations.rs`

## Key Existing Code

| What | Where |
|------|-------|
| `FeatureLearning::is_auto_prunable()` | `crates/ralph-rag/src/learning.rs:245` |
| `select_for_pruning()` | `crates/ralph-rag/src/learning.rs:468` |
| `append_feature_learning()` | `crates/sqlite-db/src/features.rs:216` |
| `TaskProvenance::System` | `crates/sqlite-db/src/types.rs:88` |
| `max_learnings_per_feature: 50` | `crates/ralph-rag/src/config.rs:53` |
| Learning rejection at cap | `crates/sqlite-db/src/features.rs:286` (returns error when all 50 protected) |
| Features page | `src/pages/FeaturesPage.tsx` |
| `<Progress>` component | `src/components/ui/progress.tsx` |

## Relationship to Other Specs

- **Doc 017 (Feature-Scoped RAG):** Defines the memory pipeline this builds on
- **Doc 018 (Feature Entity Redesign):** Added the `learnings` field to features
- **Doc 022 (RAG Integration Guide):** Phase 4 (learnings integration) is a prerequisite
- **SPEC-050 (Orchestration Philosophy):** Ralph reacts deterministically to signals — saturation check is pure orchestration, not AI
