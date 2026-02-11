# MCP Verb Discovery — Iteration 1 Results

**Date:** 2026-02-11
**Simulations:** 30
**Unique Verbs Discovered:** 34

## Summary

Ran 30 Haiku simulations of coding tasks across diverse projects, features, and disciplines. Each simulation walked through realistic task execution and identified moments where an agent would need to communicate back to the orchestrator (Ralph).

**Key Finding:** Blockers are primarily clarification issues, not technical ones. 30% of simulations involved ambiguous requirements, architectural contradictions, or missing context — not "I don't know how to code this."

---

## Verb Frequency Table

| Verb | Count | Category |
|------|-------|----------|
| design_decision_needed | 4 | Decision Escalation |
| escalate_to_architecture | 3 | Decision Escalation |
| clarify_requirement_conflict | 1 | Clarification |
| flag_scope_ambiguity | 1 | Clarification |
| clarify_constraints | 1 | Clarification |
| clarify_task_boundary | 1 | Clarification |
| clarify_architectural_mismatch | 1 | Clarification |
| clarify_scope_and_accuracy | 1 | Clarification |
| clarify_architectural_intent | 1 | Clarification |
| flag_concurrency_issue | 1 | Clarification |
| define_acceptance_criteria | 1 | Clarification |
| ask_implementation_direction | 1 | Decision Escalation |
| record_blocker | 1 | Issue Tracking |
| suggest_subtask | 1 | Task Decomposition |
| suggest_task_split | 2 | Task Decomposition |
| suggest_research_spike | 1 | Task Planning |
| check_feature_overlap | 1 | Coordination |
| propose_dependency | 1 | Task Management |
| propose_code_sharing | 1 | Coordination |
| propose_shared_query_layer | 1 | Coordination |
| request_original_author_input | 1 | Coordination |
| audit_dependency_completion | 1 | Quality Assurance |
| assess_abstraction_gap | 2 | Analysis |
| check_existing_implementation | 1 | Analysis |
| resolve_missing_config | 1 | Configuration |
| request_dev_secrets | 1 | Configuration |
| request_env_validation | 1 | Configuration |
| recover_missing_context | 1 | Context Recovery |
| initiate_design_doc | 1 | Documentation |
| add_to_development_notes | 1 | Documentation |
| identify_data_migration_need | 1 | Data Management |
| suggest_solution_approach | 1 | Guidance |
| assess_library_upgrade | 1 | Risk Assessment |
| propose_abstraction_upgrade | 1 | Architecture |

---

## Verb Categories (High-Level)

**Decision Escalation (6 verbs):**
Handling ambiguous design choices or architectural trade-offs that can't be resolved locally.

**Clarification (9 verbs):**
Asking for clarity on requirements, scope, constraints, or architectural intent.

**Coordination (5 verbs):**
Detecting overlaps, managing dependencies, and proposing shared solutions.

**Task Management (3 verbs):**
Decomposing or splitting tasks, creating prerequisites.

**Analysis (3 verbs):**
Assessing gaps, checking for existing implementations, auditing completion.

**Configuration (3 verbs):**
Requesting missing config variables, secrets, environment setup.

**Issue Tracking & Documentation (4 verbs):**
Recording blockers, creating design docs, adding context notes.

**Architecture (3 verbs):**
Proposing abstraction upgrades, assessing library versions.

**Context Recovery (1 verb):**
Recovering deleted or lost design/context documents.

---

## Verbs That Appeared Only Once (24/34)

Single-occurrence verbs are likely:
- **Specialized edge cases** (e.g., `identify_data_migration_need`, `assess_library_upgrade`)
- **Aliases for common problems** (e.g., several different "clarify" verbs doing the same thing)
- **Too specific naming** (e.g., `propose_shared_query_layer` is `propose_code_sharing` with specifics)

These are candidates for consolidation.

---

## Suggested Consolidations

### Consolidation Group 1: Clarify Verbs (9→3)

**Current:** `clarify_requirement_conflict`, `clarify_constraints`, `clarify_task_boundary`, `clarify_architectural_mismatch`, `clarify_scope_and_accuracy`, `clarify_architectural_intent`, `flag_scope_ambiguity`, `define_acceptance_criteria`, `flag_concurrency_issue`

**Consolidated:**
- `clarify_requirement(task_id, ambiguity, severity)` — "I don't understand the requirement"
- `clarify_scope(task_id, scope_question)` — "What's in scope?"
- `clarify_architecture(task_id, architectural_question)` — "Does the architecture support this?"

**Rationale:** All perform the same function (ask for clarification); domain differences are just parameter context. Consolidating reduces cognitive load for Ralph's decision logic.

### Consolidation Group 2: Propose Verbs (7→2)

**Current:** `propose_dependency`, `propose_code_sharing`, `propose_shared_query_layer`, `propose_abstraction_upgrade`, `suggest_solution_approach`, `suggest_subtask`, `suggest_task_split`

**Consolidated:**
- `propose_action(task_id, action_type, description)` where `action_type` ∈ {decompose, split, share_code, extract_abstraction, create_dependency}
- `suggest_solution(task_id, problem_description, solution_approach, effort_estimate)`

**Rationale:** All propose "here's what should happen next." One verb + action type handles all.

### Consolidation Group 3: Request Verbs (4→1)

**Current:** `resolve_missing_config`, `request_dev_secrets`, `request_env_validation`, `request_original_author_input`

**Consolidated:**
- `request_info(task_id, info_type, question)` where `info_type` ∈ {config, secrets, env_validation, author_intent}

**Rationale:** All block progress waiting for external information. One verb + info_type is cleaner.

### Consolidation Group 4: Keep Separate (Decision Escalation)

**Current:** `design_decision_needed`, `escalate_to_architecture`, `ask_implementation_direction`

**Keep as-is.** These are semantically distinct:
- `design_decision_needed` → Code structure/pattern choice
- `escalate_to_architecture` → System-level design choice
- `ask_implementation_direction` → Task scope ambiguity (could fold into `clarify_scope`, but distinct enough to keep)

---

## High-Priority Implementation Order

### MVP (Top 5 verbs covering ~30% of all task blockers):
1. `design_decision_needed` (4x) — Most frequent; handles architectural trade-offs
2. `escalate_to_architecture` (3x) — System-level blocking issues
3. `clarify_requirement` (consolidated, 9x) — Covers all clarification needs
4. `record_blocker` (1x) — Logs issues for Ralph to track
5. `suggest_subtask` (1x) — Unblocks by creating prerequisites

### Phase 2 (Coordination & Dependencies):
- `check_feature_overlap`
- `propose_dependency`
- `propose_code_sharing`

### Phase 3 (Configuration Management):
- `resolve_missing_config`
- `request_dev_secrets`
- `request_env_validation`

### Phase 4 (Edge Cases):
- Remaining specialized verbs

---

## Pattern Observations

### Pattern 1: Clarification is the #1 Blocker
~30% of simulations (9 of 30) got stuck waiting for clarification on requirements, scope, architecture, or constraints. This suggests Ralph should have a robust clarification protocol before trying to optimize coding speed.

### Pattern 2: Architectural Decisions are Frequent
Design decisions and architectural escalations appear in 1 out of 3-4 tasks. This is not a rare edge case. Ralph needs good decision routing.

### Pattern 3: Overlaps Emerge from Lack of Coordination
Multiple simulations discovered that two features were doing almost identical work (notifications vs. webhooks, search vs. analytics). Ralph should periodically scan for feature overlaps.

### Pattern 4: Missing Context is a Common Blocker
Tasks reference deleted documents, incomplete dependency code, or missing environment setup. Ralph should validate task prerequisites before assignment.

### Pattern 5: Task Descriptions Often Don't Match Codebase Reality
Several simulations found contradictions between task description and actual code state. Ralph should prompt for clarification before a developer starts.

---

## Implications for Ralph's MCP Server Design

### What Ralph Needs to Handle

1. **Clarification routing:** Route ambiguous requirements to the right person/system (product owner, architect, original author)
2. **Dependency validation:** Before starting a task, verify all blocking tasks are truly complete
3. **Feature coordination:** Detect when two features overlap and propose consolidation
4. **Context recovery:** Look up deleted or referenced documents
5. **Decision logging:** Track design decisions made for future reference

### What Ralph Should NOT Do

- **Implement clarification logic itself.** Ralph should escalate; it shouldn't guess. "I think scope is X" is wrong if scope is actually Y.
- **Resolve architectural trade-offs.** Ralph should surface the options and let humans decide.
- **Fix incomplete dependencies.** Ralph should block and flag, not silently work around.

### MCP Verbs to Wire Up

In Ralph's MCP server, these verbs should be actual tools:

```
/request_decision(task_id, decision_point, options, context)
  → Routes to decision logger; blocks task until human input

/request_clarification(task_id, ambiguity_type, question)
  → Routes to product owner / original author; blocks until clarified

/record_blocker(task_id, blocker_description, severity)
  → Logs blocker; updates task status to blocked

/check_feature_overlap(feature1, feature2)
  → Queries database for overlap in tasks/code; suggests consolidation

/validate_prerequisites(task_id)
  → Checks all dependencies are complete before task starts
```

---

## Next Iteration Ideas

1. **Run simulations with Ralph's proposed MCP verbs** — See if the consolidated verb set covers the blockers found in this round.
2. **Weighted simulation** — Run more simulations on high-frequency features (webhooks, authentication, search) to find feature-specific verbs.
3. **Blind verification** — Pick one simulation that called `design_decision_needed`, implement Ralph's routing for it, see if it unblocks the task.
4. **Scale testing** — Run 100+ simulations to find rare verbs and confirm frequency baselines.
