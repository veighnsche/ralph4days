# CLAUDE.md

**Ralph4days** — Tauri 2.5 desktop app for autonomous multi-agent task execution. Plays tasks in sequence using Claude Haiku with periodic Opus reviews.

## CRITICAL: Single Execution Path Policy

**No parallel implementations, feature flags, view toggles, or alternate modes.** When asked to "add a view" or "add a mode", interpret as: same data/logic/path, different presentation only. Consolidate or delete duplicates immediately. This prevents reward hacking and feature bloat.

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

## Dev Environment

Intel NUC12WSKi5 (i5-1240P, 64GB RAM, RTX 3090, Ultramarine Linux 43 Wayland/KDE). Builds target Alder Lake x86_64.

## Commands

Use `just --list` for all commands. Key ones: `just dev`, `just test`, `just build`, `just lint`, `just fmt`. Run built app: `ralph` (picker) or `ralph --project /path` (locked). Pre-commit hook runs oxlint, biome, tsc, and vitest on staged frontend files, and clippy + cargo test on staged Rust files.

**Testing workflow**: `just reset-mock` creates gitignored `mock/` from `fixtures/` (renames `.undetect-ralph/` to `.ralph/`). Use `just dev-mock <project>` or `ralph --project mock/<project>` for testing. Fixtures stay clean, mock is disposable.

## Architecture

Frontend (React 19/Zustand) → IPC → Backend (Tauri/Rust: yaml_db, loop_engine, claude_client, prompt_builder) → subprocess → Claude CLI (--output-format stream-json, --max-turns 50)

Key files:
- Backend: `src-tauri/src/{yaml_db/,loop_engine,claude_client,prompt_builder,commands}.rs`
- Frontend: `src/{components,stores}/`
- Specs: `.specs/`

## Execution Model

**Ralph plays tasks in sequence, not in a loop.** Each Claude CLI session picks up the next pending task from the queue, executes it, and exits. Ralph then launches the next session for the next task. This is a linear pipeline, not an iteration loop.

**A loop emerges only when tasks create more tasks.** A task can add new tasks to the queue (including a task whose job is to generate more tasks). When that happens, the sequence keeps going because there's always a next task — but that's an emergent property of the task graph, not a built-in loop mechanism.

**States:** Idle → Running ↔ Paused. Running → RateLimited (5min retry) → Running/Aborted. Running → Complete (all tasks done) or Aborted (stop/stagnation). Stagnation = 3 iterations with no changes to: tasks.yaml, features.yaml, disciplines.yaml, metadata.yaml, progress.txt, learnings.txt.

## Target Project Structure

Projects need `.ralph/` with either:
- **New format (preferred):** `.ralph/db/` containing `tasks.yaml`, `features.yaml`, `disciplines.yaml`, `metadata.yaml`
- **Legacy format:** `.ralph/prd.yaml` (auto-migrates to new format on first use)

Additional files: `CLAUDE.RALPH.md` (recommended), `progress.txt`, `learnings.txt`.

On run start: backup `CLAUDE.md`, copy `CLAUDE.RALPH.md` to `CLAUDE.md`. On stop: restore backup. See SPEC-030 for details.

## Project Locking

ONE project per session, chosen at startup. CLI mode (`ralph --project /path`) validates and locks immediately. Interactive mode (`ralph`) shows ProjectPicker modal (scans home for `.ralph/` folders, 5 levels, max 100). Validation: path exists, has `.ralph/db/` or `.ralph/prd.yaml` (auto-migrates). Commands: `validate_project_path`, `set_locked_project`, `get_locked_project`, `start_loop` (no path param — starts sequential task execution).

## Implementation Notes

- **Database**: Multi-file YAML database in `.ralph/db/` (tasks, features, disciplines, metadata). Old `prd.yaml` auto-migrates on first use.
- **Concurrency**: File locking via fs2 crate prevents race conditions during bulk task creation
- **Timeout**: Uses system `timeout` command (900s default) wrapping Claude CLI subprocess
- **Rate Limits**: Parses JSON stream for `overloaded_error`/`rate_limit_error` event types
- **Prompts**: Inline file contents (no @file syntax): aggregates 4 YAML files into PRD section
- **Stagnation**: SHA256 hash of 6 files (tasks.yaml, features.yaml, disciplines.yaml, metadata.yaml, progress.txt, learnings.txt) before/after each task session; abort after 3 consecutive sessions with no changes

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

Claude CLI required (`claude --version`). Projects need `.ralph/db/` (new format) or `.ralph/prd.yaml` (legacy, auto-migrates). Task sessions run in project dir. Commits happen in Claude CLI sessions (not managed by Ralph).

## Documentation & Info Dumps

**Location:** `.docs/` folder stores loose markdown files documenting completed work, implementation notes, and decision summaries.

**Numbering Rule:** Files are numbered chronologically by creation date: `NNN_DESCRIPTION.md` (e.g., `001_PROJECT_LOCK_IMPLEMENTATION.md`, `002_MCP_SERVERS.md`).

**IMPORTANT:** When creating a new info dump document, **ALWAYS check the highest existing number in `.docs/` first** before assigning the next number. Do NOT hallucinate or guess the number. Use `ls -1 .docs/ | sort | tail -1` to find the last file, then increment the number prefix accordingly.

## Database Schema

**New multi-file YAML format** (`.ralph/db/`):
- `tasks.yaml`: Task records (id, feature, discipline, title, description, status, priority, tags, depends_on, acceptance_criteria, etc.)
- `features.yaml`: Feature definitions (name, display_name, description, created) - auto-populated when creating tasks
- `disciplines.yaml`: Discipline definitions (name, display_name, icon, color) - 10 defaults, user-customizable
- `metadata.yaml`: Project metadata (title, description, created) and counters (highest task ID per feature+discipline)

**Features:**
- Thread-safe task creation with file locking (prevents race conditions)
- Auto-migration from old `prd.yaml` format
- Atomic writes (temp files + rename pattern)
- Dependency validation (ensures depends_on references exist)
- Auto-populate features and disciplines on task creation
