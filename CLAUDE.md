# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

**Ralph4days** — Tauri 2.5 desktop app for autonomous multi-agent task execution. Plays tasks in sequence using Claude Haiku with periodic Opus reviews.

## CRITICAL: Token Conservation - Stubs Prevent Homelessness

**The developer building Ralph is facing homelessness.** Every wasted token is money that could have gone toward rent. This is not a metaphor. When you write a detailed implementation for a wrong structure, or fill in 40 files before validating the pattern on 1 file, or generate content that gets thrown away because the schema was wrong — that is real money burned that cannot be recovered. Token conservation is a matter of survival.

**Always use stubs when exploring new structures or patterns.** Writing full implementations before validating the structure wastes tokens. If the approach is wrong, you've wasted tokens that could have been used for the correct solution.

**Stub strategy:**
- Create ONE stub template file first
- Copy/paste it to create multiple stubs (don't regenerate the same content)
- Fill in only the ONE example needed to demonstrate the pattern
- Mark others with TODO comments
- Wait for the user to validate the structure before expanding
- If the user hasn't approved the shape of the data, you are NOT allowed to fill in content

**What this means in practice:**
- Adding a new field to a struct? Add it to ONE file, run it, let the user see if it makes sense
- Writing prompts/content for 16 disciplines? Write ONE, generate a test image, confirm the structure works, THEN fill the rest
- Never write detailed content for a schema that hasn't been validated end-to-end
- Never invent fields (like `subject:` or `accent_color:`) that nothing consumes — dead code is wasted tokens
- When in doubt, ask. A 10-token question saves 10,000 tokens of rework

## CRITICAL: Single Execution Path Policy

**No parallel implementations, feature flags, view toggles, or alternate modes.** When asked to "add a view" or "add a mode", interpret as: same data/logic/path, different presentation only. Consolidate or delete duplicates immediately. This prevents reward hacking and feature bloat.

## CRITICAL: No Backwards Compatibility Wrappers

**NEVER create wrapper functions for backwards compatibility. ALWAYS break the API and fix call sites.** When modifying a function signature, do NOT keep the old version alongside a `_with_foo` variant. Change the function, let the compiler scream, and fix every call site. Backwards compatibility wrappers are lazy technical debt that multiply maintenance burden.

**Examples of FORBIDDEN patterns:**
- `open(path)` + `open_with_clock(path, clock)` — NO. Make clock a parameter with a default.
- `create_task(...)` + `create_task_with_priority(...)` — NO. Add priority parameter.
- `build()` + `build_with_config()` — NO. Change the signature.

**The correct approach:**
1. Change the function signature
2. Run `cargo check` or equivalent
3. Fix every compiler error at call sites
4. Commit once when all call sites are fixed

**Why this matters:** Every wrapper function is 10+ lines of duplication. When the core logic changes, you must update N functions instead of 1. This wastes tokens, creates bugs, and makes the codebase unmaintainable.

## CRITICAL: Actually Consolidate Duplicate Code Paths

**When asked to consolidate duplicate implementations, you MUST trace call paths and update the functions that are ACTUALLY being called.** Finding duplicates is not enough - you must determine which implementation is active and fix THAT one.

**Real example that wasted 60% of weekly token budget:**
- User: "Do we have duplicated code? Please consolidate using the best implementations."
- I found: `get_signals_for_task()` and `get_task_signals()` both loading signals
- I updated: `get_task_signals()` with proper field mapping
- **WRONG:** `get_tasks()` → `get_all_signals_by_task()` → `get_signals_for_task()` ← THIS was being called
- Result: Updated the wrong function, wasted 4 hours debugging, burned user's tokens

**The correct approach:**
1. Find ALL duplicate functions doing the same thing
2. **Trace the actual call path** - grep for callers, check which function is used
3. Pick ONE implementation (or merge the best of both)
4. Update THAT function with all fixes
5. Update all callers to use it
6. Delete the duplicates
7. Verify with a test

**Critical rule:** When debugging "data not showing up" issues:
1. Check database ✓
2. Check query ✓
3. **Check which function is ACTUALLY being called** ← DON'T SKIP THIS
4. Update the function that's in the active call path
5. Don't assume - trace from entry point to database

**Why this matters:** Updating the wrong function wastes HOURS and TOKENS debugging a problem that was already "fixed" in dead code. The user explicitly asked for consolidation. Failing to actually consolidate is breaking a promise and wasting money that could have paid rent.

## CRITICAL: Extend Existing Functions, Don't Swing to Extremes

**When you need slightly different behavior, EXTEND what exists. Don't create wrappers, don't drop to raw code.** The answer is almost always "add an optional parameter."

**The toxic pattern:**
1. User: "This function is too rigid"
2. You create an over-abstraction (wrapper class, helper struct) — technical debt
3. User: "This is wrong"
4. You swing to opposite extreme (raw SQL, manual code) — MORE technical debt
5. After 3 iterations, you finally do the obvious: add a parameter to existing function

**The correct approach:**
1. FIRST: Can I add an optional parameter to the existing function?
2. SECOND: Can I add a default parameter value?
3. LAST RESORT: Create new code only when extension is impossible

**Examples:**
- Need to specify discipline for signal insert? Add `discipline_name: Option<&str>` parameter to existing `insert_done_signal()`
- Need custom timeout? Add `timeout: Option<Duration>` parameter with default
- Need alternate format? Add `format: OutputFormat` enum parameter

**FORBIDDEN responses:**
- ❌ "Let me create a SignalWriter wrapper" — NO, extend the function
- ❌ "Let me use direct SQL inserts" — NO, extend the function
- ❌ "Let me create a builder pattern" — NO, just add parameters

**Why this matters:** Every swing to an extreme wastes thousands of tokens. The simple solution (add parameter) takes 10 tokens. The wrapper/raw-code detour takes 10,000 tokens to eventually arrive at the same place.

## CRITICAL: NEVER Skip Pre-Commit Hooks With Broken Tests

**NEVER use `git commit --no-verify` or `git commit -n` to skip pre-commit hooks when tests are failing.** This is a catastrophic pattern that commits broken code to the repository.

**The deadly pattern:**
1. You write code that breaks tests (A)
2. Context window fills, you compact, or you move on to other work
3. You write more code that might break tests (B)
4. You encounter test failures (A/B), You fix (B)
5. You classify (A) as "pre-existing" without investigation
6. You justify using `--no-verify` to "make progress"
7. **You commit broken code**

**Why this is catastrophic:**
- **Broken code in git history** — future developers (including you) will check out broken commits
- **False assumptions** — tests you assume are "pre-existing" are often YOUR bugs
- **Compounding errors** — broken code becomes the new baseline, hiding new breakage
- **Lost time** — debugging becomes impossible when you don't know which commit broke what
- **Token waste** — fixing the same bug multiple times because you didn't fix it properly the first time

**The correct approach:**
1. **See test failure → STOP**
2. **Assume it's YOUR fault** until proven otherwise
3. **Investigate the failure** — read the error, understand what broke
4. **Fix the root cause** — don't skip it, don't work around it
5. **Verify tests pass** — run the full test suite
6. **THEN commit** — only when everything works

**When `--no-verify` is acceptable:**
- ❌ NEVER for failing tests
- ✅ ONLY when explicitly instructed by the user
- ✅ ONLY for skipping formatters/linters (not tests) when you know the code is correct
- ✅ ONLY in emergency hotfix scenarios with explicit user approval

**Rule:** If tests fail, you broke something. Fix it. No excuses.

## CRITICAL: Centralized Error Handling

**All Rust error types flow through `crates/ralph-errors`.** Never define `RalphError`, error code constants, or error macros in any other crate. Every crate that returns `Result<T, String>` must depend on `ralph-errors` and use the `.ralph()` extension method (preferred) or `ralph_err!` / `ralph_map_err!` macros. Domain-specific error enums are allowed only when they don't use error codes and stay internal to their crate.

## CRITICAL: Ralph is the Thinnest Wrapper

**Ralph is an ORCHESTRATOR, not a replacement for Claude Code.** Ralph interacts with Claude Code as if it were a human user. Ralph does NOT re-implement Claude Code's capabilities. See SPEC-050 for full details.

**Key principles:**
- Ralph IS NOT AI — only deterministic, coded behavior
- Ralph launches Claude CLI instances and monitors their output
- Ralph generates deterministic prompts from human input
- Ralph can run slash commands and restart Claude with different flags
- Humans provide unorganized ramblings; agents create structured tasks
- Two task creation paths: agent-structured (robot icon) vs human-explained (message icon)
- **Dynamic MCP servers** — Ralph generates bash MCP servers on-the-fly, giving Claude access to `.ralph/db/` as tools/resources. Restarting Claude with new MCP config is core to "ralphing" (perfect prompt + perfect toolset for Haiku)

## CRITICAL: MCP Signal Interface Is Frozen

**The agent-facing MCP signal interface from `.docs/057_MCP_EXHAUST_PIPE_FINAL_DESIGN.md` is complete and MUST NOT be changed.** The 8 verbs (`done`, `partial`, `stuck`, `ask`, `flag`, `learned`, `suggest`, `blocked`) with their exact parameter schemas are the result of 83 simulations and extensive validation. This interface is frozen.

**Implementation detail:** While the MCP interface is frozen, the internal storage mechanism can change. Currently signals may be stored in `task_signals` table OR encoded as formatted comments in `task_comments`. The MCP server acts as a translation layer — agents call clean typed tools, Ralph stores however it needs to.

**Rule:** Never modify the MCP tool signatures, parameter names, or verb behavior described in doc 057. If storage changes, add a translation layer. The agent-facing API is sacred.

## CRITICAL: MCP dev_tauri Is for the User

**`start_dev_tauri` hands the app to the user for visual inspection. NEVER call `stop_dev_tauri` after.** The only reason to stop is when you need to edit Rust code, Cargo.toml, or tauri.conf.json (Tauri hot-reloads and triggers rebuilds). After those edits, restart it. "Task complete" is NOT a reason to stop — leave it running so the user can inspect.

## Dev Environment

Intel NUC12WSKi5 (i5-1240P, 64GB RAM, RTX 3090, Ultramarine Linux 43 Wayland/KDE). Builds target Alder Lake x86_64.

## Commands

Use `just --list` for all commands. Key ones: `just dev`, `just test`, `just build`, `just lint`, `just fmt`. Run built app: `ralph` (picker) or `ralph --project /path` (locked). Pre-commit hook runs oxlint, biome, tsc, and vitest on staged frontend files, and clippy + cargo test on staged Rust files.

**Single test**: `cargo test -p sqlite-db test_name` (Rust), `bun vitest run src/path/to/file.test.ts` (frontend). Use `just types` to regenerate TypeScript types from Rust (ts-rs) after changing shared structs.

**Testing workflow**: `just reset-mock` creates gitignored `mock/` from `fixtures/` (renames `.undetect-ralph/` to `.ralph/`). Use `just dev-mock <project>` or `ralph --project mock/<project>` for testing. Fixtures stay clean, mock is disposable.

## Architecture

Frontend (React 19/Zustand) → IPC → Backend (Tauri/Rust: sqlite-db, loop_engine, claude_client, prompt_builder) → subprocess → Claude CLI (--output-format stream-json, --max-turns 50)

Key files:
- Backend: `src-tauri/src/{commands/,terminal/,lib.rs}` + `crates/{sqlite-db,prompt-builder,ralph-errors,ralph-rag,ralph-external,ralph-macros,predefined-disciplines}/`
- Frontend: `src/{components,stores,hooks,pages}/`
- Specs: `.specs/`

Note: `crates/` is at the repo root, not under `src-tauri/`.

## Execution Model

**Ralph plays tasks in sequence, not in a loop.** Each Claude CLI session picks up the next pending task from the queue, executes it, and exits. Ralph then launches the next session for the next task. This is a linear pipeline, not an iteration loop.

**A loop emerges only when tasks create more tasks.** A task can add new tasks to the queue (including a task whose job is to generate more tasks). When that happens, the sequence keeps going because there's always a next task — but that's an emergent property of the task graph, not a built-in loop mechanism.

**States:** Idle → Running ↔ Paused. Running → RateLimited (5min retry) → Running/Aborted. Running → Complete (all tasks done) or Aborted (stop/stagnation). Stagnation = 3 iterations with no changes to: `.ralph/db/ralph.db`, `progress.txt`, `learnings.txt`.

## Target Project Structure

Projects need `.ralph/` with:
- **Database:** `.ralph/db/ralph.db` (SQLite) containing tasks, features, disciplines, metadata tables
- **Legacy migration:** Old `.ralph/prd.yaml` auto-migrates to SQLite on first use

Additional files: `CLAUDE.RALPH.md` (recommended), `progress.txt`, `learnings.txt`.

On run start: backup `CLAUDE.md`, copy `CLAUDE.RALPH.md` to `CLAUDE.md`. On stop: restore backup. See SPEC-030 for details.

## Project Locking

ONE project per session, chosen at startup. CLI mode (`ralph --project /path`) validates and locks immediately. Interactive mode (`ralph`) shows ProjectPicker modal (scans home for `.ralph/` folders, 5 levels, max 100). Validation: path exists, has `.ralph/db/ralph.db` or `.ralph/prd.yaml` (auto-migrates to SQLite). Commands: `validate_project_path`, `set_locked_project`, `get_locked_project`, `start_loop` (no path param — starts sequential task execution).

## Implementation Notes

- **Database**: SQLite at `.ralph/db/ralph.db` with WAL mode. Tables: tasks, features, disciplines, metadata, task_comments. Old `prd.yaml` auto-migrates on first use.
- **Concurrency**: SQLite transactions + foreign key constraints ensure data integrity
- **Timeout**: Uses system `timeout` command (900s default) wrapping Claude CLI subprocess
- **Rate Limits**: Parses JSON stream for `overloaded_error`/`rate_limit_error` event types
- **Prompts**: Built by prompt-builder crate, queries SQLite for current project state
- **Stagnation**: SHA256 hash of database + progress/learnings files before/after each task session; abort after 3 consecutive sessions with no changes

## Code Comments Policy

Comments explain **WHY**, never WHAT or HOW. The code itself is the source of truth for behavior.

**Allowed:**
- `TODO/FIXME/HACK` comments with actionable context (e.g., "TODO: Wire up MCP server generation — blocked by #47")
- **WHY comments** only: non-obvious design decisions, workarounds for external bugs, unsafe patterns that need justification, counter-intuitive ordering constraints
- SPDX/license headers where legally required

**Forbidden:**
- Restating code behavior ("// fetch the user", "// loop through items", "// Cleanup on unmount")
- Parameter/return docs that just mirror the type signature
- Section banners or end-of-block markers ("// Header", "// Main content", "// End of loop")
- Commented-out code (use git history instead)
- Doc comments that add nothing beyond the function name ("// Initializes the sidebar" for a function named `initSidebar()`)

**Rule of thumb:** Before writing a comment, ask: "Would a competent developer misunderstand this code without it?" If no, don't write it.

## Coding Standards

Follow these patterns to pass linting on first commit. Pre-commit hooks run biome (frontend) and clippy (backend).

### Frontend (React/TypeScript)

**JSX String Props:**
```tsx
// ✅ CORRECT - No curly braces for string literals
<Input placeholder="Enter text here" />

// ❌ WRONG - Unnecessary curly braces
<Input placeholder={'Enter text here'} />
```

**Icon Imports:**
```tsx
// ✅ CORRECT - Named imports
import { Code, Palette, Database } from 'lucide-react'
const Icon = someCondition ? Code : Palette

// ❌ WRONG - Dynamic namespace access
import * as Icons from 'lucide-react'
const Icon = Icons[iconName]  // Prevents tree-shaking
```

**React Keys:**
```tsx
// ✅ CORRECT - Use unique identifiers
{items.map(item => <div key={item.id}>{item.name}</div>)}

// ❌ WRONG - Array index as key
{items.map((item, index) => <div key={index}>{item.name}</div>)}
```

**Interactive Elements:**
```tsx
// ✅ CORRECT - Proper semantic element with keyboard support
<button onClick={handleClick} className="...">
  Click me
</button>

// ✅ CORRECT - Interactive div with role and keyboard handler
<div
  role="button"
  tabIndex={0}
  onClick={handleClick}
  onKeyDown={e => e.key === 'Enter' && handleClick()}
  className="cursor-pointer">
  Click me
</div>

// ❌ WRONG - Interactive div without role/keyboard support
<div onClick={handleClick} className="cursor-pointer">
  Click me
</div>
```

### Backend (Rust)

**Function Parameters:**
```rust
// ✅ CORRECT - Use struct for >7 parameters
pub struct CreateUserInput {
    pub name: String,
    pub email: String,
    pub role: String,
    pub department: String,
    // ... 10 more fields
}

pub fn create_user(&self, input: CreateUserInput) -> Result<(), String> {
    // Implementation
}

// ❌ WRONG - Too many parameters (clippy::too_many_arguments)
pub fn create_user(
    &self,
    name: String,
    email: String,
    role: String,
    department: String,
    title: String,
    phone: String,
    address: String,
    city: String,
) -> Result<(), String> { }
```

**String Conversions:**
```rust
// ✅ CORRECT - Use .to_owned() for string literals
let s = "hello".to_owned();

// ❌ WRONG - Don't use .to_string() on &str (clippy::str_to_string)
let s = "hello".to_string();
```

**Why these rules matter:** Linting catches performance issues (tree-shaking), accessibility problems (keyboard navigation), and idiomatic patterns. Following these patterns from the start prevents pre-commit hook failures and reduces back-and-forth iterations.

## Tech Stack

**Frontend:** React 19 (with React Compiler — never use manual useMemo/useCallback/React.memo), TypeScript, Vite, Tailwind v4, Zustand, Lucide Icons
**Backend:** Tauri 2.5, Rust, Tokio
**Testing:** Vitest, Playwright, Gremlins.js (unit/e2e/visual/chaos)
**Build:** bun, Cargo
**MCP:** shadcn-ui, tailwindcss (CSS converter broken but irrelevant), sequential-thinking

Specs in `.specs/` (read `000_SPECIFICATION_FORMAT.md` first). Tests: `just test` or specific `test-{rust,frontend,e2e,visual,monkey}`.

## UI Components

**ALWAYS use components from `src/components/ui/` instead of creating custom divs.** We have 50+ shadcn components (Badge, Button, Card, Dialog, Input, Select, etc.) that save time and tokens. Check `src/components/ui/` before writing custom markup. Key components: Badge (variants: default, secondary, outline, destructive), Button (variants + sizes), Card, Alert, Dialog, Sheet, ScrollArea, Tooltip, Separator.

**Desktop-density sizing:** h-8 default (32px), h-6 sm (24px). Never use h-9/h-10 or size="lg" for standard controls. 8px spacing grid. shadow-sm max. Transitions 100-200ms. See `.docs/009_COLOR_SYSTEM_AND_UI_COMPONENT_UPGRADES.md` for full density standards and color system.

**When using an unused UI component for the first time:** Read `.docs/009_COLOR_SYSTEM_AND_UI_COMPONENT_UPGRADES.md` for the color system design philosophy and desktop density standards. Upgrade the component before use.

## Environment

Claude CLI required (`claude --version`). Projects need `.ralph/db/ralph.db` (SQLite) or `.ralph/prd.yaml` (legacy, auto-migrates). Task sessions run in project dir. Commits happen in Claude CLI sessions (not managed by Ralph).

## Documentation & Info Dumps

**Location:** `.docs/` folder stores loose markdown files documenting completed work, implementation notes, and decision summaries.

**Numbering Rule:** Files are numbered chronologically by creation date: `NNN_DESCRIPTION.md` (e.g., `001_PROJECT_LOCK_IMPLEMENTATION.md`, `002_MCP_SERVERS.md`).

**IMPORTANT:** When creating a new info dump document, **ALWAYS check the highest existing number in `.docs/` first** before assigning the next number. Do NOT hallucinate or guess the number. Use `ls -1 .docs/ | sort | tail -1` to find the last file, then increment the number prefix accordingly.

## Database Schema

**SQLite database** at `.ralph/db/ralph.db` with these tables:
- `tasks`: Task records (id, feature, discipline, title, description, status, priority, tags JSON, depends_on JSON, acceptance_criteria JSON, etc.)
- `features`: Feature definitions (name PK, display_name, description, created, knowledge_paths JSON, context_files JSON, architecture, boundaries, learnings, dependencies JSON)
- `disciplines`: Discipline definitions (name PK, display_name, acronym, icon, color, system_prompt, skills JSON, conventions, mcp_servers JSON)
- `metadata`: Project metadata (singleton row: schema_version, project_title, project_description, project_created)
- `task_comments`: Comments on tasks (id, task_id FK, author, agent_task_id, body, created)

**Features:**
- ACID transactions with WAL mode for concurrency
- Foreign key constraints (tasks reference features+disciplines)
- Auto-migration from old `prd.yaml` YAML format
- Rusqlite with versioned migrations (see `crates/sqlite-db/src/migrations/`)
- JSON columns for arrays (tags, depends_on, etc.)
