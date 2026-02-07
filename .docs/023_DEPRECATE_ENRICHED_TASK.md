# Deprecate EnrichedTask — Fold Back Into Task

**Created:** 2026-02-07
**Status:** Implementation Plan
**Enforces:** CLAUDE.md "Evolve, Don't Rename" rule

## The Problem

During development, display fields (feature/discipline names, icons, colors) and inferred status were added to tasks. Instead of adding them to `Task`, a new type `EnrichedTask` was created alongside it. Now there are two types for the same thing and two query paths.

This is the rename-on-develop antipattern:
```
Phase 1: Task
Phase 2: EnrichedTask (Task + display fields)
Phase 3: ???EnrichedTask (+ deadlines, + learnings, + ...)
```

The fix: fold `EnrichedTask` back into `Task`. Future development evolves `Task` directly.

## What EnrichedTask Has That Task Doesn't

7 fields, all populated via SQL JOIN or computation:

```
inferredStatus        ← computed from status + dependency graph
featureDisplayName    ← JOIN features
featureAcronym        ← JOIN features
disciplineDisplayName ← JOIN disciplines
disciplineAcronym     ← JOIN disciplines
disciplineIcon        ← JOIN disciplines
disciplineColor       ← JOIN disciplines
```

## Blast Radius

### Rust

| Symbol | File | Action |
|--------|------|--------|
| `EnrichedTask` struct | `types.rs:199` | Delete — fields move into `Task` |
| `get_enriched_tasks()` | `tasks.rs:363` | Delete — its JOIN query becomes `get_tasks()` |
| `get_tasks()` | `tasks.rs:334` | Absorb the JOIN query from `get_enriched_tasks()` |
| `get_task_by_id()` | `tasks.rs:313` | Update to JOIN (currently returns bare Task) |
| `row_to_task()` | `tasks.rs:461` | Update to parse wider row |
| `lib.rs` re-exports | `lib.rs` | Remove `EnrichedTask` |
| `get_enriched_tasks` IPC | `commands.rs:778` | Rename to `get_tasks` |
| `lib.rs` registration | `lib.rs:76` | Update command name |
| 4 tests | `crud_operations.rs` | `get_enriched_tasks()` → `get_tasks()` |
| `PromptContext.tasks` | `context.rs:9` | No change — already `Vec<Task>`, gets fuller data now |
| `loop_engine.rs:474` | loop_engine.rs | No change — already calls `get_tasks()` |
| `export.rs:87` | export.rs | No change — already calls `get_tasks()` |

### TypeScript

| Symbol | File | Action |
|--------|------|--------|
| `PRDTask` interface | `prd.ts:32` | Delete — was the "base" type |
| `EnrichedTask` interface | `prd.ts:58` | Delete — define `Task` with all fields |
| 17 component/hook files | various | `EnrichedTask` → `Task` (mechanical find-replace) |
| `usePRDData.ts` | hooks | `"get_enriched_tasks"` → `"get_tasks"` IPC call |
| `taskId.ts` | lib | `PRDTask` → `Task` |
| `constants/prd.ts` | constants | `PRDTask["status"]` → `Task["status"]` |

## Execution

All mechanical. No behavior change, no schema change.

### Rust

1. Add the 7 fields to `Task` in `types.rs`. Delete `EnrichedTask`.
2. Move the JOIN query from `get_enriched_tasks()` into `get_tasks()`. Delete `get_enriched_tasks()`.
3. Update `get_task_by_id()` to use the same JOIN.
4. Update `row_to_task()` for the wider row.
5. Rename `get_enriched_tasks` IPC command to `get_tasks` in `commands.rs` and `lib.rs`.
6. Update tests.

### TypeScript

1. Replace `PRDTask` + `EnrichedTask` with single `Task` interface in `prd.ts`.
2. Find-replace `EnrichedTask` → `Task` and `PRDTask` → `Task` across all files.
3. Update IPC call from `"get_enriched_tasks"` to `"get_tasks"`.

### Verify

```bash
grep -r "EnrichedTask\|PRDTask" --include="*.rs" --include="*.ts" --include="*.tsx"
# Should return 0 hits (excluding .docs/)
```
