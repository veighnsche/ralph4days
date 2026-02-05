# Filters Modal Implementation

## Overview

Replaced inline filter dropdowns with a cleaner modal-based approach that shows active filters as removable badges.

## Components Created

### 1. FiltersModal.tsx (`src/components/prd/FiltersModal.tsx`)

A Dialog-based modal that contains all filter options:
- Search input (text)
- Status filter (select: all, blocked, in_progress, pending, done, skipped)
- Priority filter (select: all, critical, high, medium, low)
- Tag filter (select: all, or any existing tag)
- Clear All button (disabled when no active filters)
- Done button (closes modal)

**Features:**
- Button shows indicator dot when filters are active
- Button style changes based on active state (default vs outline)
- Uses shadcn Dialog components for consistent modal behavior

### 2. ActiveFilters.tsx (`src/components/prd/ActiveFilters.tsx`)

Displays active filters as removable badges:
- Each filter shown as a Badge with label and X button
- Clicking X removes that specific filter
- Auto-formats filter values (e.g., "in_progress" → "In Progress")
- Conditionally renders (hidden when no active filters)

**Badge format:**
- Search: `Search: "query"`
- Status: `Status: Pending`
- Priority: `Priority: High`
- Tag: `Tag: backend`

## Changes to Existing Components

### PRDHeader.tsx

**Removed:**
- Inline Status, Priority, and Tag select dropdowns
- Filter icon import
- Clear Filters button (moved to modal)

**Added:**
- FiltersModal button (replaces inline dropdowns)
- ActiveFilters component (shows below filter row when active)

**Kept:**
- Search input (common pattern to have search separate)
- "Showing X of Y" counter

## File Structure

```
src/components/prd/
├── ActiveFilters.tsx      (new)
├── FiltersModal.tsx       (new)
├── PRDHeader.tsx          (updated)
└── PRDHeader.stories.tsx  (updated - fixed filter interfaces)
```

## User Flow

1. Click "Filters" button to open modal
2. Set any combination of filters
3. Click "Done" or close modal
4. Active filters appear as badges below the search bar
5. Click X on any badge to remove that filter
6. Open modal and click "Clear All" to reset all filters

## Notes

- Filter state managed by existing `usePRDFilters` hook (no changes)
- Fully compatible with existing FilterState/FilterSetters interfaces
- Uses existing shadcn components (Dialog, Badge, Select, Input, Label)
- Stories updated to match new filter interfaces
