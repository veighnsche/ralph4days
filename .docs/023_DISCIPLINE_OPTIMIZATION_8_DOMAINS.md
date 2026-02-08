# Discipline Optimization: 8 Domain-Based Disciplines

**Date:** 2026-02-08
**Status:** Implemented

## Summary

Redesigned Ralph's discipline system from 10 vague/overlapping disciplines to 8 focused domain-based disciplines with rich defaults (system_prompt, skills, conventions).

## The Problem

**Old 10 disciplines:**
- frontend, backend, wiring, database, testing, infra, security, docs, design, api
- Most had no system_prompt, skills, or conventions (empty shells)
- Unclear boundaries: "wiring" vs "integration", "api" vs "backend"
- "design" was ambiguous (UI design? System architecture?)
- Only 2-3 disciplines actually used in practice

**Core insight:** Disciplines define Claude Code's **execution environment** (MCP servers, expertise, mindset), not work modes. Tasks naturally have different types (refactor, debug, implement) through their descriptions.

## The Solution

**New 8 disciplines** (domain-based):

1. **frontend** - UI, components, client-side (React, TypeScript, Tailwind)
2. **backend** - Server logic, APIs, services
3. **data** - Databases, schemas, queries, migrations
4. **platform** - Infrastructure, deployment, CI/CD, tooling
5. **quality** - Testing, reliability, QA
6. **security** - Auth, vulnerabilities, secure coding
7. **integration** - Third-party APIs, webhooks, inter-service communication
8. **documentation** - READMEs, guides, API docs

**Changes from old 10:**
- ❌ Removed: design (merged into frontend), api (merged into backend)
- ✅ Renamed: database→data, infra→platform, testing→quality, wiring→integration, docs→documentation
- ✅ Added rich defaults for all 8

## Implementation

### File Structure

```
crates/sqlite-db/src/defaults/disciplines/
├── frontend/
│   ├── system_prompt.txt (500+ word persona)
│   ├── skills.json (10 skills array)
│   └── conventions.txt (naming, structure, patterns, quality)
├── backend/
│   ├── system_prompt.txt
│   ├── skills.json
│   └── conventions.txt
... (8 total)
```

### Seeding Logic

Updated `disciplines.rs::seed_defaults()`:
- Loads text files via `include_str!()` at compile time
- Inserts system_prompt, skills, conventions into database on init
- Only creates if discipline doesn't exist (safe for existing projects)

### Default Content

Each discipline includes:

**System Prompt (500+ words):**
- Expertise areas (core tech, practices, knowledge domains)
- Approach (design first, build reliably, optimize wisely)
- Priorities (ordered list of what matters most)

**Skills (10-15 items):**
- JSON array of concrete capabilities
- Example: `["React 19", "TypeScript", "Accessibility", ...]`

**Conventions (structured text):**
- Naming (components, files, functions)
- Structure (directory organization)
- Patterns (code patterns to follow)
- Quality standards (rules and metrics)

## User Customization

Users can:
1. **Use defaults as-is** - Start creating tasks immediately
2. **Customize existing** - Use "Discuss" prompt or UI to update disciplines
3. **Create new disciplines** - Via braindump/discuss prompts or UI
4. **Delete unused** - Remove disciplines they don't need

Disciplines are **project-specific** - each project has its own set stored in `.ralph/db/ralph.db`.

## Migration Path

**Existing projects** (with old 10 disciplines):
- Old disciplines remain unchanged
- `seed_defaults()` only creates disciplines that don't exist
- User can manually delete old disciplines and restart to get new 8
- Or keep using old ones - no breaking change

**New projects:**
- Get the new 8 disciplines with full defaults automatically

## Next Steps

1. ✅ Create default text files for all 8 disciplines
2. ✅ Update `seed_defaults()` to use default files
3. 🔄 Update MCP builder subsystem in prompt-builder crate
4. 📝 Update fixtures to use new 8 disciplines
5. 📝 Update docs (CLAUDE.md, README.md) to reference new disciplines
6. 📝 UI updates (discipline picker, discipline management)

## Files Changed

- `crates/sqlite-db/src/defaults/disciplines/*/` - New default files (24 files)
- `crates/sqlite-db/src/disciplines.rs` - Updated seed_defaults()
- `.docs/023_DISCIPLINE_OPTIMIZATION_8_DOMAINS.md` - This doc

## Design Philosophy

**Disciplines = Domain Environment**
- Each discipline provides domain-specific tooling (MCP servers)
- Each discipline has domain-specific expertise (system_prompt, skills)
- Each discipline enforces domain-specific practices (conventions)

**Tasks = Work Items**
- Tasks naturally have different types (refactor, debug, implement, investigate)
- The task description/title/tags indicate the work mode
- No need for explicit "mode" field - it's emergent from task content

**Benefits:**
- Clear separation: discipline = toolset/expertise, task = work to do
- Simpler model: one dimension (discipline), not two (discipline + mode)
- Extensible: users can add more disciplines for their specific needs
- Rich defaults: Claude gets real expertise, not empty shells
