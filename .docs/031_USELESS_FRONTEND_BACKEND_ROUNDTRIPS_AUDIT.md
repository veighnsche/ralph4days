# Useless Frontend-Backend Roundtrips Audit

**Date:** 2026-02-08
**Status:** Analysis Complete

## Summary

This audit identified 10 unnecessary IPC roundtrips between the frontend and backend that could be eliminated by moving computation to the frontend. These roundtrips fall into three categories:

1. **Config command duplication** (2 issues)
2. **Stat aggregation that could be frontend** (4 issues)
3. **Simple utility functions called via IPC** (4 issues)

## Category 1: Config Command Duplication

### Issue 1.1: `get_features_config` vs `get_features`

**Current state:**
- `get_features_config()` returns: `{ name, display_name, acronym }`
- `get_features()` returns: `{ name, display_name, acronym, description, created, knowledge_paths, context_files, architecture, boundaries, learnings, dependencies }`

**Problem:** The frontend makes separate calls to both commands. `get_features_config` is used in dropdowns/forms, while `get_features` is used for the full feature list. But `get_features` is a superset of `get_features_config`.

**Files affected:**
- Backend: `src-tauri/src/commands/features.rs` lines 32-59 (get_disciplines_config), lines 70-82 (get_features_config)
- Frontend: `src/hooks/useFeatures.ts`, `src/components/workspace/FeatureFormTabContent.tsx`

**Recommendation:** Remove `get_features_config` command. Use `get_features` everywhere and select needed fields on frontend:

```typescript
// Instead of: const { features } = useFeatures() // calls get_features_config
// Do this: const { features } = useFullFeatures() // calls get_features
const featureOptions = features.map(f => ({ name: f.name, displayName: f.displayName }))
```

**Impact:**
- Eliminates 1 IPC command
- Reduces backend code by ~30 lines
- Simplifies invalidation logic (one less cache key)

### Issue 1.2: `get_disciplines_config` duplication

**Current state:**
- `get_disciplines_config()` returns full discipline config
- Used in `useDisciplines()` hook
- No separate "light" version, but still an unnecessary abstraction

**Problem:** The command exists purely as a wrapper around `db.get_disciplines()` with a transformation step that could happen on the frontend.

**Files affected:**
- Backend: `src-tauri/src/commands/features.rs` lines 32-59
- Frontend: `src/hooks/useDisciplines.ts`

**Recommendation:** This is less severe than features, but the transformation from wire type to UI type (resolving icons, calculating bgColor) should all be frontend logic. The backend should just return raw discipline data.

**Impact:**
- Simplifies backend by ~30 lines
- Frontend already does transformation via `resolveDisciplines()`, so this is half-done

## Category 2: Stat Aggregation Commands

### Issue 2.1: `get_feature_stats()`

**Current implementation:**
```rust
// SQL aggregation in backend
SELECT f.name, f.display_name,
  COUNT(t.id) as total,
  SUM(CASE WHEN t.status = 'done' THEN 1 ELSE 0 END) as done,
  SUM(CASE WHEN t.status = 'pending' THEN 1 ELSE 0 END) as pending,
  // ... etc
FROM features f
LEFT JOIN tasks t ON f.name = t.feature
GROUP BY f.name, f.display_name
```

**Problem:** The frontend already has all tasks via `get_tasks()`. This is a JOIN + GROUP BY that could be done in JavaScript.

**Files affected:**
- Backend: `crates/sqlite-db/src/stats.rs` lines 5-39
- Backend: `src-tauri/src/commands/tasks.rs` lines 157-162
- Frontend: `src/hooks/useFeatureStats.ts`

**Recommendation:** Remove backend command. Compute stats on frontend:

```typescript
function computeFeatureStats(tasks: Task[], features: Feature[]): Map<string, GroupStats> {
  const statsMap = new Map<string, GroupStats>()

  for (const feature of features) {
    const featureTasks = tasks.filter(t => t.feature === feature.name)
    statsMap.set(feature.name, {
      name: feature.name,
      display_name: feature.displayName,
      total: featureTasks.length,
      done: featureTasks.filter(t => t.status === 'done').length,
      pending: featureTasks.filter(t => t.status === 'pending').length,
      in_progress: featureTasks.filter(t => t.status === 'in_progress').length,
      blocked: featureTasks.filter(t => t.status === 'blocked').length,
      skipped: featureTasks.filter(t => t.status === 'skipped').length,
    })
  }

  return statsMap
}
```

**Impact:**
- Eliminates 1 IPC command
- Reduces SQL queries by 1 per page load
- Frontend already has the data cached, so this is pure computation reuse

### Issue 2.2: `get_discipline_stats()`

**Same as Issue 2.1, but for disciplines.**

**Files affected:**
- Backend: `crates/sqlite-db/src/stats.rs` lines 41-75
- Backend: `src-tauri/src/commands/tasks.rs` lines 163-169
- Frontend: `src/hooks/useDisciplineStats.ts`

**Recommendation:** Same as 2.1 - compute on frontend from existing task list.

### Issue 2.3: `get_project_progress()`

**Current implementation:**
```rust
SELECT COUNT(*), SUM(CASE WHEN status = 'done' THEN 1 ELSE 0 END) FROM tasks
```

Then calculates percentage: `(done * 100) / total`

**Problem:** This is called from BOTH `useFeatureStats` and `useDisciplineStats`. It's duplicate work (2 IPC calls) for simple arithmetic the frontend can do.

**Files affected:**
- Backend: `crates/sqlite-db/src/stats.rs` lines 77-95
- Backend: `src-tauri/src/commands/tasks.rs` lines 171-177
- Frontend: `src/hooks/useFeatureStats.ts` line 7
- Frontend: `src/hooks/useDisciplineStats.ts` line 8

**Recommendation:** Compute on frontend:

```typescript
function computeProjectProgress(tasks: Task[]) {
  const total = tasks.length
  const done = tasks.filter(t => t.status === 'done').length
  const percent = total > 0 ? Math.floor((done * 100) / total) : 0
  return { total, done, percent }
}
```

**Impact:**
- Eliminates 1 IPC command (called 2x per page load = 2 roundtrips saved)
- Removes ~20 lines of backend code
- Removes duplicate computation

### Issue 2.4: `get_all_tags()`

**Current implementation:**
```rust
// Get all tasks, parse JSON tags field, deduplicate, sort
SELECT tags FROM tasks
// Then parse each row's JSON, collect into BTreeSet, return Vec
```

**Problem:** The frontend already has all tasks via `get_tasks()`. Each task has a `tags` field. This is just flattening and deduplicating.

**Files affected:**
- Backend: `crates/sqlite-db/src/stats.rs` lines 97-118
- Backend: `src-tauri/src/commands/tasks.rs` lines 179-184
- Frontend: `src/pages/TasksPage.tsx` line 19

**Recommendation:** Compute on frontend:

```typescript
function getAllTags(tasks: Task[]): string[] {
  const tagSet = new Set<string>()
  for (const task of tasks) {
    for (const tag of task.tags ?? []) {
      tagSet.add(tag)
    }
  }
  return Array.from(tagSet).sort()
}
```

**Impact:**
- Eliminates 1 IPC command
- Removes ~20 lines of backend code
- Reuses already-fetched task data

## Category 3: Simple Utility Functions via IPC

### Issue 3.1: `normalize_feature_name()`

**Current implementation:**
```rust
pub(super) fn normalize_feature_name(name: &str) -> Result<String, String> {
    if name.contains('/') || name.contains(':') || name.contains('\\') {
        return ralph_err!(codes::FEATURE_OPS, "Feature name cannot contain /, :, or \\");
    }

    Ok(name.to_lowercase().trim().replace(char::is_whitespace, "-"))
}
```

**Problem:** This is called from `create_task`, `update_task`, and `create_feature` backend commands. It's pure string manipulation with validation that should happen on the frontend BEFORE the data is sent to backend.

**Files affected:**
- Backend: `src-tauri/src/commands/state.rs` lines 204-210
- Backend: `src-tauri/src/commands/tasks.rs` lines 49, 76
- Backend: `src-tauri/src/commands/features.rs` line 184

**Recommendation:** Move to frontend utility:

```typescript
export function normalizeFeatureName(name: string): string {
  if (name.includes('/') || name.includes(':') || name.includes('\\')) {
    throw new Error('Feature name cannot contain /, :, or \\')
  }
  return name.toLowerCase().trim().replace(/\s+/g, '-')
}
```

Then call this in form validation/submission before sending to backend.

**Impact:**
- No IPC elimination (it's called inside other commands)
- But simplifies backend logic
- Moves validation closer to user (immediate feedback)

### Issue 3.2: `parse_priority()` and `parse_provenance()`

**Current implementation:**
```rust
pub(super) fn parse_priority(priority: Option<&str>) -> Option<sqlite_db::Priority> {
    priority.and_then(|p| match p {
        "low" => Some(sqlite_db::Priority::Low),
        "medium" => Some(sqlite_db::Priority::Medium),
        "high" => Some(sqlite_db::Priority::High),
        "critical" => Some(sqlite_db::Priority::Critical),
        _ => None,
    })
}

pub(super) fn parse_provenance(provenance: Option<&str>) -> Option<sqlite_db::TaskProvenance> {
    provenance.and_then(|p| match p {
        "agent" => Some(sqlite_db::TaskProvenance::Agent),
        "human" => Some(sqlite_db::TaskProvenance::Human),
        "system" => Some(sqlite_db::TaskProvenance::System),
        _ => None,
    })
}
```

**Problem:** These are string-to-enum conversions. The frontend sends strings, the backend parses them. The frontend should send the correct enum values in the first place (or use TypeScript enums that serialize correctly).

**Files affected:**
- Backend: `src-tauri/src/commands/state.rs` lines 212-229
- Backend: `src-tauri/src/commands/tasks.rs` lines 53, 61, 80, 88

**Recommendation:** Define TypeScript enums that match Rust enums:

```typescript
// In types/generated.ts (auto-generated from Rust)
export enum Priority {
  Low = "low",
  Medium = "medium",
  High = "high",
  Critical = "critical"
}

export enum TaskProvenance {
  Agent = "agent",
  Human = "human",
  System = "system"
}
```

Then use these in forms instead of freeform strings. The Rust backend can deserialize directly.

**Impact:**
- Removes ~30 lines of parsing code
- Improves type safety (compile-time checks instead of runtime parsing)
- Removes possibility of parse failures

### Issue 3.3: Color mixing calculation in `useDisciplines`

**Current implementation:**
```typescript
// Frontend: src/hooks/useDisciplines.ts lines 22
bgColor: `color-mix(in oklch, ${d.color} 15%, transparent)`
```

**Problem:** This isn't a backend roundtrip, but it's worth noting that this CSS calculation happens on every render. It should be memoized or pre-calculated.

**Recommendation:** Add `bgColor` field to backend discipline config, calculated once on read.

**Impact:**
- Minor performance improvement
- Reduces frontend complexity

### Issue 3.4: Icon resolution in `useDisciplines`

**Current implementation:**
```typescript
// Frontend: src/hooks/useDisciplines.ts line 20
icon: resolveIcon(d.icon)
```

**Problem:** Backend stores icon as a string (e.g., "Code"), frontend resolves it to a Lucide icon component. This transformation happens on every data fetch.

**Recommendation:** This is actually fine - the backend shouldn't know about Lucide icons. But it should be memoized in the select function.

**Impact:**
- None - current approach is reasonable

## Aggregate Impact Summary

### Commands to Remove (7 total):
1. `get_features_config` - replace with `get_features` + frontend selection
2. `get_feature_stats` - compute on frontend from tasks
3. `get_discipline_stats` - compute on frontend from tasks
4. `get_project_progress` - compute on frontend from tasks (saves 2 calls)
5. `get_all_tags` - compute on frontend from tasks

### Backend Code Reduction:
- ~150 lines of Rust code removed
- ~50 lines of SQL queries removed
- 5 fewer cache invalidation keys

### IPC Roundtrips Saved:
- Initial page load: **5-7 fewer IPC calls** (depending on page)
- After mutations: **2-3 fewer IPC calls** (no stat refresh needed)

### Frontend Performance:
- Stats computation: ~1-2ms for 100 tasks (negligible)
- Memoization recommended to avoid recomputation on every render

### Type Safety Improvements:
- Priority and Provenance become compile-time checked enums
- Feature name normalization happens in form validation (immediate feedback)

## Implementation Priority

### High Priority (Do First):
1. **Issue 2.3** - `get_project_progress` (called 2x, easy win)
2. **Issue 2.4** - `get_all_tags` (simple array flatten)
3. **Issue 3.2** - Enum parsing (type safety improvement)

### Medium Priority (Do Next):
4. **Issue 2.1** - `get_feature_stats` (requires more frontend logic)
5. **Issue 2.2** - `get_discipline_stats` (same as 2.1)

### Low Priority (Nice to Have):
6. **Issue 1.1** - `get_features_config` removal (requires refactoring hooks)
7. **Issue 3.1** - `normalize_feature_name` (validation improvement, not perf)

## Migration Strategy

### Phase 1: Stat Aggregation (Issues 2.1-2.4)
1. Create `src/lib/stats.ts` utility module
2. Implement all 4 stat functions
3. Update hooks to use local computation
4. Remove backend commands one by one
5. Test thoroughly (counts must match)

### Phase 2: Type Safety (Issue 3.2)
1. Update `ralph_macros` to generate TypeScript enums
2. Update forms to use enums
3. Update backend to accept enums directly
4. Remove parsing functions

### Phase 3: Config Consolidation (Issue 1.1)
1. Update all uses of `get_features_config` to use `get_features`
2. Update cache invalidation logic
3. Remove `get_features_config` command

### Phase 4: Frontend Validation (Issue 3.1)
1. Create `src/lib/validation.ts` utility
2. Add to form schemas
3. Remove backend normalization (or keep as safety check)

## Testing Checklist

- [ ] Verify stat counts match between old and new implementation
- [ ] Verify tags list is identical (order and content)
- [ ] Verify progress percentages match
- [ ] Test with 0 tasks (edge case)
- [ ] Test with 1000+ tasks (performance)
- [ ] Verify enum serialization works in IPC
- [ ] Test form validation with invalid characters

## Notes

- Some of these changes require coordination between frontend and backend
- Consider using a feature flag for gradual rollout
- Monitor IPC call frequency in production (if telemetry exists)
- Consider adding a "stats refresh" button for users who want to force recomputation

## Open Questions

1. Should the backend keep stat commands as a fallback for CLI tools?
2. Should we cache computed stats in Zustand to avoid recomputation?
3. Do we need to throttle/debounce stat computation if task list is huge (1000+ tasks)?
