# MCP Verb Discovery — Haiku Simulation Prompt

**Created:** 2026-02-11
**Purpose:** Run cheap Haiku simulations to discover what MCP tools an agent naturally reaches for during task execution. Each run generates a random project/feature/task/discipline context and the agent "hallucinates" tool calls it wishes existed.

## The Prompt

```
You are Claude, an AI coding assistant. You have been spawned by an orchestrator called Ralph to execute a single coding task. You have access to standard coding tools (read files, write files, run commands, search code). You do NOT have any tools for communicating back to Ralph — but you WISH you did.

Your job: mentally simulate executing the task below. Walk through what you would do step by step. When you hit a moment where you need to SIGNAL something back to the orchestrator — a completion, a problem, a discovery, a decision, a suggestion — describe the tool call you WISH you could make. Use whatever verb name feels most natural. Don't worry about matching an existing API. Just describe what you'd call and what you'd pass.

## Your Context

**Project:** {{project_name}} — {{project_description}}
**Tech stack:** {{tech_stack}}
**Feature:** "{{feature_name}}" — {{feature_description}}
**Discipline:** {{discipline_name}} ({{discipline_skills}})
**Your task:** "{{task_title}}" — {{task_description}}

**What's already done in this feature:**
{{completed_tasks}}

**What's pending after you:**
{{pending_tasks}}

**Feature learnings so far:**
{{learnings}}

## Rules

1. Walk through the task step by step as if you're actually doing it
2. When you encounter something that makes you want to communicate back to the orchestrator, write a tool call block like this:

→ TOOL: verb_name
  param: value
  param: value
  WHY: one sentence explaining why you need this

3. Be specific about WHAT went wrong or WHAT you discovered — don't be vague
4. You WILL encounter at least one complication (ambiguity, missing context, blocking issue, or unexpected codebase state). Simulate it realistically.
5. End with either a completion signal or a stuck signal
6. Keep it under 500 words total
```

## The Randomizer

Generate contexts by mixing from these pools:

### Projects
- Recipe sharing app (Flutter/Dart)
- Internal HR portal (Next.js/TypeScript)
- IoT sensor dashboard (Rust/Axum + HTMX)
- E-commerce marketplace (Rails/Ruby)
- CLI database migration tool (Go)
- Social media scheduler (Python/FastAPI)
- Multiplayer game lobby (Elixir/Phoenix)
- Personal finance tracker (SvelteKit/TypeScript)
- Document collaboration tool (React/Node)
- CI/CD pipeline manager (Rust CLI)
- Podcast hosting platform (Django/Python)
- Fleet management system (C#/.NET)
- Code review bot (TypeScript/GitHub API)
- Weather station network (Embedded C + Python backend)
- Learning management system (Laravel/PHP)
- Real estate listing aggregator (Go/Templ)
- Music production DAW plugin (C++/JUCE)
- Inventory management (React Native/Expo)
- Video transcription service (Python/Whisper)
- Smart home automation (Kotlin/Spring)

### Features (mix and match with any project)
- user-authentication — login, signup, password reset, session management
- search — full-text search with filters and faceted results
- notifications — real-time and batched notification delivery
- payments — payment processing with multiple provider support
- data-export — export user data in multiple formats (CSV, JSON, PDF)
- audit-log — immutable log of all system actions
- role-permissions — granular role-based access control
- file-uploads — multi-part upload with progress and validation
- analytics-dashboard — usage metrics and visualization
- api-rate-limiting — per-user and per-endpoint rate limiting
- onboarding-wizard — multi-step guided setup flow
- offline-sync — local-first data with conflict resolution
- multi-tenancy — tenant isolation and switching
- i18n — internationalization with dynamic locale loading
- webhook-system — configurable outbound webhook delivery
- caching-layer — multi-tier cache with invalidation strategies
- migration-system — schema versioning and rollback
- plugin-architecture — extensible plugin loading and lifecycle
- batch-processing — async job queue with progress tracking
- accessibility — WCAG 2.1 AA compliance across all views

### Disciplines
- frontend (React, CSS, accessibility, component architecture)
- backend (API design, database, business logic, error handling)
- fullstack (end-to-end implementation, integration)
- data (analytics, ETL, data modeling, SQL)
- devops (CI/CD, infrastructure, monitoring, deployment)
- security (auth, encryption, vulnerability scanning, compliance)
- testing (unit tests, integration tests, e2e, test infrastructure)
- documentation (API docs, user guides, architecture diagrams)

### Complications (inject one per simulation)
- A dependency task marked "done" left incomplete or broken code
- The task description contradicts what the codebase actually does
- A required API/service/secret is missing from the environment
- The task is actually two separate tasks that should be split
- An existing abstraction doesn't support what the task needs
- The feature's design has a gap that no task covers
- A race condition or edge case makes the naive approach fail
- The task's acceptance criteria are ambiguous or contradictory
- A third-party library has a breaking change since the last task
- The file referenced in context_files no longer exists or was refactored
- Performance requirements make the obvious approach unviable
- The task overlaps with work done in a different feature

## Output Format

Each simulation produces a structured block:

```
## Simulation #N
Project: ...
Feature: ...
Discipline: ...
Task: ...
Complication: ...

### Walkthrough
[step-by-step narrative, 200-400 words]

### Tool Calls Made
1. → TOOL: verb(params) — WHY: ...
2. → TOOL: verb(params) — WHY: ...
...

### Verbs Used
[list of unique verb names from this simulation]
```

## Running the Simulations

Run 30+ simulations. After all simulations, produce a summary:

```
## Verb Frequency Table
| Verb | Count | Typical Params | Category |
|------|-------|----------------|----------|
| ... | ... | ... | ... |

## Verbs That Appeared Only Once
[list — these might be too specific or might be aliases]

## Suggested Consolidations
[verbs that mean the same thing but were named differently across simulations]
```
