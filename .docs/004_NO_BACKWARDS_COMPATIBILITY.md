# NO BACKWARDS COMPATIBILITY - FAIL HARD AND FAST! 💥

## What Changed

**REMOVED all backwards compatibility.** Old-style task IDs will now **FAIL HARD** during PRD load.

## Validation Rules

### ✅ VALID (3-Tier Format)
```yaml
tasks:
  - id: "auth/frontend/1"
  - id: "search/backend/2"
  - id: "user-profile/api/42"
```

### ❌ INVALID (Will CRASH on load)
```yaml
tasks:
  - id: "task-001"        # REJECTED: Not 3-tier format
  - id: "backend-003"     # REJECTED: Not 3-tier format
  - id: "AUTH/frontend/1" # REJECTED: Uppercase feature
  - id: "auth/Frontend/1" # REJECTED: Uppercase discipline
  - id: "auth/frontend"   # REJECTED: Missing number
```

## Error Message

When loading a PRD with invalid task IDs, you'll see:

```
Failed to load PRD: Invalid task ID 'task-001' in PRD.
All task IDs must use 3-tier format (feature/discipline/number).
Invalid task ID format: task-001
```

**NO MIGRATION TOOL. NO FALLBACK. NO MERCY.** 🔥

## Implementation

### Backend Changes

**File:** `src-tauri/src/prd.rs`

Added validation in `PRD::from_file()`:
```rust
// CRITICAL: Validate ALL task IDs - FAIL HARD on invalid format
for task in &prd.tasks {
    Self::validate_task_id(&task.id).map_err(|e| {
        format!(
            "Invalid task ID '{}' in PRD. All task IDs must use 3-tier format (feature/discipline/number). {}",
            task.id, e
        )
    })?;
}
```

### Tests Updated

**Added 2 new tests:**
1. `test_rejects_old_style_ids` - Verifies old IDs are rejected ✅
2. `test_accepts_only_3tier_ids` - Verifies 3-tier IDs work ✅

**Removed:**
- Backwards compatibility test (replaced with rejection test)

### Fixtures Updated

**All 3 fixtures now use 3-tier IDs:**

1. **`fixtures/single-task/.ralph/prd.yaml`**
   - `task-001` → `hello/backend/1`

2. **`fixtures/3-tier-tasks/.ralph/prd.yaml`**
   - Already using 3-tier format ✅

3. **`fixtures/elaborate-prd/.ralph/prd.yaml`**
   - `backend-001` → `database/backend/1`
   - `backend-002` → `api/backend/1`
   - `frontend-001` → `catalog/frontend/1`
   - `backend-003` → `payment/backend/1`
   - `frontend-002` → `cart/frontend/1`
   - `infra-001` → `cicd/infrastructure/1`
   - `backend-004` → `inventory/backend/1`
   - `frontend-003` → `auth/frontend/1`
   - `backend-005` → `orders/backend/1`
   - `backend-006` → `search/backend/1`
   - `frontend-004` → `reviews/frontend/1`
   - `testing-001` → `checkout/testing/1`
   - `docs-001` → `api/documentation/1`
   - `feature-001` → `social/marketing/1`
   - `frontend-005` → `wishlist/frontend/1`
   - `backend-007` → `notifications/backend/1`
   - `frontend-006` → `admin/frontend/1`
   - `backend-008` → `shipping/backend/1`
   - `frontend-007` → `comparison/frontend/1`
   - `backend-009` → `promotions/backend/1`
   - `frontend-008` → `navigation/frontend/1`
   - `backend-010` → `analytics/backend/1`
   - `testing-002` → `payment/testing/1`
   - `frontend-009` → `orders/frontend/1`
   - `backend-011` → `database/infrastructure/1`
   - `security-001` → `security/backend/1`

**Also updated all `depends_on` references to use new IDs.**

## Test Results

```bash
cargo test prd::

running 5 tests
test prd::tests::test_counter_persistence ... ok
test prd::tests::test_generate_task_id ... ok
test prd::tests::test_validate_task_id ... ok
test prd::tests::test_rejects_old_style_ids ... ok
test prd::tests::test_accepts_only_3tier_ids ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**ALL TESTS PASS** ✅

## Why No Backwards Compatibility?

1. **No real users yet** - Only mock data/fixtures
2. **Clean slate** - Better to enforce standards from day one
3. **No technical debt** - No mixed ID states to maintain
4. **Simpler code** - No fallback logic cluttering the codebase
5. **Clear errors** - Users know exactly what's wrong

## Migration Path (If Needed)

For anyone with old PRD files:

1. Open `prd.yaml`
2. For each task, convert ID to format: `{feature}/{discipline}/{number}`
3. Add `_counters` section (will auto-rebuild on first load)
4. Update `depends_on` arrays to use new IDs

**Example:**
```yaml
# Before
- id: "backend-001"
  depends_on: ["backend-002"]

# After
- id: "database/backend/1"
  depends_on: ["api/backend/1"]
```

## Summary

✅ **Strict 3-tier validation added**
✅ **All fixtures updated**
✅ **All tests passing**
✅ **No backwards compatibility**
✅ **Fails fast with clear error messages**

**THIS IS THE ONLY WAY.** 💪
