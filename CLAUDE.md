# CLAUDE.md

**Ralph4days** — Tauri 2.5 desktop app for autonomous multi-agent build loops. Runs Claude Haiku in a loop to complete PRD tasks with periodic Opus reviews.

## CRITICAL: Single Execution Path Policy

**No parallel implementations, feature flags, view toggles, or alternate modes.** When asked to "add a view" or "add a mode", interpret as: same data/logic/path, different presentation only. Consolidate or delete duplicates immediately. This prevents reward hacking and feature bloat.

## Dev Environment

Intel NUC12WSKi5 (i5-1240P, 64GB RAM, RTX 3090, Ultramarine Linux 43 Wayland/KDE). Builds target Alder Lake x86_64.

## Commands

Use `just --list` for all commands. Key ones: `just dev`, `just test`, `just build`, `just lint`, `just fmt`. Run built app: `ralph` (picker) or `ralph --project /path` (locked). Pre-commit hook runs oxlint and biome on staged files.

**Testing workflow**: `just reset-mock` creates gitignored `mock/` from `fixtures/` (renames `.undetect-ralph/` to `.ralph/`). Use `just dev-mock <project>` or `ralph --project mock/<project>` for testing. Fixtures stay clean, mock is disposable.

## Architecture

Frontend (React 19/Zustand) → IPC → Backend (Tauri/Rust: loop_engine, claude_client, prompt_builder) → subprocess → Claude CLI (--output-format stream-json, --max-turns 50)

Key files: `src-tauri/src/{loop_engine,claude_client,prompt_builder,commands}.rs`, `src/{components,stores}/`, `.specs/`

## Loop States

Idle → Running ↔ Paused. Running → RateLimited (5min retry) → Running/Aborted. Running → Complete (all tasks done) or Aborted (stop/stagnation). Stagnation = 3 iterations with no progress.txt/prd.yaml changes.

## Target Project Structure

Projects need `.ralph/` with `prd.yaml` (REQUIRED), `CLAUDE.RALPH.md` (recommended), `progress.txt`, `learnings.txt`. On loop start: backup `CLAUDE.md`, copy `CLAUDE.RALPH.md` to `CLAUDE.md`. On stop: restore backup. See SPEC-030 for details.

## Project Locking

ONE project per session, chosen at startup. CLI mode (`ralph --project /path`) validates and locks immediately. Interactive mode (`ralph`) shows ProjectPicker modal (scans home for `.ralph/` folders, 5 levels, max 100). Validation: path exists, has `.ralph/prd.yaml`. Commands: `validate_project_path`, `set_locked_project`, `get_locked_project`, `start_loop` (no path param).

## Implementation Notes

- **Timeout**: Uses system `timeout` command (900s default) wrapping Claude CLI subprocess
- **Rate Limits**: Parses JSON stream for `overloaded_error`/`rate_limit_error` event types
- **Prompts**: Inline file contents (no @file syntax): `PRD:\n{prd}\n\nProgress:\n{progress}\n\nLearnings:\n{learnings}`
- **Stagnation**: SHA256 hash of progress.txt + prd.yaml before/after iteration; abort after 3 unchanged iterations

## Tech Stack

**Frontend:** React 19, TypeScript, Vite, Tailwind v4, Zustand, Lucide Icons
**Backend:** Tauri 2.5, Rust, Tokio
**Testing:** Vitest, Playwright, Gremlins.js (unit/e2e/visual/chaos)
**Build:** bun, Cargo
**MCP:** shadcn-ui, tailwindcss (CSS converter broken but irrelevant), sequential-thinking

Specs in `.specs/` (read `000_SPECIFICATION_FORMAT.md` first). Tests: `just test` or specific `test-{rust,frontend,e2e,visual,monkey}`.

## UI Components

**ALWAYS use components from `src/components/ui/` instead of creating custom divs.** We have 50+ shadcn components (Badge, Button, Card, Dialog, Input, Select, etc.) that save time and tokens. Check `src/components/ui/` before writing custom markup. Key components: Badge (variants: default, secondary, outline, destructive), Button (variants + sizes), Card, Alert, Dialog, Sheet, ScrollArea, Tooltip, Separator.

## Environment

Claude CLI required (`claude --version`). Projects need `.ralph/prd.yaml`. Loop runs in project dir. Commits happen in Claude CLI sessions (not managed by Ralph).
