# Task Entity Model for Concurrent Ralph Loops

**Created:** 2026-02-06
**Status:** Design Analysis

## What Are Ralph Loops?

Ralph loops are an **iterative autonomous agent execution pattern** where:

- Each iteration is a **fresh Claude instance** with clean context
- Progress persists in **files** (git history, progress.txt, tasks.yaml), NOT in LLM context
- Each iteration picks the **highest priority task** where status != done
- Simple, deterministic iteration beats sophisticated complexity
- Named after Ralph Wiggum from The Simpsons ("I'm in danger" meme)

### Key Insight

> **Naive persistence beats sophisticated complexity.**
> Memory lives in files, not context windows.

## User Requirements

1. **Traditional ralph loops** - single Claude instance iteratively working through tasks
2. **Concurrent ralph runs** - multiple Claude instances working on different tasks in parallel
3. **Dependency graph tracking** - know which tasks can run because all dependencies are met

## Current Task Model Analysis

### Schema (Already Perfect!)

```rust
pub struct Task {
    pub id: u32,
    pub feature: String,
    pub discipline: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,              // ← State tracking
    pub priority: Option<Priority>,       // ← Task ordering
    pub tags: Vec<String>,
    pub depends_on: Vec<u32>,            // ← Dependency graph!
    pub blocked_by: Option<String>,       // ← External blockers
    pub created: Option<String>,
    pub updated: Option<String>,
    pub completed: Option<String>,
    pub acceptance_criteria: Vec<String>,
}

pub enum TaskStatus {
    Pending,      // ← Not started
    InProgress,   // ← Claude is working on it
    Done,         // ← Completed
    Blocked,      // ← External blocker (manual)
    Skipped,      // ← Intentionally skipped
}

pub enum Priority {
    Low, Medium, High, Critical  // ← For sorting ready tasks
}
```

### What We Already Have ✅

- **Dependency tracking**: `depends_on: Vec<u32>` for task dependencies
- **Status lifecycle**: Pending → InProgress → Done
- **Priority sorting**: Critical > High > Medium > Low
- **Thread safety**: File locking via `fs2` crate
- **Circular dependency detection**: `has_circular_dependency()` method
- **Atomic operations**: `acquire_lock()` → `load_all()` → mutate → `save_all()`

### What We Need to Add 🔧

**NO schema changes needed!** Just add query/coordination methods.

## Dependency Graph Semantics

### Task Readiness States

1. **Ready** = `status == Pending` AND all `depends_on` tasks have `status == Done` AND `blocked_by.is_none()`
2. **Waiting on dependencies** = `status == Pending` AND some `depends_on` tasks are not `Done`
3. **Externally blocked** = `status == Blocked` (manual override)
4. **In progress** = `status == InProgress` (claimed by a Claude instance)
5. **Completed** = `status == Done`

### Blocking vs Dependencies

- **`depends_on: Vec<u32>`** - Dependency-based blocking (computed from task graph)
- **`blocked_by: Option<String>`** - External/manual blocking (e.g., "waiting for API key from client")
- **`status: Blocked`** - Explicit blocked status (separate from dependency blocking)

A task is **ready to run** if it's `Pending`, has no unmet dependencies, and isn't externally blocked.

## Required Database Methods

### 1. Query Ready Tasks

```rust
impl YamlDatabase {
    /// Get tasks ready to run (no unmet dependencies, sorted by priority)
    /// Returns tasks sorted by: priority DESC, created ASC
    pub fn get_ready_tasks(&self) -> Vec<&Task> {
        let status_map: HashMap<u32, TaskStatus> = self.tasks
            .get_all()
            .iter()
            .map(|t| (t.id, t.status))
            .collect();

        let mut ready: Vec<&Task> = self.tasks
            .get_all()
            .iter()
            .filter(|task| {
                // Must be Pending
                if task.status != TaskStatus::Pending {
                    return false;
                }

                // Must not be externally blocked
                if task.status == TaskStatus::Blocked {
                    return false;
                }

                // All dependencies must be Done
                task.depends_on.iter().all(|dep_id| {
                    status_map.get(dep_id) == Some(&TaskStatus::Done)
                })
            })
            .collect();

        // Sort by priority DESC, then created ASC (FIFO for ties)
        ready.sort_by(|a, b| {
            let priority_order = |p: Option<Priority>| match p {
                Some(Priority::Critical) => 4,
                Some(Priority::High) => 3,
                Some(Priority::Medium) => 2,
                Some(Priority::Low) => 1,
                None => 0,
            };

            match priority_order(b.priority).cmp(&priority_order(a.priority)) {
                Ordering::Equal => a.created.cmp(&b.created),
                other => other,
            }
        });

        ready
    }
}
```

### 2. Claim Task (Atomic)

```rust
impl YamlDatabase {
    /// Atomically claim a task for execution
    /// Sets status: Pending → InProgress
    /// Thread-safe: Uses exclusive file lock
    ///
    /// # Errors
    /// - Task doesn't exist
    /// - Task is not Pending
    /// - Task has unmet dependencies
    pub fn claim_task(&mut self, id: u32) -> Result<(), String> {
        // 1. Acquire exclusive lock
        let _lock = self.acquire_lock()?;

        // 2. Reload from disk (ensure fresh state)
        self.load_all()?;

        // 3. Find the task
        let task = self.get_task_by_id(id)
            .ok_or_else(|| format!("Task {} does not exist", id))?;

        // 4. Validate it's claimable
        if task.status != TaskStatus::Pending {
            return Err(format!("Task {} is not pending (status: {:?})", id, task.status));
        }

        // 5. Check dependencies are met
        if !self.is_task_ready(task) {
            return Err(format!("Task {} has unmet dependencies", id));
        }

        // 6. Update status to InProgress
        self.update_task_status_internal(id, TaskStatus::InProgress)?;

        // 7. Save atomically
        self.save_all()?;

        Ok(())
    }
}
```

### 3. Complete Task

```rust
impl YamlDatabase {
    /// Mark a task as complete
    /// Sets status: InProgress → Done
    /// Updates completion timestamp
    /// Thread-safe: Uses exclusive file lock
    pub fn complete_task(&mut self, id: u32) -> Result<(), String> {
        // 1. Acquire exclusive lock
        let _lock = self.acquire_lock()?;

        // 2. Reload from disk
        self.load_all()?;

        // 3. Find the task
        let task_index = self.tasks
            .items_mut()
            .iter()
            .position(|t| t.id == id)
            .ok_or_else(|| format!("Task {} does not exist", id))?;

        // 4. Validate it's in progress
        if self.tasks.items_mut()[task_index].status != TaskStatus::InProgress {
            return Err(format!("Task {} is not in progress", id));
        }

        // 5. Update status and timestamp
        self.tasks.items_mut()[task_index].status = TaskStatus::Done;
        self.tasks.items_mut()[task_index].completed =
            Some(chrono::Utc::now().format("%Y-%m-%d").to_string());
        self.tasks.items_mut()[task_index].updated =
            Some(chrono::Utc::now().format("%Y-%m-%d").to_string());

        // 6. Save atomically
        self.save_all()?;

        Ok(())
    }
}
```

### 4. General Status Update

```rust
impl YamlDatabase {
    /// Update task status with validation
    /// Thread-safe: Uses exclusive file lock
    pub fn update_task_status(
        &mut self,
        id: u32,
        status: TaskStatus
    ) -> Result<(), String> {
        let _lock = self.acquire_lock()?;
        self.load_all()?;
        self.update_task_status_internal(id, status)?;
        self.save_all()?;
        Ok(())
    }

    /// Internal status update (assumes lock held)
    fn update_task_status_internal(
        &mut self,
        id: u32,
        status: TaskStatus
    ) -> Result<(), String> {
        let task_index = self.tasks
            .items_mut()
            .iter()
            .position(|t| t.id == id)
            .ok_or_else(|| format!("Task {} does not exist", id))?;

        self.tasks.items_mut()[task_index].status = status;
        self.tasks.items_mut()[task_index].updated =
            Some(chrono::Utc::now().format("%Y-%m-%d").to_string());

        if status == TaskStatus::Done {
            self.tasks.items_mut()[task_index].completed =
                Some(chrono::Utc::now().format("%Y-%m-%d").to_string());
        }

        Ok(())
    }
}
```

## Helper Methods for Dependency Analysis

### 5. Check Task Readiness (Private)

```rust
impl YamlDatabase {
    /// Check if a task has all dependencies met
    fn is_task_ready(&self, task: &Task) -> bool {
        // Externally blocked tasks are never ready
        if task.status == TaskStatus::Blocked {
            return false;
        }

        // All dependencies must be Done
        task.depends_on.iter().all(|dep_id| {
            self.get_task_by_id(*dep_id)
                .map(|dep| dep.status == TaskStatus::Done)
                .unwrap_or(false) // Missing dependency = not ready
        })
    }
}
```

### 6. Get Blocking Tasks

```rust
impl YamlDatabase {
    /// Get IDs of tasks blocking this one (unmet dependencies)
    pub fn get_blocking_tasks(&self, id: u32) -> Vec<u32> {
        let task = match self.get_task_by_id(id) {
            Some(t) => t,
            None => return vec![],
        };

        task.depends_on
            .iter()
            .filter(|dep_id| {
                self.get_task_by_id(**dep_id)
                    .map(|dep| dep.status != TaskStatus::Done)
                    .unwrap_or(true)
            })
            .copied()
            .collect()
    }
}
```

### 7. Get Blocked Tasks

```rust
impl YamlDatabase {
    /// Get IDs of tasks that depend on this one
    /// Useful for showing "completing this will unblock N tasks"
    pub fn get_blocked_tasks(&self, id: u32) -> Vec<u32> {
        self.tasks
            .get_all()
            .iter()
            .filter(|task| task.depends_on.contains(&id))
            .map(|task| task.id)
            .collect()
    }

    /// Get IDs of tasks that will become ready if this task completes
    pub fn get_tasks_unblocked_by_completion(&self, id: u32) -> Vec<u32> {
        self.get_blocked_tasks(id)
            .into_iter()
            .filter(|blocked_id| {
                let task = self.get_task_by_id(*blocked_id).unwrap();
                // Would this be the last blocking dependency?
                let blocking = self.get_blocking_tasks(*blocked_id);
                blocking.len() == 1 && blocking[0] == id
            })
            .collect()
    }
}
```

## Frontend Enhancements

### Enriched Task with Dependency Metadata

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrichedTask {
    // ... existing fields ...
    pub depends_on: Vec<u32>,
    pub blocked_by: Option<String>,

    // NEW: Computed dependency metadata
    pub is_ready: bool,                    // Can be claimed right now
    pub blocking_task_ids: Vec<u32>,       // Tasks blocking this one
    pub blocked_task_ids: Vec<u32>,        // Tasks this one blocks
    pub will_unblock_count: u32,           // How many tasks will become ready
}
```

### Updated get_enriched_tasks()

```rust
impl YamlDatabase {
    pub fn get_enriched_tasks(&self) -> Vec<EnrichedTask> {
        let features = self.features.get_all();
        let disciplines = self.disciplines.get_all();

        self.tasks
            .get_all()
            .iter()
            .map(|task| {
                let feature = features.iter().find(|f| f.name == task.feature);
                let discipline = disciplines.iter().find(|d| d.name == task.discipline);

                // Compute dependency metadata
                let is_ready = task.status == TaskStatus::Pending
                    && self.is_task_ready(task);
                let blocking_task_ids = self.get_blocking_tasks(task.id);
                let blocked_task_ids = self.get_blocked_tasks(task.id);
                let will_unblock_count =
                    self.get_tasks_unblocked_by_completion(task.id).len() as u32;

                EnrichedTask {
                    // ... existing field mappings ...
                    is_ready,
                    blocking_task_ids,
                    blocked_task_ids,
                    will_unblock_count,
                }
            })
            .collect()
    }
}
```

## Concurrent Execution Flow

### Scenario: 3 Claude Instances Working in Parallel

```
Time  | Claude-A        | Claude-B        | Claude-C
------|-----------------|-----------------|------------------
T0    | Query ready     | Query ready     | Query ready
      | → [T1, T3, T7]  | → [T1, T3, T7]  | → [T1, T3, T7]
      |                 |                 |
T1    | claim_task(1)   | claim_task(1)   | claim_task(3)
      | → SUCCESS ✓     | → FAIL (T1 now  | → SUCCESS ✓
      |                 |    InProgress)  |
      |                 |                 |
T2    | Working on T1   | Query ready     | Working on T3
      |                 | → [T7]          |
      |                 | claim_task(7)   |
      |                 | → SUCCESS ✓     |
      |                 |                 |
T3    | Working on T1   | Working on T7   | Working on T3
      |                 |                 |
T4    | complete_task(1)| Working on T7   | complete_task(3)
      | → Task #5 now   |                 | → Task #9 now
      |   unblocked!    |                 |   unblocked!
```

### Race Condition Prevention

**File locking ensures atomicity:**

```rust
// Claude-A and Claude-B both try to claim task #1
// Whichever acquires the lock first wins:

// Claude-A (gets lock first)
acquire_lock() → ✓
load_all() → sees T1 is Pending
set T1 to InProgress → ✓
save_all() → ✓
release_lock() → ✓

// Claude-B (waits for lock, then fails validation)
acquire_lock() → ✓ (waited for Claude-A to release)
load_all() → sees T1 is InProgress
validation fails → ✗ "Task 1 is not pending"
release_lock() → picks a different task
```

## UI/UX Visualizations

### Task List with Ready Badges

```
┌─────────────────────────────────────────────┐
│ Tasks                              [+ Create]│
├─────────────────────────────────────────────┤
│ ✅ T1: Login API              [Done]         │
│ 🔵 T2: Login form             [In Progress]  │
│ 🟢 T3: Profile page           [Ready]        │
│ 🟡 T4: Dashboard              [Blocked: 2]   │
│    ↳ Waiting on: T2, T3                      │
│ ⚪ T5: Settings page          [Pending]      │
│    ↳ Waiting on: T4                          │
└─────────────────────────────────────────────┘

Legend:
✅ Done
🔵 In Progress (claimed by Claude)
🟢 Ready (can be claimed)
🟡 Blocked (waiting on dependencies)
⚪ Pending (not ready, may have dependencies)
```

### Dependency Graph View

```
        T1 (Done)
       /  \
      /    \
    T2      T3 (Ready)
  (InProg)   \
      \       \
       \       T5 (Pending)
        \     /
         \   /
          T4 (Blocked: waiting on T2, T3)
```

### Task Detail Panel

```
┌─────────────────────────────────────────┐
│ Task #4: User Dashboard                 │
│ Status: Blocked                         │
│                                         │
│ 🚫 Blocked by 2 dependencies:           │
│   • #2: Login form (In Progress)        │
│   • #3: Profile page (Ready)            │
│                                         │
│ ✓ Will unblock 1 task when complete:    │
│   • #5: Settings page                   │
│                                         │
│ [View Graph] [Edit Dependencies]        │
└─────────────────────────────────────────┘
```

## Ralph Loop Integration

### Single Loop (Traditional)

```rust
// Ralph loop: iterative execution
loop {
    let ready_tasks = db.get_ready_tasks();

    if ready_tasks.is_empty() {
        // No more work
        break;
    }

    // Pick highest priority ready task
    let task = ready_tasks[0];

    // Claim it
    db.claim_task(task.id)?;

    // Generate prompt for Claude
    let prompt = generate_task_prompt(task);

    // Launch Claude CLI
    let result = run_claude_cli(&prompt)?;

    // Mark complete
    db.complete_task(task.id)?;

    // Stagnation detection...
}
```

### Concurrent Loops

```rust
// Ralph concurrent: spawn multiple Claude instances
let num_workers = 3;
let (tx, rx) = mpsc::channel();

for worker_id in 0..num_workers {
    let tx = tx.clone();
    let db_path = project_path.join(".ralph/db");

    thread::spawn(move || {
        loop {
            // Each worker has its own db instance
            let mut db = YamlDatabase::from_path(db_path.clone())?;

            let ready_tasks = db.get_ready_tasks();
            if ready_tasks.is_empty() {
                break; // No work left
            }

            // Try to claim first ready task
            let task = ready_tasks[0];
            match db.claim_task(task.id) {
                Ok(_) => {
                    // Successfully claimed, do work
                    let prompt = generate_task_prompt(task);
                    run_claude_cli(&prompt)?;
                    db.complete_task(task.id)?;
                    tx.send((worker_id, task.id, "completed"))?;
                },
                Err(_) => {
                    // Another worker claimed it first, try next task
                    continue;
                }
            }
        }
    });
}
```

## Performance Considerations

### Complexity Analysis

- `get_ready_tasks()`: O(n) where n = total tasks (single pass with filter)
- `claim_task()`: O(1) with file lock (constant time validation)
- `complete_task()`: O(1) with file lock
- `is_task_ready()`: O(m) where m = avg dependencies per task
- `get_blocked_tasks()`: O(n) (linear scan)

### Optimization Opportunities

For projects with >1000 tasks:

1. **Cache status map** - build once per query batch
2. **Index depends_on** - reverse mapping: `task_id → [dependent_task_ids]`
3. **Memoize readiness** - cache `is_ready` computation results

But for typical Ralph projects (<100 tasks), current approach is fine.

### File Lock Contention

With 3+ concurrent Claude instances:
- File lock becomes serialization point
- Each `claim_task()` blocks other instances briefly (~10ms)
- This is intentional and correct (prevents race conditions)
- Alternative: optimistic locking with retry logic

## Summary

### What's Great About Current Design ✅

1. **Task model already has everything needed** - no schema changes!
2. **Thread-safe file locking** - concurrent access works
3. **Dependency graph built-in** - `depends_on` field handles it
4. **Status lifecycle clear** - Pending → InProgress → Done
5. **Priority sorting ready** - Critical > High > Medium > Low

### What to Implement 🔧

1. **Add 7 new methods to YamlDatabase**:
   - `get_ready_tasks()` - query ready tasks
   - `claim_task()` - atomic Pending → InProgress
   - `complete_task()` - atomic InProgress → Done
   - `update_task_status()` - general status update
   - `is_task_ready()` - dependency check
   - `get_blocking_tasks()` - what blocks this task
   - `get_blocked_tasks()` - what this task blocks

2. **Extend EnrichedTask** with computed fields:
   - `is_ready: bool`
   - `blocking_task_ids: Vec<u32>`
   - `blocked_task_ids: Vec<u32>`
   - `will_unblock_count: u32`

3. **Update frontend** to visualize:
   - Ready tasks (can be claimed)
   - Blocked tasks (waiting on dependencies)
   - Dependency graph view

### Key Insight

> **Ralph loops are about file-based state persistence.**
> The task model already supports this perfectly via `depends_on` + `status`.
> We just need query/coordination methods for concurrent execution.

## References

- **Ralph loop concept**: [Vercel Labs ralph-loop-agent](https://github.com/vercel-labs/ralph-loop-agent)
- **What is Ralph Loop**: [Medium article by Ewan Mak](https://medium.com/@tentenco/what-is-ralph-loop-a-new-era-of-autonomous-coding-96a4bb3e2ac8)
- **From ReAct to Ralph**: [Alibaba Cloud blog](https://www.alibabacloud.com/blog/from-react-to-ralph-loop-a-continuous-iteration-paradigm-for-ai-agents_602799)
- **Current DB implementation**: `crates/yaml-db/src/database.rs:213-272` (has circular dependency detection!)
