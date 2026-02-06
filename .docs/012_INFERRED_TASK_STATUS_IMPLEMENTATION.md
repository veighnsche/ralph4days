# Inferred Task Status Implementation

**Created:** 2026-02-06
**Status:** Implemented
**Related:** See `.docs/011_TASK_MODEL_FOR_CONCURRENT_RALPH_LOOPS.md` for full context

## Problem Statement

Tasks have an **actual status** (what's stored in YAML) but also need an **inferred status** (computed from the dependency graph).

### Example

```yaml
- id: 2
  feature: authentication
  discipline: frontend
  title: Build login form
  status: pending        # ← Actual status
  depends_on: [1]        # ← Task #1 is not done yet
```

**Question:** Is this task "pending" or "blocked by dependencies"?

**Answer:** BOTH!
- **Actual status**: `pending` (in the YAML file)
- **Inferred status**: `waiting_on_deps` (computed because task #1 isn't done)

## Solution: Dual Status Model

### Actual Status (TaskStatus enum)

What's **stored in YAML**:

```rust
pub enum TaskStatus {
    Pending,      // Not started
    InProgress,   // Claude is working on it
    Done,         // Completed
    Blocked,      // External blocker (manual, like "waiting for API key")
    Skipped,      // Intentionally skipped
}
```

### Inferred Status (InferredTaskStatus enum)

What's **computed from actual status + dependency graph**:

```rust
pub enum InferredTaskStatus {
    /// Task is ready to be claimed (pending + all deps done + not blocked)
    Ready,
    /// Task is pending but waiting on dependencies
    WaitingOnDeps,
    /// Task is manually blocked (external blocker)
    ExternallyBlocked,
    /// Task is currently being worked on
    InProgress,
    /// Task has been completed
    Done,
    /// Task was intentionally skipped
    Skipped,
}
```

## Computation Logic

```rust
fn compute_inferred_status(task: &Task) -> InferredTaskStatus {
    match task.status {
        TaskStatus::InProgress => InferredTaskStatus::InProgress,
        TaskStatus::Done => InferredTaskStatus::Done,
        TaskStatus::Skipped => InferredTaskStatus::Skipped,
        TaskStatus::Blocked => InferredTaskStatus::ExternallyBlocked,
        TaskStatus::Pending => {
            // Check if all dependencies are met
            let all_deps_met = task.depends_on.iter().all(|dep_id| {
                self.get_task_by_id(*dep_id)
                    .map(|dep| dep.status == TaskStatus::Done)
                    .unwrap_or(false)
            });

            if all_deps_met {
                InferredTaskStatus::Ready
            } else {
                InferredTaskStatus::WaitingOnDeps
            }
        }
    }
}
```

## UI Impact

### Status Badge Colors

```typescript
function getStatusBadge(inferredStatus: InferredTaskStatus) {
  switch (inferredStatus) {
    case "ready":              return "🟢 Ready";           // Green - can claim
    case "waiting_on_deps":    return "🟡 Waiting";         // Yellow - blocked by deps
    case "externally_blocked": return "🔴 Blocked";         // Red - manual block
    case "in_progress":        return "🔵 In Progress";     // Blue - being worked on
    case "done":               return "✅ Done";            // Green check
    case "skipped":            return "⏭️ Skipped";         // Gray skip
  }
}
```

### Task List View

```
┌─────────────────────────────────────────────┐
│ Tasks                              [+ Create]│
├─────────────────────────────────────────────┤
│ ✅ T1: Login API              [Done]         │
│ 🔵 T2: Login form             [In Progress]  │
│ 🟢 T3: Profile page           [Ready]        │ ← Can be claimed!
│ 🟡 T4: Dashboard              [Waiting]      │ ← Depends on T2, T3
│    ↳ Waiting on: T2, T3                      │
│ 🔴 T5: Payments               [Blocked]      │ ← External blocker
│    ↳ Waiting for API credentials             │
└─────────────────────────────────────────────┘
```

## Changes Made

### Backend (Rust)

1. **Added `InferredTaskStatus` enum** to `crates/yaml-db/src/lib.rs`
2. **Added `inferred_status` field** to `EnrichedTask` struct
3. **Added `compute_inferred_status()` method** to `YamlDatabase` in `database.rs`
4. **Updated `get_enriched_tasks()`** to compute and include `inferred_status`

### Frontend (TypeScript)

1. **Added `InferredTaskStatus` type** to `src/types/prd.ts`
2. **Added `inferredStatus` field** to `EnrichedTask` interface
3. **Updated `StatusFilter` type** to include inferred statuses
4. **Updated all mock data** in story files:
   - `PlaylistView.stories.tsx`
   - `PRDBody.stories.tsx`
   - `PlaylistItem.stories.tsx`

## Usage Examples

### Example 1: Pending with No Dependencies

```yaml
# tasks.yaml
- id: 3
  title: "Profile API endpoints"
  status: pending
  depends_on: []
```

```typescript
// EnrichedTask
{
  id: 3,
  status: "pending",
  inferredStatus: "ready",  // ← Can be claimed right now!
  dependsOn: []
}
```

### Example 2: Pending with Unmet Dependencies

```yaml
# tasks.yaml
- id: 4
  title: "Profile page UI"
  status: pending
  depends_on: [3]  # Task 3 is still pending
```

```typescript
// EnrichedTask
{
  id: 4,
  status: "pending",
  inferredStatus: "waiting_on_deps",  // ← Waiting for task 3
  dependsOn: [3]
}
```

### Example 3: Pending with Met Dependencies

```yaml
# tasks.yaml (after task 3 completes)
- id: 3
  status: done
- id: 4
  title: "Profile page UI"
  status: pending
  depends_on: [3]  # Task 3 is now done
```

```typescript
// EnrichedTask
{
  id: 4,
  status: "pending",
  inferredStatus: "ready",  // ← Now ready! Dependencies met
  dependsOn: [3]
}
```

### Example 4: Externally Blocked

```yaml
# tasks.yaml
- id: 5
  title: "Integrate payment gateway"
  status: blocked
  blocked_by: "Waiting for API credentials"
```

```typescript
// EnrichedTask
{
  id: 5,
  status: "blocked",
  inferredStatus: "externally_blocked",  // ← Manual override
  blockedBy: "Waiting for API credentials"
}
```

## Ralph Loop Integration

### Task Selection for Concurrent Runs

```rust
// Get tasks that are ready to be claimed
let ready_tasks: Vec<EnrichedTask> = db.get_enriched_tasks()
    .into_iter()
    .filter(|task| task.inferred_status == InferredTaskStatus::Ready)
    .collect();

// Sort by priority
ready_tasks.sort_by(|a, b| {
    priority_value(b.priority).cmp(&priority_value(a.priority))
});

// Claude instances claim the first ready task
for task in ready_tasks {
    match db.claim_task(task.id) {
        Ok(_) => {
            // Successfully claimed! Start work.
            break;
        }
        Err(_) => {
            // Another Claude claimed it. Try next task.
            continue;
        }
    }
}
```

### UI Dashboard

```typescript
// Count tasks by inferred status
const ready = tasks.filter(t => t.inferredStatus === "ready").length;
const waiting = tasks.filter(t => t.inferredStatus === "waiting_on_deps").length;
const blocked = tasks.filter(t => t.inferredStatus === "externally_blocked").length;

// Show actionable metrics
<Dashboard>
  <Metric label="Ready to Work" value={ready} color="green" />
  <Metric label="Waiting on Deps" value={waiting} color="yellow" />
  <Metric label="Blocked" value={blocked} color="red" />
</Dashboard>
```

## Benefits

1. **Clear task eligibility** - UI shows which tasks can actually be worked on
2. **Dependency visualization** - See why a task is blocked
3. **Better filtering** - Filter by "ready" to see actionable work
4. **Concurrent execution** - Multiple Claude instances can query ready tasks
5. **Accurate progress tracking** - Distinguish between "pending" and "blocked"

## Next Steps

To fully utilize this:

1. **Update UI components** to display `inferredStatus` badges
2. **Add filter chips** for "Ready", "Waiting on Deps", "Blocked"
3. **Show dependency chains** in task detail view
4. **Implement `get_ready_tasks()`** method (see doc 011)
5. **Add "Claim Task"** button in UI (for manual testing)

## Related Files

**Backend:**
- `crates/yaml-db/src/lib.rs:31-58` (InferredTaskStatus enum)
- `crates/yaml-db/src/lib.rs:109` (EnrichedTask.inferred_status field)
- `crates/yaml-db/src/database.rs:453-476` (compute_inferred_status method)

**Frontend:**
- `src/types/prd.ts:4-14` (InferredTaskStatus type)
- `src/types/prd.ts:33` (EnrichedTask.inferredStatus field)
- `src/components/prd/*.stories.tsx` (updated mock data)

## See Also

- **`.docs/011_TASK_MODEL_FOR_CONCURRENT_RALPH_LOOPS.md`** - Full analysis and design
- **SPEC-050: Ralph Orchestration Philosophy** - Core principles
