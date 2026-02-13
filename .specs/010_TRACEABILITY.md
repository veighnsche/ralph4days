# Traceability Standard

| Field | Value |
|-------|-------|
| **Spec ID** | SPEC-010 |
| **Title** | Traceability Standard |
| **Status** | Active |
| **Version** | 1.0.0 |
| **Created** | 2026-02-05 |
| **Author** | Vince Liem |
| **Co-Author** | Claude Opus 4.5 |

---

## 1. Purpose

This specification defines how requirements are traced through the system:

```
Requirement → Implementation → Test → Verification
```

Traceability answers:
- **Forward**: "Where is this requirement implemented?"
- **Backward**: "Why does this code exist?"
- **Coverage**: "Is every requirement tested?"

## 2. Scope

This specification covers:
- Requirement identification
- Implementation linking
- Test linking
- Traceability matrices

## 3. Definitions

| Term | Definition |
|------|------------|
| **Requirement** | A statement of what the system MUST, SHOULD, or MAY do |
| **Trace Link** | A documented connection between a requirement and its implementation or test |
| **Forward Traceability** | Linking from requirement → implementation |
| **Backward Traceability** | Linking from implementation → requirement |
| **Traceability Matrix** | A table showing all trace links for a specification |

## 4. Requirement Identification

### 4.1 Requirement ID Format

```
REQ-{SPEC}-{SEQ}
```

| Component | Description | Example |
|-----------|-------------|---------|
| `REQ` | Fixed prefix | `REQ` |
| `{SPEC}` | Spec number (2-3 digits) | `010`, `060` |
| `{SEQ}` | Sequence within spec (2 digits) | `01`, `15` |

**Examples:**
- `REQ-010-01` — First requirement in spec 10
- `REQ-060-15` — Fifteenth requirement in spec 60

### 4.2 Requirement Statement

Every requirement MUST:

| Rule | Description |
|------|-------------|
| Be atomic | One testable statement per requirement |
| Use RFC 2119 | MUST, SHOULD, MAY keywords |
| Be verifiable | Can be tested or demonstrated |
| Be unique | No duplicate requirements |

## 5. Implementation Linking

### 5.1 Traces To Format

The `Traces To` field links to implementation:

```
{path}:{symbol}
```

| Component | Description | Example |
|-----------|-------------|---------|
| `{path}` | File path from project root | `src-tauri/src/loop_engine.rs` |
| `{symbol}` | Function, struct, or trait | `LoopEngine::start` |

**Examples:**
```markdown
| Traces To | `src-tauri/src/loop_engine.rs:LoopEngine::start` |
| Traces To | `src/stores/useLoopStore.ts:useLoopStore` |
```

### 5.2 Code Annotations

Implementation code SHOULD reference requirement IDs:

**Rust:**
```rust
/// Starts the loop engine.
///
/// # Requirements
/// - REQ-060-05: Loop engine MUST validate project path
/// - REQ-060-06: Loop engine MUST emit state changed events
pub fn start(&self, app: AppHandle, project_path: PathBuf) -> Result<(), RalphError>
```

**TypeScript:**
```typescript
/**
 * Zustand store for loop state.
 *
 * @requirements
 * - REQ-F10-01: Store MUST track current iteration
 * - REQ-F10-02: Store MUST buffer output lines
 */
export const useLoopStore = create<LoopStore>((set) => ({ ... }));
```

## 6. Test Linking

### 6.1 Tested By Format

The `Tested By` field links to verification:

| Type | Format | Example |
|------|--------|---------|
| Rust test | `{module}::tests::{test_name}` | `loop_engine::tests::test_start_validates_path` |
| Vitest | `{file}::{test_name}` | `useLoopStore.test.ts::tracks iteration` |
| WebdriverIO | `{file}::{test_name}` | `e2e-tauri/terminal.spec.js::opens terminal UI` |
| Manual | `"Manual: {description}"` | `"Manual: UI inspection"` |

### 6.2 Test Annotations

Tests SHOULD reference requirement IDs:

**Rust:**
```rust
/// Tests REQ-060-05: Loop engine MUST validate project path
#[test]
fn test_start_validates_path() {
    // ...
}
```

**TypeScript:**
```typescript
// Verifies: REQ-F10-01, REQ-F10-02
test('tracks iteration and buffers output', () => {
    // ...
});
```

## 7. Traceability Matrix

### 7.1 Format

Every specification SHOULD end with a traceability matrix:

```markdown
## Traceability Matrix

| Req ID | Requirement Summary | Implementation | Test | Status |
|--------|---------------------|----------------|------|--------|
| REQ-060-01 | E2E tests use WebdriverIO + tauri-driver | `e2e-tauri/**/*.spec.js` | CI run | ✓ |
| REQ-060-02 | Visual tests use WebdriverIO | `e2e-tauri/visual.spec.js` | CI run | ✓ |
```

### 7.2 Status Values

| Status | Symbol | Meaning |
|--------|--------|---------|
| Complete | ✓ | Implemented and tested |
| Partial | ◐ | Implemented, test incomplete |
| Missing | ✗ | Not implemented |
| N/A | — | Not applicable |

## 8. Bidirectional Traceability

### 8.1 Forward Trace (Spec → Code)

Starting from a requirement, find its implementation:

```
REQ-060-05
    ↓ Traces To
loop_engine.rs:LoopEngine::start
    ↓ Tested By
test_start_validates_path
```

### 8.2 Backward Trace (Code → Spec)

Starting from code, find its requirement via doc comments.

## Change Log

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-02-05 | Initial specification adapted from iuppiter-dar |
