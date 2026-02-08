# Discipline Model Completion - Backend

**Date:** 2026-02-08
**Status:** Backend Complete, Frontend Pending

## Summary

Completed backend support for rich discipline fields (system_prompt, skills, conventions, mcp_servers) in create/update operations. The disciplines model now fully supports all fields from database to IPC.

## Context

While the database schema already had rich fields and `seed_defaults()` could populate them, the create/update operations only supported basic fields (name, display_name, acronym, icon, color). This prevented users from creating or editing disciplines with custom expertise.

## Changes Made

### 1. Database Operations (`crates/sqlite-db/src/disciplines.rs`)

**Updated `create_discipline`** (lines 6-68):
- Added parameters: `system_prompt: Option<String>`, `skills: String`, `conventions: Option<String>`, `mcp_servers: String`
- Updated INSERT statement to include all 9 fields
- Skills and mcp_servers are JSON strings (validated as arrays)

**Updated `update_discipline`** (lines 70-123):
- Added same rich field parameters
- Updated UPDATE statement to set all 8 fields (excluding name which is PK)
- Skills and mcp_servers are JSON strings

### 2. Tauri Commands (`src-tauri/src/commands/features.rs`)

**Updated `McpServerConfigData`** (lines 7-15):
- Added `serde::Deserialize` derive (was only Serialize)
- Needed for deserialization from frontend

**Created `CreateDisciplineParams`** (lines 207-217):
- Struct for create_discipline parameters
- Rich fields: `system_prompt`, `skills`, `conventions`, `mcp_servers`
- Skills: `Option<Vec<String>>` (deserialized from JSON array)
- MCP servers: `Option<Vec<McpServerConfigData>>` (deserialized from JSON array)

**Updated `create_discipline` command** (lines 219-253):
- Accepts `CreateDisciplineParams` instead of individual parameters
- Serializes `skills` Vec to JSON string
- Converts `McpServerConfigData` to `sqlite_db::McpServerConfig`
- Serializes `mcp_servers` Vec to JSON string
- Passes all fields to database

**Created `UpdateDisciplineParams`** (lines 263-273):
- Same structure as CreateDisciplineParams

**Updated `update_discipline` command** (lines 275-309):
- Accepts `UpdateDisciplineParams` instead of individual parameters
- Same serialization logic as create_discipline
- Passes all fields to database

### 3. Type Safety

**Frontend → Backend flow:**
```
Frontend sends:
{
  name: "frontend",
  displayName: "Frontend",
  acronym: "FRNT",
  icon: "Monitor",
  color: "#3b82f6",
  systemPrompt: "You are a frontend specialist...",
  skills: ["React 19", "TypeScript", ...],
  conventions: "## Naming\n- Components: PascalCase\n...",
  mcpServers: [
    { name: "shadcn-ui", command: "npx", args: [...], env: {} },
    ...
  ]
}

Tauri command deserializes to:
CreateDisciplineParams {
  skills: Option<Vec<String>>,
  mcp_servers: Option<Vec<McpServerConfigData>>,
  ...
}

Command serializes to JSON strings:
skills_json: String = '["React 19", "TypeScript", ...]'
mcp_json: String = '[{"name":"shadcn-ui",...}]'

Database stores:
skills TEXT = '["React 19", "TypeScript", ...]'
mcp_servers TEXT = '[{"name":"shadcn-ui",...}]'
```

## Verification

**Compilation:** ✅ `cargo check --workspace` passes
**Tests:** ✅ All 76 workspace tests pass
- prompt-builder: 15 tests
- ralph-rag: 28 tests
- ralph4days: 20 tests
- sqlite-db: 13 tests

## What's Still Missing

### Frontend UI (Next Priority)

1. **No discipline UI components** - Grep found zero React components for disciplines
2. **Type generation** - Need to generate TypeScript types from Rust structs
3. **Components needed:**
   - Discipline list view (read-only for now)
   - Discipline detail/edit modal with tabs:
     - Basic Info: display_name, acronym, icon, color
     - System Prompt: textarea
     - Skills: array editor (add/remove strings)
     - Conventions: textarea
     - MCP Servers: JSON editor or structured form

### Project Scaffolding (Future)

1. **Stack preset selection** - Choose from 5 stacks (Empty, Generic, Tauri+React, Next.js, Flutter)
2. **Scaffold entire projects** - Generate full project structure
3. **Add .ralph/ to existing** - Initialize Ralph in existing codebases

## Files Changed

- `crates/sqlite-db/src/disciplines.rs` - Updated create/update to accept rich fields
- `src-tauri/src/commands/features.rs` - Updated commands to accept rich fields, added Deserialize

## Next Steps

1. Generate TypeScript types for DisciplineConfig
2. Create frontend discipline management UI
3. Test end-to-end create/update with all fields
4. Add stack preset selection to project initialization
