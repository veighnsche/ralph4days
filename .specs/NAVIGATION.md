# Navigation System

## Overview

Added navigation between three pages (Tasks, Features, Disciplines) with a menu button on the bottom bar.

## Architecture

### Navigation State
- `useNavigation` hook manages current page state
- Page type: `"tasks" | "features" | "disciplines"`
- Shared via window object between PRDViewer and BottomBar

### Components

#### NavigationMenu (`src/components/NavigationMenu.tsx`)
- DropdownMenu component with three options
- Shows current page with indicator dot
- Positioned in BottomBar (left side)
- Icons: ListTodo (Tasks), Target (Features), Layers (Disciplines)

#### FeaturesPage (`src/components/pages/FeaturesPage.tsx`)
- Lists all features with statistics
- Shows task counts (total, done, in progress, pending)
- Displays progress percentage per feature
- Fetches data via `get_features` IPC command

#### DisciplinesPage (`src/components/pages/DisciplinesPage.tsx`)
- Lists all disciplines with task distribution
- Shows discipline icon, color, and statistics
- Displays progress percentage per discipline
- Uses existing `useDisciplines` hook

## Backend Changes

### New Tauri Command: `get_features`
- Returns full feature data (name, display_name, description, created)
- Registered in `src-tauri/src/lib.rs`
- Added `FeatureData` struct in `commands.rs`

## File Structure

```
src/
├── hooks/
│   └── useNavigation.ts               (new)
├── components/
│   ├── NavigationMenu.tsx             (new)
│   ├── BottomBar.tsx                  (updated - added nav menu)
│   └── pages/
│       ├── FeaturesPage.tsx           (new)
│       └── DisciplinesPage.tsx        (new)
│   └── prd/
│       └── PRDViewer.tsx              (updated - page switching)
├── types/
│   └── prd.ts                         (updated - added Feature type)

src-tauri/src/
├── commands.rs                        (updated - added get_features)
└── lib.rs                             (updated - registered get_features)
```

## User Flow

1. Click menu button (☰) on bottom left
2. Select page from dropdown (Tasks, Features, or Disciplines)
3. Page switches in main content area
4. Current page is indicated with dot in menu

## Notes

- Navigation state shared via window object (simple, no context needed)
- Features and Disciplines pages use existing PRD data for statistics
- All three pages share the same header (project info, progress bar)
- Task detail sidebar only available on Tasks page
- Filters only apply to Tasks page
