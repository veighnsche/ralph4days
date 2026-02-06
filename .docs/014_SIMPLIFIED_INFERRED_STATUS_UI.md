# Simplified Inferred Status UI

**Created:** 2026-02-06
**Status:** Implemented (Simplified)
**Supersedes:** `.docs/013_INFERRED_STATUS_UI_INTEGRATION.md` (original approach was too crowded)

## Problem with Original Approach

The initial implementation added separate inferred status badges everywhere, making the UI cluttered:

```
❌ TOO CROWDED:
AUTH-004-FRNT Dashboard
[⏱ Waiting on Deps] [2 deps] [High]
  ↑ Redundant!        ↑ Already shows deps!
```

## Simplified Approach

### Principle: Enhance Existing UI, Don't Add More

1. **List Item:** Color the existing deps badge when waiting
2. **Detail View:** Consolidate status into one clean row

## Implementation

### 1. List Item (PlaylistItem)

**Enhanced the existing deps badge** instead of adding a new one:

```tsx
{/* Dependencies Badge - enhanced with inferred status color */}
{task.dependsOn && task.dependsOn.length > 0 && (() => {
  const isWaiting = task.inferredStatus === "waiting_on_deps";
  const inferredConfig = isWaiting ? INFERRED_STATUS_CONFIG.waiting_on_deps : null;

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Badge
          variant="outline"
          className="text-xs px-1.5 py-0.5 h-5 cursor-help"
          style={inferredConfig ? {
            borderColor: inferredConfig.color,
            color: inferredConfig.color,
            backgroundColor: inferredConfig.bgColor,
          } : undefined}
        >
          {task.dependsOn.length} dep{task.dependsOn.length !== 1 ? 's' : ''}
        </Badge>
      </TooltipTrigger>
      <TooltipContent>
        {isWaiting
          ? getInferredStatusExplanation(task.status, task.inferredStatus, task.dependsOn.length)
          : `${task.dependsOn.length} ${task.dependsOn.length === 1 ? 'Dependency' : 'Dependencies'}`
        }
      </TooltipContent>
    </Tooltip>
  );
})()}
```

**Visual Result:**

```
✅ CLEAN:
AUTH-003-FRNT Login form (ready - deps met)
[3 AC] [High]

AUTH-004-FRNT Dashboard (waiting)
[3 AC] [2 deps] [High]
        ↑ Orange/yellow when waiting!
```

### 2. Detail View (TaskDetailTabContent)

**Consolidated status into one row** with inline inferred status:

```tsx
<PropertyRow label="Status">
  <div className="flex flex-col gap-1.5">
    {/* Actual Status */}
    <div className="flex items-center gap-1.5">
      <StatusIcon className="h-3.5 w-3.5" style={{ color: statusConfig.color }} />
      <span className="text-sm" style={{ color: statusConfig.color }}>
        {statusConfig.label}
      </span>
    </div>

    {/* Inferred Status (only if different from actual) */}
    {shouldShowInferredStatus(task.status, task.inferredStatus) && (() => {
      const inferredConfig = INFERRED_STATUS_CONFIG[task.inferredStatus];
      const InferredIcon = inferredConfig.icon;
      const explanation = getInferredStatusExplanation(
        task.status,
        task.inferredStatus,
        task.dependsOn?.length ?? 0
      );

      return (
        <div className="flex items-start gap-1.5 pl-5">
          <span className="text-xs text-muted-foreground mt-0.5">→</span>
          <div className="flex flex-col gap-0.5">
            <div className="flex items-center gap-1.5">
              <InferredIcon className="h-3 w-3" style={{ color: inferredConfig.color }} />
              <span className="text-xs font-medium" style={{ color: inferredConfig.color }}>
                {inferredConfig.label}
              </span>
            </div>
            {explanation && (
              <span className="text-xs text-muted-foreground">{explanation}</span>
            )}
          </div>
        </div>
      );
    })()}
  </div>
</PropertyRow>
```

**Visual Result:**

```
✅ CONSOLIDATED:

Status:  ○ Pending
         → ⏱ Waiting on Deps
            Waiting on 2 dependencies

(Instead of two separate rows)
```

## Visual Comparison

### List Item

**Before (crowded):**
```
AUTH-004-FRNT Dashboard
[⏱ Waiting on Deps] [2 deps] [High]
```

**After (clean):**
```
AUTH-004-FRNT Dashboard
[3 AC] [2 deps] [High]
        ↑ Orange when waiting!
```

### Detail View

**Before (two rows):**
```
Status:           ○ Pending
Computed Status:  ⏱ Waiting on Deps
                  Waiting on 2 dependencies
Priority:         High
```

**After (one row):**
```
Status:  ○ Pending
         → ⏱ Waiting on Deps
            Waiting on 2 dependencies
Priority: High
```

## Key Changes from Original

| Aspect | Original (Doc 013) | Simplified (This) |
|--------|-------------------|-------------------|
| List item | Added separate badge | Enhanced existing deps badge |
| Detail view | Two separate rows | One consolidated row |
| Visual noise | Higher | Lower |
| Information | Same | Same |
| Code complexity | Higher | Lower |

## When Enhancements Show

### List Item - Deps Badge

| Scenario | Badge Style | Tooltip |
|----------|-------------|---------|
| No dependencies | Not shown | N/A |
| Has deps, all done (ready) | Normal outline | "2 Dependencies" |
| Has deps, some pending (waiting) | 🟡 Orange border/text | "Waiting on 2 dependencies" |

### Detail View - Status Row

| Scenario | Display |
|----------|---------|
| In progress | `▶ In Progress` (no arrow) |
| Done | `✓ Done` (no arrow) |
| Pending + ready | `○ Pending` (no arrow) |
| Pending + waiting | `○ Pending → ⏱ Waiting on Deps` |

## Benefits of Simplified Approach

1. **Less visual clutter** - No extra badges
2. **Contextual enhancement** - Color only when needed
3. **Preserves existing layout** - Works with current design
4. **Same information** - Nothing lost
5. **Cleaner code** - Fewer conditionals

## Files Modified

**Components:**
- `src/components/prd/PlaylistItem.tsx:9` - Removed unused import
- `src/components/prd/PlaylistItem.tsx:80-106` - Enhanced deps badge instead of adding new badge
- `src/components/workspace/TaskDetailTabContent.tsx:112-146` - Consolidated status rows

**No changes needed to:**
- Constants (still using `INFERRED_STATUS_CONFIG`)
- Utilities (still using `getInferredStatusExplanation`)
- Types (no changes)

## Implementation Summary

✅ **List item:** Enhanced existing deps badge with color when waiting
✅ **Detail view:** Consolidated two status rows into one with inline inferred status
✅ **Build:** Passes successfully
✅ **Visual:** Cleaner, less crowded

## See Also

- **`.docs/012_INFERRED_TASK_STATUS_IMPLEMENTATION.md`** - Backend implementation
- **`.docs/011_TASK_MODEL_FOR_CONCURRENT_RALPH_LOOPS.md`** - Full design analysis
