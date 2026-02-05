# Bug Fixes and Gap Filling - yaml-db Crate

## Summary

Proactive review of the yaml-db crate identified and fixed several bugs and gaps in functionality.

## Issues Found and Fixed

### 1. ✅ Feature Name Normalization (FIXED)
**Issue:** `name_to_display_name()` only handled hyphens, not underscores
- `user_profile` → `User_profile` (incorrect)
- `user-profile_settings` → `User Profile_settings` (incorrect)

**Fix:** Updated to split on both `-` and `_`
- `user_profile` → `User Profile` ✓
- `user-profile_settings` → `User Profile Settings` ✓

**File:** `crates/yaml-db/src/features.rs:41`
**Tests:** `test_feature_name_with_underscores`, `test_feature_name_with_mixed_separators`

### 2. ✅ Missing CRUD Operations (ADDED)
**Issue:** Only CREATE was implemented, no UPDATE, DELETE, or GET_BY_ID

**Added Methods:**
- `YamlDatabase::get_task_by_id(id: u32) -> Option<&Task>`
- `YamlDatabase::update_task(id: u32, update: TaskInput) -> Result<(), String>`
- `YamlDatabase::delete_task(id: u32) -> Result<(), String>`

**Features:**
- Thread-safe with exclusive file locking
- Atomic writes using temp file pattern
- Preserves created timestamp and status on update
- Prevents deletion of tasks with dependents
- Validates dependencies on update

**File:** `crates/yaml-db/src/database.rs`
**Tests:** 14 comprehensive tests in `tests/crud_operations.rs`

### 3. ✅ Circular Dependency Detection (ADDED)
**Issue:** Tasks could create dependency cycles (A→B→C→A or A→A)

**Added Validation:**
- Self-referential dependency check (task depends on itself)
- Circular dependency detection using depth-first search
- Validates cycles when updating task dependencies

**Methods Added:**
- `validate_dependencies_with_cycles()` - validates without creating cycles
- `has_circular_dependency()` - recursive cycle detection

**File:** `crates/yaml-db/src/database.rs`
**Tests:**
- `test_update_task_creates_self_referential_dependency`
- `test_update_task_creates_circular_dependency`
- `test_update_task_complex_circular_dependency`

### 4. ✅ Whitespace Validation (ALREADY WORKING)
**Finding:** Existing validation already correctly handles whitespace-only strings
- Feature, discipline, and title validation uses `.trim().is_empty()`
- No bug found, added comprehensive tests to verify

**Tests:**
- `test_whitespace_only_feature_name`
- `test_whitespace_only_discipline_name`
- `test_whitespace_only_title`

## Test Coverage Summary

### Before
- 36 tests total
- No CRUD operation tests beyond CREATE
- No circular dependency tests
- No normalization edge case tests

### After
- **56 tests total** (+20 new tests)
- Full CRUD lifecycle coverage
- Circular dependency validation
- Feature name normalization edge cases
- All existing tests still pass

### Test Files
1. `tests/bug_fixes.rs` - 9 tests for discovered issues
2. `tests/crud_operations.rs` - 14 tests for new CRUD methods
3. `tests/edge_cases.rs` - 13 tests (existing)
4. `tests/transformation_tests.rs` - 6 tests (existing)
5. `tests/golden_tests.rs` - 12 tests (existing)
6. `tests/generate_snapshots.rs` - 2 tests (existing)

## Remaining Gaps (Known Limitations)

### 1. Entity Save Method Not Atomic
- `EntityFile::save()` writes directly without temp file pattern
- Only `save_to_temp()` + `commit_temp()` are atomic
- **Impact:** Low - YamlDatabase uses atomic pattern, direct save() not exposed

### 2. No Task Status Update Method
- Can update entire task, but no dedicated method to change status
- **Workaround:** Use `update_task()` with current values + new status
- **Recommendation:** Consider adding `update_task_status(id, status)`

### 3. No Bulk Operations
- No `update_many()`, `delete_many()`, or `create_many()` methods
- **Impact:** Low - current use case is single-task operations
- **Future Enhancement:** If needed for performance

## Files Modified

- `crates/yaml-db/src/features.rs` - Fixed normalization
- `crates/yaml-db/src/database.rs` - Added CRUD methods and cycle detection
- `crates/yaml-db/tests/bug_fixes.rs` - New test file
- `crates/yaml-db/tests/crud_operations.rs` - New test file

## Breaking Changes

None. All changes are additive (new methods) or bug fixes (normalization).

## Verification

All 56 tests pass:
```bash
cargo test --manifest-path crates/yaml-db/Cargo.toml
# Result: 56 passed; 0 failed
```

All PRD integration tests pass:
```bash
cargo test --manifest-path src-tauri/Cargo.toml prd
# Result: 3 passed; 0 failed
```
