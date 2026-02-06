# CLAUDE.md

**Ralph4days** — Tauri 2.5 desktop app for autonomous multi-agent build loops. Runs Claude Haiku in a loop to complete PRD tasks with periodic Opus reviews.

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

Use `just --list` for all commands. Key ones: `just dev`, `just test`, `just build`, `just lint`, `just fmt`. Run built app: `ralph` (picker) or `ralph --project /path` (locked). Pre-commit hook runs oxlint and biome on staged files.

**Testing workflow**: `just reset-mock` creates gitignored `mock/` from `fixtures/` (renames `.undetect-ralph/` to `.ralph/`). Use `just dev-mock <project>` or `ralph --project mock/<project>` for testing. Fixtures stay clean, mock is disposable.

## Architecture

Frontend (React 19/Zustand) → IPC → Backend (Tauri/Rust: yaml_db, loop_engine, claude_client, prompt_builder) → subprocess → Claude CLI (--output-format stream-json, --max-turns 50)

Key files:
- Backend: `src-tauri/src/{yaml_db/,loop_engine,claude_client,prompt_builder,commands}.rs`
- Frontend: `src/{components,stores}/`
- Specs: `.specs/`

## Loop States

Idle → Running ↔ Paused. Running → RateLimited (5min retry) → Running/Aborted. Running → Complete (all tasks done) or Aborted (stop/stagnation). Stagnation = 3 iterations with no changes to: tasks.yaml, features.yaml, disciplines.yaml, metadata.yaml, progress.txt, learnings.txt.

## Target Project Structure

Projects need `.ralph/` with either:
- **New format (preferred):** `.ralph/db/` containing `tasks.yaml`, `features.yaml`, `disciplines.yaml`, `metadata.yaml`
- **Legacy format:** `.ralph/prd.yaml` (auto-migrates to new format on first use)

Additional files: `CLAUDE.RALPH.md` (recommended), `progress.txt`, `learnings.txt`.

On loop start: backup `CLAUDE.md`, copy `CLAUDE.RALPH.md` to `CLAUDE.md`. On stop: restore backup. See SPEC-030 for details.

## Project Locking

ONE project per session, chosen at startup. CLI mode (`ralph --project /path`) validates and locks immediately. Interactive mode (`ralph`) shows ProjectPicker modal (scans home for `.ralph/` folders, 5 levels, max 100). Validation: path exists, has `.ralph/db/` or `.ralph/prd.yaml` (auto-migrates). Commands: `validate_project_path`, `set_locked_project`, `get_locked_project`, `start_loop` (no path param).

## Implementation Notes

- **Database**: Multi-file YAML database in `.ralph/db/` (tasks, features, disciplines, metadata). Old `prd.yaml` auto-migrates on first use.
- **Concurrency**: File locking via fs2 crate prevents race conditions during bulk task creation
- **Timeout**: Uses system `timeout` command (900s default) wrapping Claude CLI subprocess
- **Rate Limits**: Parses JSON stream for `overloaded_error`/`rate_limit_error` event types
- **Prompts**: Inline file contents (no @file syntax): aggregates 4 YAML files into PRD section
- **Stagnation**: SHA256 hash of 6 files (tasks.yaml, features.yaml, disciplines.yaml, metadata.yaml, progress.txt, learnings.txt) before/after iteration; abort after 3 unchanged iterations

## Tech Stack

**Frontend:** React 19, TypeScript, Vite, Tailwind v4, Zustand, Lucide Icons
**Backend:** Tauri 2.5, Rust, Tokio
**Testing:** Vitest, Playwright, Gremlins.js (unit/e2e/visual/chaos)
**Build:** bun, Cargo
**MCP:** shadcn-ui, tailwindcss (CSS converter broken but irrelevant), sequential-thinking

Specs in `.specs/` (read `000_SPECIFICATION_FORMAT.md` first). Tests: `just test` or specific `test-{rust,frontend,e2e,visual,monkey}`.

## UI Components

**ALWAYS use components from `src/components/ui/` instead of creating custom divs.** We have 50+ shadcn components (Badge, Button, Card, Dialog, Input, Select, etc.) that save time and tokens. Check `src/components/ui/` before writing custom markup. Key components: Badge (variants: default, secondary, outline, destructive), Button (variants + sizes), Card, Alert, Dialog, Sheet, ScrollArea, Tooltip, Separator.

**When using an unused UI component for the first time:** Read `.docs/009_COLOR_SYSTEM_AND_UI_COMPONENT_UPGRADES.md` for the color system design philosophy and upgrade the component before use. The guide explains when to use primary (main actions), secondary (brand/form states), or accent (hover/focus feedback).

## Environment

Claude CLI required (`claude --version`). Projects need `.ralph/db/` (new format) or `.ralph/prd.yaml` (legacy, auto-migrates). Loop runs in project dir. Commits happen in Claude CLI sessions (not managed by Ralph).

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
