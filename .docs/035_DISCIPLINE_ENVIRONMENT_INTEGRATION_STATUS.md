# Discipline Environment Integration - Status Report

**Date:** 2026-02-08
**Status:** Mostly Complete, Needs Task Execution Integration

## Summary

Disciplines ARE fully integrated into Ralph's prompt-builder system. System prompts, skills, conventions, and MCP servers are all being used. However, there's a gap: **task execution doesn't set the target task**, so disciplines don't activate during actual task execution.

## What's Already Working

### 1. Prompt Builder Integration (✅ COMPLETE)

**File: `crates/prompt-builder/src/sections/discipline_persona.rs`**

The `discipline_persona` section:
- Reads `discipline.system_prompt`
- Formats as "## You Are\n\n{system_prompt}"
- Lists skills under "### Skills"
- Includes conventions under "### Conventions"

```rust
fn build(ctx: &PromptContext) -> Option<String> {
    let discipline = ctx.target_task_discipline()?;
    let system_prompt = discipline.system_prompt.as_ref()?;

    let mut out = format!("## You Are\n\n{system_prompt}");

    if !discipline.skills.is_empty() {
        out.push_str("\n\n### Skills\n\n");
        for skill in &discipline.skills {
            out.push_str(&format!("- {skill}\n"));
        }
    }

    if let Some(conventions) = &discipline.conventions {
        out.push_str(&format!("\n### Conventions\n\n{conventions}"));
    }

    Some(out)
}
```

### 2. MCP Server Generation (✅ COMPLETE)

**File: `crates/prompt-builder/src/mcp/mod.rs` (lines 176-203)**

MCP config generation includes discipline MCP servers:

```rust
// Discipline-specific MCP servers (for TaskExecution prompts)
if let Some(discipline) = ctx.target_task_discipline() {
    for mcp in &discipline.mcp_servers {
        let name = json_escape(&mcp.name);
        let command = json_escape(&mcp.command);
        let args: Vec<String> = mcp.args.iter()
            .map(|a| format!("\"{}\"", json_escape(a)))
            .collect();
        // ... generates JSON config
        servers.push(server);
    }
}
```

### 3. Claude Code Launcher (✅ COMPLETE)

**File: `src-tauri/src/terminal/manager.rs` (lines 96-112)**

Claude CLI is launched with MCP config:

```rust
let mut cmd = CommandBuilder::new("claude");
cmd.cwd(working_dir);
cmd.args(["--permission-mode", "bypassPermissions"]);
cmd.arg("--verbose");
cmd.arg("--no-chrome");

if let Some(model) = &config.model {
    cmd.args(["--model", model]);
}

let settings_json = build_settings_json(&config);
cmd.args(["--settings", &settings_json]);

if let Some(mcp_config) = mcp_config {
    cmd.args(["--mcp-config", &mcp_config.to_string_lossy()]);
}
```

### 4. Context Building (✅ Loads Disciplines)

**File: `src-tauri/src/commands/state.rs` (lines 52-96)**

```rust
pub(super) fn build_prompt_context(...) -> Result<PromptContext, String> {
    // ...
    Ok(PromptContext {
        features: db.get_features(),
        tasks: db.get_tasks(),
        disciplines: db.get_disciplines(),  // ✅ Disciplines loaded
        metadata: db.get_project_info(),
        // ...
        target_task_id: None,  // ⚠️ NOT SET - this is the problem
        // ...
    })
}
```

## The Missing Piece

### Problem: Target Task Not Set

**Current flow:**
1. User creates PTY session for "task_creation" or "discuss" mode
2. `generate_mcp_config(mode, project_path)` is called
3. `build_prompt_context()` is called **WITHOUT** a target task
4. MCP config generated without discipline-specific servers
5. Claude Code launches, but discipline environment is NOT activated

**What's missing:**
- When executing a **specific task**, need to:
  1. Set `target_task_id` in PromptContext
  2. Generate MCP config WITH that task's discipline MCP servers
  3. Generate prompt WITH that task's discipline system_prompt/skills/conventions

### Current MCP Config Generation

**File: `src-tauri/src/commands/state.rs` (lines 98-150)**

```rust
pub(super) fn generate_mcp_config(
    &self,
    mode: &str,  // "task_creation" or other
    project_path: &std::path::Path,
) -> Result<PathBuf, String> {
    let prompt_type = match mode {
        "task_creation" => prompt_builder::PromptType::Braindump,
        _ => prompt_builder::PromptType::Discuss,
    };

    // ... load instruction overrides ...

    let recipe = prompt_builder::recipes::get(prompt_type);
    let ctx = self.build_prompt_context(project_path, None, overrides)?;
    //                                                    ^^^^ No target task!

    let (scripts, config_json) = prompt_builder::mcp::generate(&ctx, &recipe.mcp_tools);

    // ... write scripts and config to disk ...

    Ok(config_path)
}
```

## What Needs to Be Done

### 1. Task Execution Flow

Need a **new flow** for executing specific tasks:

```rust
// NEW METHOD NEEDED
pub(super) fn generate_mcp_config_for_task(
    &self,
    task_id: u32,
    project_path: &std::path::Path,
) -> Result<PathBuf, String> {
    let mut ctx = self.build_prompt_context(project_path, None, HashMap::new())?;
    ctx.target_task_id = Some(task_id);  // ✅ Set target task!

    let recipe = prompt_builder::recipes::get(PromptType::TaskExecution);
    let (scripts, config_json) = prompt_builder::mcp::generate(&ctx, &recipe.mcp_tools);

    // ... write to disk ...

    Ok(config_path)
}
```

### 2. Create PTY Session for Task

```rust
// NEW TAURI COMMAND NEEDED
#[tauri::command]
pub fn create_pty_session_for_task(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    task_id: u32,
    model: Option<String>,
    thinking: Option<bool>,
) -> Result<(), String> {
    let locked = state.locked_project.lock()?;
    let project_path = locked.as_ref()?.clone();
    drop(locked);

    // Generate MCP config with discipline-specific servers
    let mcp_config = state.generate_mcp_config_for_task(task_id, &project_path)?;

    let config = SessionConfig { model, thinking };

    state.pty_manager.create_session(
        app,
        session_id,
        &project_path,
        Some(mcp_config),  // ✅ MCP config with discipline servers
        config
    )
}
```

### 3. Generate Prompt for Task

When Claude Code starts, send initial prompt:

```rust
// After PTY session created for task
let mut ctx = state.build_prompt_context(&project_path, None, HashMap::new())?;
ctx.target_task_id = Some(task_id);

let recipe = prompt_builder::recipes::get(PromptType::TaskExecution);
let prompt = prompt_builder::execute(&recipe, &ctx);

// Send prompt to Claude Code via PTY
state.pty_manager.send_input(&session_id, prompt.as_bytes())?;
```

## Architecture Flow

### Current Flow (Braindump/Discuss)
```
User clicks "Create Tasks"
  ↓
create_pty_session(mcp_mode="task_creation")
  ↓
generate_mcp_config(mode)
  ├─ build_prompt_context(target_task_id=None)
  ├─ get recipe (Braindump or Discuss)
  ├─ generate MCP config (Ralph tools only, NO discipline servers)
  └─ write to disk
  ↓
PTY spawns: claude --mcp-config /path/to/config.json
  ↓
Claude Code launches with Ralph MCP tools
User types braindump/discusses features
```

### Needed Flow (Task Execution)
```
User clicks "Execute Task 123"
  ↓
create_pty_session_for_task(task_id=123)
  ↓
generate_mcp_config_for_task(task_id)
  ├─ build_prompt_context(target_task_id=Some(123))
  ├─ get recipe (TaskExecution)
  ├─ generate MCP config:
  │    ├─ Ralph tools
  │    └─ Discipline-specific MCP servers ✅
  └─ write to disk
  ↓
PTY spawns: claude --mcp-config /path/to/config-task-123.json
  ↓
Claude Code launches with:
  - Ralph MCP tools
  - Discipline MCP servers (shadcn-ui, tailwindcss, etc.)
  ↓
Send initial prompt to Claude:
  - Task description
  - Acceptance criteria
  - Discipline system_prompt ("## You Are...")
  - Discipline skills list
  - Discipline conventions
  ↓
Claude executes task with full discipline environment
```

## Claude Code CLI Flags

**Current flags used:**
- `--permission-mode bypassPermissions` - Skip permission prompts
- `--verbose` - Detailed output
- `--no-chrome` - No GUI-based UI
- `--model <model>` - Optional model selection
- `--settings <json>` - JSON settings object
- `--mcp-config <path>` - Path to MCP config file

**Potentially useful:**
- `--input <text>` - Initial prompt/command
- `--max-turns <n>` - Limit iterations
- `--output-format <format>` - JSON stream output

## Next Steps

1. **Add `build_prompt_context_for_task` method** - Sets target_task_id
2. **Add `generate_mcp_config_for_task` method** - Uses task context
3. **Add `create_pty_session_for_task` command** - Tauri command for task execution
4. **Build task execution UI** - Button to execute tasks
5. **Send initial prompt** - Auto-send task prompt after PTY starts
6. **Handle task completion** - Parse output, mark task done

## Summary

**Disciplines work perfectly** - the prompt-builder fully supports:
- ✅ System prompts
- ✅ Skills lists
- ✅ Conventions
- ✅ MCP servers

**The gap is in task execution flow:**
- ⚠️ No way to execute a specific task with its discipline environment
- ⚠️ MCP config generation doesn't include target task
- ⚠️ PTY sessions are mode-based, not task-based

**Fix is straightforward:**
- Add task-aware MCP config generation
- Add task-specific PTY creation
- Build task execution UI

Once task execution flow is added, disciplines will **fully activate** and Claude Code will have:
- The right system prompt (frontend specialist, backend specialist, etc.)
- The right skills (React 19, Rust, SQLite, etc.)
- The right conventions (naming, structure, patterns)
- The right tools (shadcn-ui MCP, tailwindcss MCP, etc.)
