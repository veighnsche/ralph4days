# 090 UX Terminology Glossary

This document captures the UX terminology used during the iOS SSH screen redesign and test harness iteration.

## Core Interaction Terms

- **Hot reload**: Immediate frontend refresh while the app is running in dev mode.
- **HMR (Hot Module Replacement)**: Partial in-place update of changed modules without full app restart.
- **Native feel**: Interaction behavior expected from iOS system apps (momentum scroll, bounce, drag-dismiss sheets).
- **Determinism**: Repeatable behavior and state setup for tests.

## Mobile Scrolling Terms

- **Scroll bounce / Rubber-band**: iOS spring effect when pulling past top or bottom.
- **Always-bounce vertical**: Bounce is available even when content is shorter than viewport.
- **Scroll lock / stuck scroll**: User cannot return to top or continue scrolling after keyboard/viewport transition.
- **Momentum scrolling**: Inertial scrolling (`-webkit-overflow-scrolling: touch`).
- **Dynamic viewport height (`dvh`)**: Viewport unit that tracks keyboard and browser UI changes.
- **Small viewport height (`svh`)**: Static smaller viewport unit; can break post-keyboard layouts.

## Bottom Sheet / Drawer Terms

- **Bottom sheet**: Panel emerging from the bottom of screen.
- **Drawer**: Implementation primitive used for draggable bottom sheet behavior.
- **Drag-to-dismiss**: Closing sheet by swiping it downward.
- **Grab handle**: Thin line at top of sheet indicating it is draggable.
- **Close affordance**: Visual cue that indicates how to dismiss a surface.

## Navigation and Density Terms

- **Declutter pass**: Reduce on-screen controls to improve readability and thumb ergonomics.
- **Primary action**: Most important action kept visible (e.g., `Connect`).
- **Secondary actions**: Lower-priority actions hidden behind a menu/sheet (e.g., `Edit`, `Delete`, `Set Default`).
- **Overflow actions**: Actions exposed behind a `More` trigger.
- **Icon-first control**: Compact interaction entry point (e.g., search icon toggle).

## Search UX Terms

- **Search behind icon**: Search input hidden by default and revealed through icon tap.
- **Inline expand**: Search field expands within current layout instead of opening a separate screen.
- **Search clear**: Explicit control to clear query text.

## SSH-Specific UX Terms

- **SSH profile**: Saved host/credential configuration for connecting.
- **Default SSH profile**: Preferred profile used first when no explicit active profile is set.
- **Quick connect**: Sticky shortcut button for fastest connect/reconnect.
- **Reconnect**: Connect action label when an SSH session already exists.
- **Host key verification**: Trust workflow for unknown host fingerprints.

## Testing Terms

- **Stage tests**: Numbered, ordered test files for specific setup/connect flows.
- **Precondition / prerequisite**: Required prior state before a stage can run.
- **Isolation run**: Running one stage independently.
- **State preserve mode**: Keep app install/data between test runs.
- **Reset mode**: Force clean app state before run.
- **Harness**: Shared test helper layer used by e2e specs.

## UI Primitive Terms Introduced

- **`MobileScrollPage`**: Shared mobile page container for stable scroll + bounce behavior.
- **`DottedList` / `DottedListItem`**: New bullet-list primitives for concise explanatory copy.

## Naming Notes

- “Default behavior” should be unnamed, while non-default behavior should be explicitly named.
- Prefer concrete labels (`Reconnect`, `Set as Default`) over ambiguous verbs.
- Keep test IDs stable and action-oriented (`ssh-profile-action-delete-<id>`).
