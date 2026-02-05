# Page Architecture

## Overview

Proper page abstraction with self-contained pages, each having their own header and body. Clean separation of concerns with no shared headers.

## Architecture

```
App.tsx
├── Left Panel (Pages with BottomBar)
│   ├── TasksPage (self-contained)
│   ├── FeaturesPage (self-contained)
│   ├── DisciplinesPage (self-contained)
│   └── BottomBar (navigation + transport controls)
└── Right Panel (OutputPanel - always visible)
```

## Pages

### TasksPage (`src/pages/TasksPage.tsx`)
**Self-contained task management page**

Components:
- PRDHeader - Project stats, progress, filters, new task button
- PRDBody - Task list (PlaylistView)
- TaskDetailSidebar - Task detail panel

Features:
- Full task filtering (search, status, priority, tags)
- Active filter badges
- Task creation
- Task detail view with navigation

### FeaturesPage (`src/pages/FeaturesPage.tsx`)
**Self-contained features overview page**

Header:
- Features icon + title
- Total features count
- Done/remaining task counts
- Overall progress percentage
- Progress bar

Body:
- List of features with:
  - Feature name and description
  - Task count badge
  - Done/in-progress/pending breakdown
  - Progress percentage per feature

### DisciplinesPage (`src/pages/DisciplinesPage.tsx`)
**Self-contained disciplines overview page**

Header:
- Disciplines icon + title
- Total disciplines count
- Done/remaining task counts
- Overall progress percentage
- Progress bar

Body:
- List of disciplines with:
  - Colored icon
  - Discipline name
  - Task count badge
  - Done/in-progress/pending breakdown
  - Progress percentage per discipline

## Navigation

- **Menu button** (☰) on bottom-left of BottomBar
- Opens dropdown with 3 options:
  - 📝 Tasks
  - 🎯 Features
  - 📚 Disciplines
- Current page indicated with dot (•)
- State managed in App.tsx, passed as props

## Layout Characteristics

### Consistent Elements
- Right panel: Always shows OutputPanel
- Bottom bar: Always present with navigation menu + transport controls

### Page-Specific Elements
- Each page has its own tailored header
- Each page manages its own loading/error states
- Each page has its own body layout

## File Structure

```
src/
├── pages/
│   ├── TasksPage.tsx          (tasks page with PRD header)
│   ├── FeaturesPage.tsx       (features page with custom header)
│   └── DisciplinesPage.tsx    (disciplines page with custom header)
├── components/
│   ├── BottomBar.tsx          (navigation + transport controls)
│   ├── NavigationMenu.tsx     (page navigation dropdown)
│   └── prd/
│       ├── PRDHeader.tsx      (used by TasksPage only)
│       ├── PRDBody.tsx        (used by TasksPage only)
│       └── TaskDetailSidebar.tsx (used by TasksPage only)
└── App.tsx                     (page switching logic)
```

## Key Improvements

1. ✅ **Proper abstraction** - Each page is a complete, self-contained component
2. ✅ **No shared headers** - Features and Disciplines have their own tailored headers
3. ✅ **Clean separation** - Pages don't depend on each other
4. ✅ **Consistent layout** - Right panel (output) always visible
5. ✅ **State management** - Navigation state in App.tsx, passed as props
6. ✅ **No window hacks** - Proper React props flow
