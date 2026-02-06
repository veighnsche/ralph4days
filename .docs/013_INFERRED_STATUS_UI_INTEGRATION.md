# Inferred Status UI Integration

**Created:** 2026-02-06
**Status:** Implemented
**Related:** See `.docs/012_INFERRED_TASK_STATUS_IMPLEMENTATION.md` for backend implementation

## Overview

Updated the frontend to display inferred task status **only when it differs from the actual status**. This provides users with actionable information about task readiness without cluttering the UI with redundant data.

## Design Principle

**Show inferred status only when it adds value:**

| Actual Status | Inferred Status | Display Inferred? | Why?
|---------------|-----------------|-------------------|------
| `pending`     | `ready`         | ✅ YES            | Indicates task can be claimed
| `pending`     | `waiting_on_deps` | ✅ YES          | Explains why task can't start yet
| `in_progress` | `in_progress`   | ❌ NO             | Redundant
| `done`        | `done`          | ❌ NO             | Redundant
| `blocked`     | `externally_blocked` | ❌ NO        | Redundant
| `skipped`     | `skipped`       | ❌ NO             | Redundant

## Implementation

### 1. Added INFERRED_STATUS_CONFIG Constant

**File:** `src/constants/prd.ts`

```typescript
export const INFERRED_STATUS_CONFIG = {
  ready: {
    label: "Ready",
    icon: CheckCircle2,
    color: "var(--status-done)",  // Green
    bgColor: "color-mix(in oklch, var(--status-done) 15%, transparent)",
    description: "All dependencies met - can be claimed",
  },
  waiting_on_deps: {
    label: "Waiting on Deps",
    icon: Clock,
    color: "var(--priority-medium)",  // Yellow/Orange
    bgColor: "color-mix(in oklch, var(--priority-medium) 15%, transparent)",
    description: "Waiting for dependencies to complete",
  },
  // ... other statuses
} as const;
```

### 2. Created Helper Functions

**File:** `src/lib/taskStatus.ts`

```typescript
/**
 * Determines if the inferred status should be displayed.
 * Only show inferred status when it provides additional meaningful information.
 */
export function shouldShowInferredStatus(
  actualStatus: TaskStatus,
  inferredStatus: InferredTaskStatus
): boolean {
  // If actual status is "pending", show inferred status if it's different
  if (actualStatus === "pending") {
    return inferredStatus === "ready" || inferredStatus === "waiting_on_deps";
  }

  // For all other statuses, don't show inferred status (they map 1:1)
  return false;
}

/**
 * Get a human-readable explanation of why a task has a particular inferred status.
 */
export function getInferredStatusExplanation(
  actualStatus: TaskStatus,
  inferredStatus: InferredTaskStatus,
  dependsOnCount: number = 0
): string {
  if (actualStatus === "pending") {
    if (inferredStatus === "ready") {
      return "All dependencies met - ready to work on";
    }
    if (inferredStatus === "waiting_on_deps") {
      const depText = dependsOnCount === 1 ? "1 dependency" : `${dependsOnCount} dependencies`;
      return `Waiting on ${depText}`;
    }
  }

  return "";
}
```

### 3. Updated PlaylistItem Component

**File:** `src/components/prd/PlaylistItem.tsx`

Added inferred status badge in the metadata section (before priority badge):

```tsx
{/* Inferred Status Badge (only if different from actual status) */}
{shouldShowInferredStatus(task.status, task.inferredStatus) && (() => {
  const inferredConfig = INFERRED_STATUS_CONFIG[task.inferredStatus];
  const Icon = inferredConfig.icon;
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <div
          className="flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium cursor-help"
          style={{
            backgroundColor: inferredConfig.bgColor,
            color: inferredConfig.color,
          }}
        >
          <Icon className="w-3 h-3" />
          <span>{inferredConfig.label}</span>
        </div>
      </TooltipTrigger>
      <TooltipContent>
        {getInferredStatusExplanation(task.status, task.inferredStatus, task.dependsOn?.length ?? 0)}
      </TooltipContent>
    </Tooltip>
  );
})()}
```

### 4. Updated TaskDetailTabContent Component

**File:** `src/components/workspace/TaskDetailTabContent.tsx`

Added "Computed Status" property row in the sidebar (after actual status):

```tsx
{/* Inferred Status (only if different from actual status) */}
{shouldShowInferredStatus(task.status, task.inferredStatus) && (() => {
  const inferredConfig = INFERRED_STATUS_CONFIG[task.inferredStatus];
  const InferredIcon = inferredConfig.icon;
  const explanation = getInferredStatusExplanation(
    task.status,
    task.inferredStatus,
    task.dependsOn?.length ?? 0
  );

  return (
    <PropertyRow label="Computed Status">
      <div className="flex flex-col gap-1">
        <div className="flex items-center gap-1.5">
          <InferredIcon className="h-3.5 w-3.5" style={{ color: inferredConfig.color }} />
          <span className="text-sm" style={{ color: inferredConfig.color }}>
            {inferredConfig.label}
          </span>
        </div>
        {explanation && (
          <span className="text-xs text-muted-foreground">{explanation}</span>
        )}
      </div>
    </PropertyRow>
  );
})()}
```

### 5. Updated StatusFilter Type

**File:** `src/types/prd.ts`

Extended StatusFilter to include both actual AND inferred statuses:

```typescript
export type StatusFilter =
  | "all"
  // Actual statuses (match TaskStatus)
  | "pending"
  | "in_progress"
  | "blocked"
  | "done"
  | "skipped"
  // Inferred statuses (additional computed states)
  | "ready"              // Inferred: pending + all deps met
  | "waiting_on_deps";   // Inferred: pending + unmet deps
```

This allows users to filter by both:
- Actual status: "Show me all pending tasks"
- Inferred status: "Show me tasks ready to work on"

## Visual Examples

### Example 1: Task Ready to Work On

**Task List View:**
```
┌────────────────────────────────────────────────┐
│ AUTH-003-FRNT Login form                       │
│                                                │
│  Status: pending (in YAML)                     │
│  [✓ Ready] [3 AC] [High]                       │
│  ↑ Badge shown because inferredStatus differs  │
└────────────────────────────────────────────────┘
```

**Task Detail View:**
```
Properties
──────────
Status:           ○ Pending
Computed Status:  ✓ Ready
                  All dependencies met - ready to work on
Priority:         High Priority
```

### Example 2: Task Waiting on Dependencies

**Task List View:**
```
┌────────────────────────────────────────────────┐
│ USER-004-FRNT Profile page UI                  │
│                                                │
│  Status: pending (in YAML)                     │
│  [⏱ Waiting on Deps] [2 deps] [Medium]        │
│  ↑ Badge explains why task is blocked          │
└────────────────────────────────────────────────┘
```

**Task Detail View:**
```
Properties
──────────
Status:           ○ Pending
Computed Status:  ⏱ Waiting on Deps
                  Waiting on 2 dependencies
Priority:         Medium Priority
Feature:          User Profile
Discipline:       Frontend
```

### Example 3: Task In Progress (No Inferred Status Shown)

**Task List View:**
```
┌────────────────────────────────────────────────┐
│ AUTH-002-FRNT Login form                       │
│                                                │
│  Status: in_progress                           │
│  [3 AC] [1 deps] [Medium]                      │
│  ↑ No inferred badge - would be redundant      │
└────────────────────────────────────────────────┘
```

**Task Detail View:**
```
Properties
──────────
Status:    ▶ In Progress
           (no computed status row shown)
Priority:  Medium Priority
```

## Badge Color Scheme

| Inferred Status | Icon | Color | Meaning |
|-----------------|------|-------|---------|
| `ready` | ✓ CheckCircle2 | Green | Can claim and start work |
| `waiting_on_deps` | ⏱ Clock | Yellow/Orange | Blocked by unmet dependencies |
| `externally_blocked` | ⚠ AlertCircle | Red | Manual external blocker |

## User Benefits

1. **Quick scanning** - At a glance, see which tasks are actionable
2. **Dependency awareness** - Understand why tasks can't start yet
3. **Reduced noise** - Only show computed status when it adds value
4. **Tooltip explanations** - Hover for detailed reasoning
5. **Filtering support** - Filter by "ready" to see claimable work

## Updated Files

**Constants:**
- `src/constants/prd.ts:1-7` - Added Clock and AlertCircle imports
- `src/constants/prd.ts:37-66` - Added INFERRED_STATUS_CONFIG

**Utilities:**
- `src/lib/taskStatus.ts` - NEW - Helper functions for status display logic

**Components:**
- `src/components/prd/PlaylistItem.tsx:1-8` - Added imports
- `src/components/prd/PlaylistItem.tsx:78-103` - Added inferred status badge
- `src/components/workspace/TaskDetailTabContent.tsx:1-11` - Added imports
- `src/components/workspace/TaskDetailTabContent.tsx:120-145` - Added computed status property

**Types:**
- `src/types/prd.ts:55-63` - Extended StatusFilter type

**Stories (Updated Mock Data):**
- `src/components/prd/TaskIdDisplay.stories.tsx:42-62` - Added inferredStatus to createTask

## Next Steps

To complete the inferred status feature:

1. **Update filter UI** - Add filter chips for "Ready" and "Waiting on Deps"
2. **Add status icons to filters** - Show status icons in filter dropdown
3. **Implement filter logic** - Filter tasks by inferredStatus in usePRDFilters hook
4. **Add keyboard shortcuts** - Quick filter to "ready" tasks (e.g., Ctrl+R)
5. **Add stats dashboard** - Show count of ready/waiting tasks in header
6. **Dependency graph view** - Visualize task dependency chains

## Testing Scenarios

### Manual Testing

1. **Task with no dependencies:**
   - Status: `pending`
   - Inferred: `ready`
   - ✅ Should show green "Ready" badge

2. **Task with unmet dependencies:**
   - Status: `pending`
   - Depends on: [1, 2] (both pending)
   - Inferred: `waiting_on_deps`
   - ✅ Should show yellow "Waiting on Deps" badge

3. **Task in progress:**
   - Status: `in_progress`
   - Inferred: `in_progress`
   - ✅ Should NOT show inferred badge (redundant)

4. **Task completed:**
   - Status: `done`
   - Inferred: `done`
   - ✅ Should NOT show inferred badge (redundant)

5. **Tooltip on hover:**
   - Hover over "Ready" badge
   - ✅ Should show: "All dependencies met - ready to work on"
   - Hover over "Waiting on Deps" badge
   - ✅ Should show: "Waiting on 2 dependencies"

## See Also

- **`.docs/012_INFERRED_TASK_STATUS_IMPLEMENTATION.md`** - Backend implementation
- **`.docs/011_TASK_MODEL_FOR_CONCURRENT_RALPH_LOOPS.md`** - Full design analysis
- **SPEC-050: Ralph Orchestration Philosophy** - Core principles
