# MCP Verb Consolidation — 30 Simulations Across 4 Independent Contexts

**Created:** 2026-02-11
**Source:** 30 simulations from 4 independent Haiku/Opus agents (no cross-contamination)

## Raw Verb Inventory

### Batch A (Opus, sims 1-15) — 19 unique verbs
`im_done`, `partially_done`, `im_stuck`, `learned`, `decided`, `register_file`, `flag_issue`, `flag_ambiguity`, `flag_overlap`, `flag_abstraction_mismatch`, `flag_contradiction`, `stale_context`, `report_bug`, `suggest_task`, `need_env`, `need_dependency`, `need_service`, `request_config`, `stub_until_verified`, `warn_breaking_change`

### Batch B (Haiku, sims 16-20) — 16 unique verbs
`flag_task_contradiction`, `report_unmapped_dependency`, `request_clarification`, `report_missing_artifact`, `flag_environment_config_mismatch`, `suggest_refactoring`, `report_concurrency_hazard`, `request_architectural_decision`, `warn_dependency_chain_impact`, `flag_codebase_inconsistency`, `escalate_to_human`, `block_on_prior_task`, `report_abstraction_limitation`, `report_missing_dependencies`, `suggest_prerequisite_task`, `request_test_strategy`

### Batch C (Haiku, sims 21-25) — 24 unique verbs
`flag_incomplete_dependency`, `request_task_context_update`, `escalate_for_human_review`, `report_dependency_version_mismatch`, `query_project_constraints`, `record_version_incompatibility`, `request_architecture_guidance`, `propose_task_split`, `flag_design_gap`, `detect_data_race_condition`, `request_prior_task_completion_check`, `propose_idempotency_tracking`, `detect_spec_code_divergence`, `request_requirement_clarification`, `record_task_metadata_stale`

### Batch D (Haiku, sims 26-30) — 24 unique verbs
`block_on_dependency`, `verify_external_resource`, `pause_and_escalate`, `detect_schema_conflict`, `query_branch_status`, `request_design_review`, `suggest_dependency_ordering`, `flag_performance_concern`, `request_acceptance_criteria_clarification`, `suggest_alternative_approach`, `dependency_request`, `detect_dependency_version_mismatch`, `verify_upstream_completion`, `request_environment_sync`, `suggest_rework_scope`, `architecture_conflict`, `request_permission_check`, `propose_refactoring`, `suggest_workaround`, `document_design_decision`

**Total raw verbs: ~83 unique names across 30 simulations**

## Consolidation: What Agents Actually Need

Looking at the 83 verbs, they collapse into **7 natural categories**:

### 1. COMPLETION — "I'm done / I'm stuck"
**Raw verbs:** `im_done`, `partially_done`, `im_stuck`
**Frequency:** Every single simulation ends with one of these.
**Consolidation:** These are distinct intents. Keep all 3.

| Verb | Params | When |
|------|--------|------|
| `done` | `summary` | Task fully complete |
| `stuck` | `reason` | Can't continue, here's why |
| `partial` | `summary`, `remaining` | Did what I could, here's what's left |

### 2. KNOWLEDGE — "I learned something reusable"
**Raw verbs:** `learned`, `decided`, `document_design_decision`
**Frequency:** `learned` in 25/30 sims. `decided` in 22/30 sims. These are the #1 and #2 most common verbs after `done`.
**Consolidation:** `decided` IS a type of learning — it's a learning with a rationale and alternatives rejected. Merge into one verb with a `kind` field.

| Verb | Params | When |
|------|--------|------|
| `learned` | `text`, `kind` (discovery/decision/convention) | Reusable knowledge for future tasks |

Or keep `decided` separate because it carries `rationale` and `alternatives_rejected`:

| Verb | Params | When |
|------|--------|------|
| `learned` | `text` | Factual discovery about the codebase |
| `decided` | `decision`, `rationale` | I made a judgment call |

### 3. FLAG — "Something is wrong / weird / contradictory"
**Raw verbs (22 variants!):**
- `flag_issue`, `flag_ambiguity`, `flag_overlap`, `flag_abstraction_mismatch`, `flag_contradiction`, `flag_task_contradiction`, `flag_environment_config_mismatch`, `flag_codebase_inconsistency`, `flag_incomplete_dependency`, `flag_design_gap`, `flag_performance_concern`
- `report_bug`, `report_unmapped_dependency`, `report_missing_artifact`, `report_concurrency_hazard`, `report_abstraction_limitation`, `report_missing_dependencies`, `report_dependency_version_mismatch`
- `detect_data_race_condition`, `detect_spec_code_divergence`, `detect_schema_conflict`, `detect_dependency_version_mismatch`
- `stale_context`, `warn_breaking_change`, `warn_dependency_chain_impact`, `architecture_conflict`

**Frequency:** At least one `flag/report/detect/warn` in 28/30 sims.

**Key insight:** Agents named these differently across contexts (`flag_*` vs `report_*` vs `detect_*` vs `warn_*`) but they all mean the same thing: **"I found a problem."** The only meaningful distinction is severity/urgency.

**Consolidation:** ONE verb with structured params:

| Verb | Params | When |
|------|--------|------|
| `flag` | `what` (description), `severity` (info/warning/blocking), `category` (bug/ambiguity/contradiction/overlap/stale/performance/security) | Any problem discovered during execution |

### 4. SUGGEST — "Here's what should happen next"
**Raw verbs:**
- `suggest_task`, `suggest_prerequisite_task`, `suggest_refactoring`, `suggest_alternative_approach`, `suggest_workaround`, `suggest_dependency_ordering`, `suggest_rework_scope`
- `propose_task_split`, `propose_refactoring`, `propose_idempotency_tracking`

**Frequency:** At least one `suggest/propose` in 18/30 sims.

**Key insight:** All of these are "I think a task should exist" or "I think the approach should change." The agent SUGGESTS, the orchestrator/human DECIDES.

**Consolidation:** ONE verb:

| Verb | Params | When |
|------|--------|------|
| `suggest` | `what` (description), `kind` (new_task/split/refactor/alternative/workaround), `why` | Agent recommends an action it can't take itself |

### 5. NEED — "I'm blocked on something external"
**Raw verbs:**
- `need_env`, `need_service`, `need_dependency`, `request_config`
- `block_on_dependency`, `block_on_prior_task`, `dependency_request`
- `verify_external_resource`, `verify_upstream_completion`
- `request_environment_sync`, `request_permission_check`

**Frequency:** At least one `need/block/request` in 15/30 sims.

**Key insight:** These are all "I can't proceed because something OUTSIDE my control is missing." The distinction between needing a secret, a service, a dependency, or a completed upstream task doesn't matter to the verb — the params carry that info.

**Consolidation:** ONE verb:

| Verb | Params | When |
|------|--------|------|
| `blocked` | `on` (description), `kind` (secret/service/dependency/upstream_task/config/infra), `who_can_help` (optional) | External blocker, agent can't resolve |

### 6. ASK — "I need a decision from someone smarter"
**Raw verbs:**
- `request_clarification`, `request_architectural_decision`, `request_design_review`, `request_acceptance_criteria_clarification`, `request_requirement_clarification`, `request_test_strategy`, `request_architecture_guidance`
- `escalate_to_human`, `escalate_for_human_review`, `pause_and_escalate`
- `query_project_constraints`, `query_branch_status`

**Frequency:** At least one `request/escalate/query` in 16/30 sims.

**Key insight:** The agent wants to ASK a question and get an answer. Sometimes it's asking the orchestrator, sometimes a human, sometimes the project itself. But the verb is always "I have a question that blocks me."

**Consolidation:** ONE verb:

| Verb | Params | When |
|------|--------|------|
| `ask` | `question`, `options` (if known), `preferred` (if agent has opinion), `blocking` (bool) | Agent needs a decision or clarification |

### 7. REGISTER — "This file/context is now relevant"
**Raw verbs:** `register_file`, `request_task_context_update`, `record_version_incompatibility`, `record_task_metadata_stale`

**Frequency:** `register_file` in 20/30 sims. Others rare.

**Consolidation:** ONE verb:

| Verb | Params | When |
|------|--------|------|
| `register` | `path` (file path), `why` (optional) | A new/modified file that future tasks should know about |

## The Final 9 Verbs

| # | Verb | Category | Frequency | Purpose |
|---|------|----------|-----------|---------|
| 1 | `done` | Closing | 30/30 | Task complete |
| 2 | `stuck` | Closing | 5/30 | Can't continue |
| 3 | `partial` | Closing | 6/30 | Did what I could |
| 4 | `learned` | Knowledge | 25/30 | Reusable discovery |
| 5 | `decided` | Knowledge | 22/30 | Judgment call with rationale |
| 6 | `flag` | Signaling | 28/30 | Found a problem |
| 7 | `suggest` | Signaling | 18/30 | Recommend an action |
| 8 | `blocked` | Signaling | 15/30 | External blocker |
| 9 | `register` | Metadata | 20/30 | File is now relevant |

**83 raw verbs → 9 consolidated verbs.** And because the MCP server is pre-bound per-task, none of these need a `task_id` parameter — it's baked in.

## What's NOT Here

Things the agent never asked for:
- **Read operations** — agents never wished for `get_task`, `list_features`, etc. The prompt gave them context. They used Claude's native tools for code.
- **CRUD on other entities** — no agent wanted to `update_feature` or `create_discipline`. They only wanted to signal.
- **Admin operations** — no `delete_task`, no `set_task_status` on OTHER tasks. Only their own status via `done`/`stuck`/`partial`.

This validates the "exhaust pipe" hypothesis: **the MCP server is output-only from the agent's perspective.** All input comes from the prompt.
