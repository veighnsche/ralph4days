# Mobile-First Responsive Frontend Dump

Date: 2026-02-15

## Goal
Treat the React frontend as mobile-first (small viewport first), while preserving the existing 2-column desktop UI.

Constraint from current product:
- The app's primary locked-project UI is a 2-column split: main content (left) + workspace (right).
- On phone-sized viewports, both columns cannot be simultaneously legible.

## Findings (Before Changes)
1. `src/App.tsx` hard-coded a desktop split view:
   - `ResizablePanelGroup` was always `orientation="horizontal"` with 2 side-by-side panels.
   - On narrow widths this effectively crushes both panes.
2. `src/components/app-shell/ProjectSelector.tsx` was desktop-only layout:
   - 3-column grid (`grid-cols-[1fr_auto_1fr]`) with large padding.
   - Not viable for phone-sized viewports.
3. Several header rows assumed horizontal space:
   - `src/components/prd/PRDHeader.tsx` top stats + search/filters were single-row flex layouts.
   - `src/pages/SubsystemsPage.tsx` and `src/pages/DisciplinesPage.tsx` headers packed multiple stats into a single row.
4. Task list headshot overlay used substantial horizontal space:
   - `src/components/prd/PlaylistItem.tsx` shows a large left-side image overlay and shifts the ID column (`ml-22`).
   - On very small screens this is a readability tax.

## Decisions
1. Viewport-based "mobile" is defined as `< 768px` (matches Tailwind `md` and existing `useIsMobile()` breakpoint).
2. Mobile-first behavior:
   - Collapse the split view into a single-pane UI.
   - Provide a deterministic way to switch between "Main" and "Workspace" on small viewports.
3. Desktop behavior remains unchanged for `>= md`:
   - Keep the resizable 2-column split view.

## Implementation Summary
1. Mobile-first app shell (single-pane + toggle)
   - `src/App.tsx`
   - For small viewports:
     - Render either the main page content or the workspace, not both at once.
     - Add a Workspace toggle button into the bottom bar (right side).
     - When the user changes pages, force the active pane back to "Main" (prevents "I changed pages but I'm still staring at Workspace").
   - For `md+`:
     - Keep the existing `ResizablePanelGroup` desktop split.

2. Bottom bar extensibility
   - `src/components/app-shell/BottomBar.tsx`
   - Added optional `rightActions` slot so mobile can inject the Workspace toggle without forking the component.

3. Project selector becomes responsive
   - `src/components/app-shell/ProjectSelector.tsx`
   - `grid-cols-1` on small screens, stacks: project lists -> separator -> init panel.
   - Restores the original 3-column split on `md+`.
   - Padding reduced on small screens.

4. Header rows wrap/stack on small screens
   - `src/components/prd/PRDHeader.tsx`: stacks title/stats and search/filter row on small screens.
   - `src/components/prd/TaskStatsBar.tsx`: allows wrapping.
   - `src/pages/SubsystemsPage.tsx`, `src/pages/DisciplinesPage.tsx`: header stats wrap on small screens.

5. Task list headshot is desktop-first, not phone-first
   - `src/components/prd/PlaylistItem.tsx`
   - The left-side discipline headshot is hidden on `< sm` and the ID column shift only applies at `sm+`.

## Acceptance Checklist
- [ ] At widths `< 768px`, the app never renders the 2-column split; a single pane fully fits the viewport.
- [ ] The user can always reach Workspace on small screens via the bottom-bar toggle.
- [ ] Switching pages while on Workspace returns to the main pane (no "stuck in workspace" confusion).
- [ ] Project picker is usable on phone-sized widths (no forced side-by-side columns).
- [ ] Task list remains readable on phone-sized widths (no giant headshot stealing horizontal space).

## Follow-Ups (Not Implemented Here)
- Responsive behavior for other dense multi-pane UIs (notably the full-screen Prompt Builder editor) may still need a dedicated mobile layout.
