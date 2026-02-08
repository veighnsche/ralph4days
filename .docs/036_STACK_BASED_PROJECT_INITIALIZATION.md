# Stack-Based Project Initialization

**Date:** 2026-02-08
**Status:** Implemented (Stacks 0-2 only)

## Overview

Ralph now supports **3 tech stack presets** when adding `.ralph/` to existing projects. Each preset seeds a different set of disciplines tailored to specific development workflows.

## Implementation

### Backend (Rust)

**crates/sqlite-db/src/disciplines.rs:**
- Added `seed_for_stack(stack: u8)` method
- `seed_defaults()` now calls `seed_for_stack(2)` for backward compatibility
- Stack 1 disciplines are **inlined** (full system prompts, skills, conventions in code)
- Stack 2 disciplines use existing `include_str!()` from `defaults/disciplines/` folder

**src-tauri/src/commands/project.rs:**
- Updated `initialize_ralph_project` to accept `stack: u8` parameter
- Changed from `db.seed_defaults()` to `db.seed_for_stack(stack)`

### Frontend (TypeScript/React)

**src/components/ProjectSelector.tsx:**
- Added stack selector dropdown with 3 options
- Default: Stack 2 (Tauri + React)
- Updated labels: "Initialize Existing Project" → "Add Ralph to Existing Project"
- Pass `stack` parameter to `initialize_ralph_project` command

### Test Updates

**crates/sqlite-db/tests/crud_operations.rs:**
- Updated `test_seed_defaults` to expect 8 disciplines (was 10)
- Updated assertions to check for Stack 2 disciplines

## Stack Definitions

### Stack 0: Empty
- **Disciplines:** None
- **Use case:** Complete freedom, custom disciplines created via braindump/discuss
- **Philosophy:** User builds their own workflow

### Stack 1: Generic
- **Disciplines:** 8 mode-based (language-agnostic)
  1. Implementation 🔨 (Hammer)
  2. Refactoring ♻️ (Recycle)
  3. Investigation 🔍 (Search)
  4. Testing ✅ (CheckCircle)
  5. Architecture 📐 (Compass)
  6. DevOps 🚀 (Rocket)
  7. Security 🔒 (Shield)
  8. Documentation 📚 (BookOpen)
- **Use case:** Any codebase, any language
- **Philosophy:** Disciplines are **work modes** (how you work), not tech-specific

### Stack 2: Tauri + React (Default)
- **Disciplines:** 8 tech-specific
  1. Frontend (Monitor)
  2. Backend (Server)
  3. Data (Database)
  4. Platform (Cloud)
  5. Quality (FlaskConical)
  6. Security (Shield)
  7. Integration (Cable)
  8. Documentation (BookOpen)
- **Use case:** Desktop apps with Rust backend, React frontend
- **Philosophy:** Optimized for Tauri + React + TypeScript + SQLite stack

## Design Notes

### Why Inline Stack 1?

Stack 1 disciplines are **inlined directly in code** rather than using `include_str!()` files for several reasons:
1. **Portability** — Generic disciplines are fundamental, should travel with the code
2. **Transparency** — Easier to see all discipline definitions in one place during code review
3. **Fewer files** — Reduces filesystem dependency for core presets

Stack 2 uses `include_str!()` because those disciplines are **more complex** (longer prompts, more skills) and benefit from separate maintenance files.

### Why 8 Disciplines Per Stack?

Both Stack 1 and Stack 2 have exactly 8 disciplines:
- Manageable cognitive load (user can remember them)
- Covers core development concerns
- Not too broad (avoids "everything discipline")
- Not too narrow (avoids excessive task routing complexity)

### Future Stacks

**Stack 3: Next.js SaaS** (designed, not implemented)
- 12 disciplines including fullstack, integration, performance
- Optimized for Turborepo + Vercel + tRPC + Prisma

**Stack 4: Flutter Mobile** (designed, not implemented)
- 10 disciplines including mobile-specific concerns
- Optimized for Dart + Flutter + Firebase

See `.docs/024_TECH_STACK_PRESETS_AND_DISCIPLINE_STACKS.md` for full design specifications.

## User Flow

1. User runs `ralph` (no --project flag)
2. ProjectSelector modal appears with two columns:
   - **Left:** "Add Ralph to Existing Project" (with stack selector)
   - **Right:** "Open Existing Project" (scans for `.ralph/` folders)
3. User selects stack from dropdown (0: Empty, 1: Generic, 2: Tauri + React)
4. User browses to existing project directory
5. Ralph creates `.ralph/db/ralph.db` and seeds selected disciplines
6. Ralph creates `.ralph/CLAUDE.RALPH.md` template
7. Project is locked and Ralph starts

## Code Pattern: Type Alias for Complex Tuples

Clippy complained about the complex tuple type for discipline seeds:
```rust
// ❌ WRONG: clippy::type_complexity warning
let defaults: Vec<(
    &str, &str, &str, &str, &str, &str, &str, &str,
)> = match stack { ... }
```

Solution: Use type alias
```rust
// ✅ CORRECT
type DisciplineSeed = (
    &'static str, &'static str, &'static str, &'static str,
    &'static str, &'static str, &'static str, &'static str,
);

let defaults: Vec<DisciplineSeed> = match stack { ... }
```

This pattern is now standard in CLAUDE.md coding patterns.

## Testing

All tests pass:
- `cargo test --workspace` (155 tests, all pass)
- `bun tsc --noEmit` (no errors)
- `bun biome check` (no issues)
- Fixtures regenerated with new Stack 2 seeding
