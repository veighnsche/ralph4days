# Error Handling Audit and Fixes

**Date:** 2026-02-08
**Status:** In Progress

## Summary

Comprehensive audit of error handling across the codebase revealed critical gaps where panics can occur. This document tracks all issues found and fixes implemented.

## Critical Issues Found

### 1. Database Layer Unwraps (CRITICAL - Can Panic)

**File:** `crates/sqlite-db/src/tasks.rs`, `features.rs`, `disciplines.rs`, `comments.rs`, `stats.rs`

**Problem:**
- `prepare()` calls unwrap - SQL syntax errors will panic
- `query_map()` calls unwrap - execution errors will panic
- `row.get()` calls unwrap - type mismatches or wrong indices will panic
- `serde_json::to_string()` unwraps - serialization failures will panic

**Impact:** Any DB corruption, schema migration issue, or type mismatch causes app crash instead of graceful error.

**Fix Required:**
- Wrap all `prepare()` in `map_err()` with DB_READ code
- Handle all `query_map()` failures
- Never unwrap `row.get()` - use `?` operator
- Handle JSON serialization failures

### 2. Missing Input Validation

**File:** Multiple command files

**Problem:**
- No max length validation on strings (title, description, etc.)
- No validation on path traversal in file paths beyond basic ".." check
- No validation on color hex codes in disciplines
- No validation on icon names
- No limits on array sizes (tags, depends_on, etc.)

**Impact:** DOS attacks, memory exhaustion, invalid data in DB.

**Fix Required:**
- Add max length constants and validate
- Validate color hex format
- Validate icon names against allowed set
- Add max array size limits

### 3. File System Operations Not Handled

**File:** `src-tauri/src/commands/project.rs`

**Problem:**
- `canonicalize()` can fail (permission denied, path doesn't exist)
- `create_dir()` can fail (permission denied, disk full)
- `write()` can fail (permission denied, disk full)

**Impact:** Unclear error messages, potential panics.

**Fix Required:** Already mostly handled, but verify all paths.

### 4. Missing Tests

**Current Coverage:**
- sqlite-db: 0 unit tests for CRUD operations
- Commands: 0 integration tests
- Error codes: 0 tests verifying format
- Terminal: 2 basic tests, no error path testing

**Required Tests:**
- Task CRUD with error cases
- Feature/Discipline CRUD with error cases
- Validation failures
- Circular dependency detection
- Error code format verification
- DB corruption scenarios
- Concurrent access tests

### 5. Race Conditions

**File:** `crates/sqlite-db/src/` (all files)

**Problem:**
- No transaction usage for multi-step operations
- Task update doesn't use transactions
- Circular dependency check not atomic

**Impact:** Data corruption in concurrent scenarios.

**Fix Required:**
- Wrap multi-step operations in transactions
- Add tests for concurrent access

## Error Code Coverage Gaps

Missing codes for:
- R-2000: SQL prepare failures (only handles open/migration)
- R-2100: SQL query execution failures (only handles some queries)
- No codes for JSON serialization failures

## Implementation Plan

### Phase 1: Critical Fixes (Database Layer) ✅ COMPLETE
**Completed:** 2026-02-08
**Details:** See `.docs/031_PHASE_1_DATABASE_UNWRAP_FIXES.md`

1. ✅ Remove all `.unwrap()` from prepare/query_map/row.get
2. ✅ Add proper error codes for all DB operations
3. ✅ Handle JSON serialization failures
4. ⏭️ Add unit tests for each DB method (deferred to Phase 3)

### Phase 2: Input Validation
1. Add validation constants (MAX_TITLE_LEN, etc.)
2. Implement validation functions
3. Add validation tests

### Phase 3: Comprehensive Testing
1. Unit tests for all DB methods
2. Integration tests for all commands
3. Error path tests
4. Concurrent access tests
5. Fuzzing tests for validation

### Phase 4: Race Condition Fixes
1. Add transaction support
2. Wrap multi-step ops in transactions
3. Add concurrent access tests

## Test Coverage Goals

- **sqlite-db**: 100% of public methods tested
- **Commands**: All error paths tested
- **Error codes**: All codes verified in tests
- **Validation**: All validation rules tested
- **Concurrency**: Basic race condition tests

## Files to Modify

### High Priority (Can Panic)
- `crates/sqlite-db/src/tasks.rs` - 50+ unwraps
- `crates/sqlite-db/src/features.rs` - 20+ unwraps
- `crates/sqlite-db/src/disciplines.rs` - 15+ unwraps
- `crates/sqlite-db/src/comments.rs` - 10+ unwraps
- `crates/sqlite-db/src/stats.rs` - 10+ unwraps

### Medium Priority (Missing Validation)
- `src-tauri/src/commands/tasks.rs`
- `src-tauri/src/commands/features.rs`
- `src-tauri/src/commands/disciplines.rs`

### Low Priority (Hardening)
- Add transaction support
- Add fuzzing tests
- Add performance tests
