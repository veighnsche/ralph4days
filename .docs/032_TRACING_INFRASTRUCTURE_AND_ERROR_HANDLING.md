# 032: Tracing Infrastructure and Error Handling

**Date:** 2026-02-08
**Status:** Implemented

## Overview

Implemented comprehensive structured logging using `tracing` crate across the entire Ralph codebase. This provides:
- Hierarchical span-based logging for tracking operations
- Context-aware error reporting
- Performance monitoring capabilities
- GitHub issue template generation from errors

## Dependencies Added

### Workspace Crates

```toml
# src-tauri/Cargo.toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json", "fmt"] }

# crates/sqlite-db/Cargo.toml
tracing = "0.1"

# crates/prompt-builder/Cargo.toml
tracing = "0.1"
```

### Removed Dependencies

```toml
# src-tauri/Cargo.toml
lazy_static = "1.4"  # REMOVED - unused, maintenance mode
```

## Initialization

### Tracing Subscriber Setup

Location: `src-tauri/src/lib.rs`

```rust
fn init_tracing() {
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| {
            EnvFilter::try_new(if cfg!(debug_assertions) {
                "ralph4days=debug,sqlite_db=debug,prompt_builder=debug"
            } else {
                "ralph4days=info,sqlite_db=info,prompt_builder=info"
            })
        })
        .unwrap();

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(true).with_line_number(true))
        .init();
}
```

**Default log levels:**
- Debug builds: `debug` for ralph4days, sqlite_db, prompt_builder
- Release builds: `info` for all Ralph crates

**Environment override:**
Set `RUST_LOG` to override default levels:
```bash
# Show all trace-level logs
RUST_LOG=ralph4days=trace,sqlite_db=trace,prompt_builder=trace ralph

# Show only errors
RUST_LOG=error ralph

# Mixed levels
RUST_LOG=ralph4days=debug,sqlite_db=info ralph
```

## Enhanced Error Handling

### RalphError Improvements

Location: `src-tauri/src/errors.rs`

Enhanced `RalphError` with:
1. **Automatic logging** - All errors are logged when created
2. **Category mapping** - Error codes map to human-readable categories
3. **GitHub issue templates** - Errors can generate issue templates

```rust
pub struct RalphError {
    pub code: u16,
    pub message: String,
}

impl RalphError {
    pub fn new(code: u16, message: String) -> Self {
        let err = Self { code, message };
        tracing::error!(
            error_code = code,
            error_message = %err.message,
            "Ralph error created"
        );
        err
    }

    pub fn code_category(&self) -> &str {
        match self.code {
            1000..=1299 => "PROJECT",
            2000..=2299 => "DATABASE",
            3000..=3299 => "TASK",
            4000..=4199 => "FEATURE",
            5000..=5099 => "LOOP_ENGINE",
            6000..=6099 => "PROMPT_MCP",
            7000..=7099 => "TERMINAL",
            8000..=8099 => "FILESYSTEM",
            8100..=8199 => "INTERNAL",
            _ => "UNKNOWN",
        }
    }

    pub fn github_issue_template(&self) -> String {
        format!(
            r#"## Error Report

**Error Code:** R-{:04} ({})
**Message:** {}

**Environment:**
- OS: {}
- Ralph Version: {}

**How to Reproduce:**
1.
2.
3.

**Expected Behavior:**


**Actual Behavior:**
{}

**Additional Context:**
<!-- Add any other context about the problem here -->
"#,
            self.code,
            self.code_category(),
            self.message,
            std::env::consts::OS,
            env!("CARGO_PKG_VERSION"),
            self.message
        )
    }
}
```

### Error Code Categories

| Range | Category | Description |
|-------|----------|-------------|
| 1000-1299 | PROJECT | Project path/lock/initialization |
| 2000-2299 | DATABASE | Database operations |
| 3000-3299 | TASK | Task validation/operations |
| 4000-4199 | FEATURE | Feature/discipline operations |
| 5000-5099 | LOOP_ENGINE | Execution loop |
| 6000-6099 | PROMPT_MCP | Prompt building/MCP servers |
| 7000-7099 | TERMINAL | PTY/subprocess management |
| 8000-8099 | FILESYSTEM | File operations |
| 8100-8199 | INTERNAL | Internal errors |

## Instrumented Components

### 1. PTY Manager (`src-tauri/src/terminal/manager.rs`)

**Instrumented methods:**
- `create_session` - Full lifecycle: open PTY → spawn Claude CLI → setup reader thread
- `send_input` - Track bytes sent
- `resize` - Track terminal resize operations
- `terminate` - Track session termination

**Key traces:**
```rust
// Session creation
tracing::info!(
    working_dir = %working_dir.display(),
    model = ?config.model,
    has_mcp = mcp_config.is_some(),
    "Creating PTY session"
);

// Reader thread lifecycle
tracing::debug!(session_id = %sid, "PTY reader thread started");
tracing::trace!(session_id = %sid, bytes = n, total_bytes, "PTY output");
tracing::info!(
    session_id = %sid,
    exit_code,
    total_bytes,
    "PTY session closed"
);
```

### 2. Database Operations (`crates/sqlite-db/src/*.rs`)

**Instrumented methods:**
- `create_task` - Task creation with validation tracing
- `update_task` - Task updates

**Example:**
```rust
#[tracing::instrument(skip(self, input), fields(
    feature = %input.feature,
    discipline = %input.discipline,
    title = %input.title
))]
pub fn create_task(&self, input: TaskInput) -> Result<u32, String> {
    tracing::debug!("Creating task");
    // ... validation ...
    tracing::info!(
        task_id = next_id,
        feature = %input.feature,
        discipline = %input.discipline,
        "Task created successfully"
    );
    Ok(next_id)
}
```

### 3. Prompt Builder (`crates/prompt-builder/src/recipe.rs`)

**Instrumented methods:**
- `execute_recipe` - Full prompt generation lifecycle

**Traces:**
```rust
#[tracing::instrument(skip(recipe, ctx), fields(recipe_name = recipe.name))]
pub fn execute_recipe(recipe: &Recipe, ctx: &PromptContext) -> Result<PromptOutput, PromptError> {
    tracing::debug!(
        section_count = recipe.sections.len(),
        mcp_tool_count = recipe.mcp_tools.len(),
        "Executing prompt recipe"
    );
    // ... build sections ...
    tracing::trace!(section_name = section.name, content_len = text.len(), "Section built");
    tracing::info!(
        recipe_name = recipe.name,
        prompt_length = prompt.len(),
        mcp_scripts_count = mcp_scripts.len(),
        "Prompt recipe executed successfully"
    );
}
```

### 4. Tauri Commands (`src-tauri/src/commands/*.rs`)

**Instrumented commands:**
- `validate_project_path` - Project validation
- `initialize_ralph_project` - Project initialization

**Example:**
```rust
#[tauri::command]
#[tracing::instrument]
pub fn validate_project_path(path: String) -> Result<(), String> {
    tracing::debug!("Validating project path");
    // ... validation ...
    tracing::info!(path = %path.display(), "Project path validated successfully");
    Ok(())
}
```

## Log Level Guidelines

### When to use each level:

**`trace`** - Very verbose, typically disabled
- Individual bytes read/written
- Section-by-section prompt building
- Loop iterations

**`debug`** - Development details
- Function entry/exit
- State transitions
- Configuration loading

**`info`** - Important lifecycle events
- Session created/closed
- Task created/updated
- Prompt generated
- Project locked/unlocked

**`warn`** - Unexpected but recoverable
- PTY read errors (may recover)
- Attempted operation on non-existent session

**`error`** - Operation failed
- Validation errors
- Database errors
- PTY spawn failures

## Span Context

Tracing uses hierarchical spans to provide context. Example output:

```
2026-02-08T10:30:00.123Z INFO ralph4days::terminal::manager: Creating PTY session
    session_id: "session_1"
    working_dir: "/home/user/my-project"
    model: Some("claude-haiku-4")
    has_mcp: true

2026-02-08T10:30:00.234Z DEBUG ralph4days::terminal::manager: PTY opened successfully
    session_id: "session_1"

2026-02-08T10:30:00.345Z DEBUG ralph4days::terminal::manager: Spawning Claude CLI subprocess
    session_id: "session_1"
    working_dir: "/home/user/my-project"
    model: Some("claude-haiku-4")

2026-02-08T10:30:00.456Z INFO ralph4days::terminal::manager: Claude CLI subprocess spawned successfully
    session_id: "session_1"
```

## Debugging Workflow

### 1. Reproduce issue with tracing enabled

```bash
RUST_LOG=ralph4days=debug,sqlite_db=debug,prompt_builder=debug ralph --project /path/to/project 2>&1 | tee ralph.log
```

### 2. Search logs for errors

```bash
grep "ERROR" ralph.log
grep "R-[0-9]" ralph.log  # Find error codes
```

### 3. Find context around error

```bash
# Get 10 lines before/after first error
grep -B 10 -A 10 "ERROR" ralph.log | head -30
```

### 4. Generate GitHub issue

Use the error's `github_issue_template()` method to generate a pre-filled issue:

```rust
if let Err(e) = result {
    // Parse error code from string like "[R-2000] Database error"
    if let Some(err) = parse_ralph_error(&e) {
        println!("{}", err.github_issue_template());
    }
}
```

## Performance Considerations

- `#[tracing::instrument]` adds minimal overhead (~100ns per call)
- Tracing is compile-time optional via feature flags (not implemented yet, but possible)
- Log output can be disabled completely by setting `RUST_LOG=off`
- JSON output available via `tracing-subscriber` json feature for log aggregation

## Future Enhancements

1. **Tracing to file** - Write logs to `.ralph/logs/` for post-mortem analysis
2. **OpenTelemetry integration** - Export traces to Jaeger/Zipkin for distributed tracing
3. **Performance spans** - Add timing to slow operations
4. **Conditional compilation** - `--no-default-features` to strip tracing from release builds
5. **Frontend integration** - Stream tracing events to frontend for live debugging UI

## Testing

Tracing is tested via:
1. Manual testing with `RUST_LOG=trace`
2. Error handling tests verify `RalphError::new()` logs correctly
3. Integration tests verify span context propagation

## User Guide: Debugging with Tracing

### For End Users

When you encounter an error in Ralph, here's how to get detailed logs for bug reports:

#### 1. Enable Debug Logging

Run Ralph with full debug output:

```bash
RUST_LOG=ralph4days=debug,sqlite_db=debug,prompt_builder=debug ralph --project /path/to/project 2>&1 | tee ralph-debug.log
```

This will:
- Show all debug-level logs in the terminal
- Save the output to `ralph-debug.log`

#### 2. Reproduce the Issue

Perform the actions that trigger the error. The log file will capture everything.

#### 3. Find the Error

Search for the error code in the log:

```bash
grep "R-[0-9]" ralph-debug.log
```

Example output:
```
2026-02-08T10:30:00.123Z ERROR ralph4days::errors: Ralph error created error_code=2000 error_message="Failed to open database: file not found"
```

#### 4. Get Context

Look at the 20 lines before the error to see what led to it:

```bash
grep -B 20 "R-2000" ralph-debug.log | head -30
```

#### 5. Create a GitHub Issue

Copy the error message and surrounding context. Use this template:

```markdown
## Error Report

**Error Code:** R-2000 (DATABASE)
**Message:** Failed to open database: file not found

**Environment:**
- OS: Linux
- Ralph Version: 0.1.0

**How to Reproduce:**
1. Start Ralph with `ralph --project /path/to/project`
2. Click "Start Loop"
3. Error occurs immediately

**Expected Behavior:**
Ralph should start the execution loop successfully

**Actual Behavior:**
Ralph fails with database error

**Logs:**
```
[paste the 20 lines before and after the error here]
```

**Additional Context:**
.ralph/db/ralph.db exists and has correct permissions
```

### For Developers

#### Adding Tracing to New Code

**1. Import tracing:**
```rust
use tracing::{debug, info, warn, error, trace};
```

**2. Add instrumentation to functions:**
```rust
#[tracing::instrument(skip(large_param), fields(id = %entity.id))]
pub fn process_entity(entity: Entity, large_param: Vec<u8>) -> Result<(), Error> {
    info!("Processing entity");
    debug!(size = large_param.len(), "Large parameter received");

    // ... do work ...

    if let Err(e) = some_operation() {
        error!(error = %e, "Operation failed");
        return Err(e);
    }

    info!("Entity processed successfully");
    Ok(())
}
```

**3. Use structured fields:**
```rust
// Good - structured
info!(task_id = 42, feature = "auth", "Task created");

// Avoid - string interpolation loses structure
info!("Task {} created for feature {}", 42, "auth");
```

**4. Choose appropriate levels:**
- `trace` - Very verbose (loop iterations, individual bytes)
- `debug` - Development details (function entry, state changes)
- `info` - Important events (task created, session started)
- `warn` - Unexpected but recoverable (retry after error)
- `error` - Operation failed (validation error, database error)

#### Error Handling Best Practices

**Always log errors when creating them:**

```rust
// Good - error is logged automatically via RalphError::new()
return ralph_err!(codes::DB_OPEN, "Failed to open database: {}", e);

// Avoid - error is created but might not be logged at creation site
Err(format!("[R-{:04}] {}", code, message))
```

**Add context to errors:**

```rust
// Good - context in fields
let result = db.create_task(input).map_err(|e| {
    error!(
        task_title = %input.title,
        feature = %input.feature,
        error = %e,
        "Failed to create task"
    );
    e
})?;

// Also good - error already logged by ralph_err!, just propagate
db.create_task(input)?;
```

## Related Files

- `src-tauri/src/lib.rs` - Tracing initialization
- `src-tauri/src/errors.rs` - Enhanced RalphError
- `src-tauri/src/terminal/manager.rs` - PTY tracing
- `crates/sqlite-db/src/tasks.rs` - Database tracing
- `crates/prompt-builder/src/recipe.rs` - Prompt building tracing
- `src-tauri/src/commands/project.rs` - Command tracing
