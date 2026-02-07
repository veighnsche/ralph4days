# TEAM_001: Bug - Comment editing doesn't persist in UI

## Bug Report
- Reported behavior: Editing a comment on task id 1 in mock project 03-with-tasks-project doesn't work. Edit buttons respond but comment text doesn't change. No console error logged.
- Expected behavior: After editing a comment, the updated text should be displayed.
- Reproduction steps: Open task detail for task 1, click edit pencil on a comment, change text, click Save.

## Reproduction Results
- Successfully reproduced: Yes (via code analysis)
- Environment: Tauri 2.10.2, React 19, TanStack Query
- Minimal repro case: Any comment edit in any task detail tab

## Hypotheses

### H1: TaskDetailTabContent reads stale snapshot from Zustand store, not from React Query cache
- Evidence criteria: TaskDetailTabContent reads `task` from `tab.data?.entity` (Zustand), not from a `useInvoke`/`useQuery` hook that subscribes to `get_enriched_tasks`
- Test method: Read TaskDetailTabContent.tsx and check data source
- Status: CONFIRMED
- Findings: Line 13 reads `const task = tab.data?.entity as EnrichedTask | undefined;` — this is a static snapshot stored in Zustand when the tab is opened (TasksPage.tsx line 86). After any mutation (add/edit/delete comment), CommentsSection invalidates `get_enriched_tasks`, but TaskDetailTabContent doesn't subscribe to that query. The refetch updates the tasks list page but NOT the detail tab.

### H2: IPC argument name mismatch (camelCase vs snake_case)
- Evidence criteria: Frontend sends camelCase args but Rust expects snake_case
- Test method: Check Tauri 2 command argument naming behavior
- Status: RULED_OUT
- Findings: Tauri 2.10.2 auto-converts camelCase JS args to snake_case Rust params. Confirmed by `create_task` working with `dependsOn` mapping to `depends_on`.

### H3: SQL UPDATE not persisting or wrong column mapping
- Evidence criteria: SQL has wrong column names or doesn't commit
- Test method: Read comments.rs UPDATE query + check test
- Status: RULED_OUT
- Findings: `test_update_comment_by_id` passes. SQL is correct: `UPDATE task_comments SET body = ?1 WHERE id = ?2 AND task_id = ?3`. WAL mode commits immediately.

## Root Cause
`TaskDetailTabContent` reads the task object from `tab.data?.entity` in the Zustand workspace store. This is a static snapshot set when the tab is opened (TasksPage.tsx line 86: `data: { entityId: task.id, entity: task }`). It never updates.

After a comment mutation, `CommentsSection` correctly invalidates the `get_enriched_tasks` React Query cache key, triggering a refetch. However, `TaskDetailTabContent` does NOT subscribe to this query — it reads from Zustand. So the refetch updates the list page but the detail tab shows stale data.

Additionally, even though the mutation fires `onSuccess` (closing the edit form), the displayed comment text comes from the stale `task.comments` array, making it look like the edit was silently lost.

## Fix Assessment
- Units of Work: 1 (modify TaskDetailTabContent.tsx to read from React Query cache)
- Lines Changed: ~10
- Risk Level: Low — isolated to one component, uses existing hooks
- Decision: IMMEDIATE_FIX

## Resolution
Modified `TaskDetailTabContent` to use `usePRDData()` hook and look up the task by `entityId` from the React Query cache, falling back to the static snapshot. This ensures comment mutations (and any other task mutations) are reflected in real-time.

## Lessons Learned
- Tab-based UIs that store entity snapshots in tab state need a strategy for keeping data fresh. The pattern should be: store the entity ID in tab state, read the full entity from the query cache.
- Silent failures (mutation succeeds, UI doesn't update, no error) are the worst UX — add defensive logging or toast messages for "no visible change" scenarios.
