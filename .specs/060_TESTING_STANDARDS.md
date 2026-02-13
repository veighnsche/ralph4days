# Testing Standards

| Field | Value |
|-------|-------|
| **Spec ID** | SPEC-060 |
| **Title** | Testing Standards |
| **Status** | Active |
| **Version** | 1.0.0 |
| **Created** | 2026-02-05 |
| **Author** | Vince Liem |
| **Co-Author** | Claude Opus 4.5 |

---

## 1. Purpose

This specification defines testing requirements, tools, and patterns for the Ralph Loop desktop application. It covers the full testing pyramid from unit tests through E2E, visual, and chaos testing.

## 2. Scope

This specification covers:
- Test categories and their purposes
- Tool selection rationale
- Test requirements per layer (Rust backend, React frontend)
- E2E, visual, and monkey testing standards
- CI integration requirements

## 3. Definitions

| Term | Definition |
|------|------------|
| **Unit Test** | Tests a single function/module in isolation |
| **Integration Test** | Tests multiple components working together |
| **E2E Test** | Tests complete user flows through the full application |
| **Visual Test** | Tests UI appearance against baseline screenshots |
| **Monkey Test** | Chaos testing with random user interactions |
| **Mutation Testing** | Verifies test quality by introducing code mutations |
| **Golden Test** | Tests against pre-validated reference data |
| **Flaky Test** | Test that sometimes passes, sometimes fails |

## 4. Tool Selection

### 4.1 Recommended Tools Summary

| Category | Primary Tool | Rationale |
|----------|-------------|-----------|
| **Rust Unit Tests** | `cargo test` | Built-in, fast, standard |
| **Frontend Unit Tests** | Vitest + React Testing Library | Fast, ESM-native, Tauri mock support |
| **Integration Tests** | Vitest + `@tauri-apps/api/mocks` | Official Tauri mocking support |
| **E2E Tests** | WebdriverIO + `tauri-driver` | Native Tauri window automation |
| **Visual Regression** | WebdriverIO | Native-runner compatible |
| **Monkey/Chaos Testing** | Gremlins.js + WebdriverIO | Free, proven, native-window compatible |
| **Mutation Testing** | `cargo-mutants` (Rust) | Verifies test quality |

### 4.2 Tool Selection Rationale

#### E2E Testing: WebdriverIO over UI-only frameworks

| Consideration | WebdriverIO + tauri-driver | UI-only automation |
|--------------|------------|---------------------------|
| Stack fidelity | Native Tauri window | DOM-only web emulation |
| Runtime completeness | Shell + web surface | Web surface only |
| Setup complexity | Moderate (driver + build) | Low |
| CI scope | Fails loudly if driver/binary missing | Requires explicit runtime bootstrap |

**Decision:** Use **WebdriverIO + tauri-driver** for e2e. UI-only tooling is not acceptable for native shell runtime verification.

**Limitation:** Requires a debug Tauri binary and a discoverable `tauri-driver` in PATH.

#### Visual Testing: WebdriverIO over Chromatic/Percy

| Consideration | WebdriverIO | Chromatic/Percy |
|--------------|-------------------|-----------------|
| Cost | Free | $$$$ (SaaS) |
| Setup | In-repo runner config | Requires account |
| Integration | Native | Separate service |
| Offline | ✓ | ✗ |

**Decision:** Use **WebdriverIO-based screenshots** for visual checks. Keep baseline tooling and compare workflow in one native runtime surface.

#### Monkey Testing: Gremlins.js

**Decision:** Use **Gremlins.js** for chaos testing. It's specifically designed for frontend monkey testing, simulates random user actions, and integrates with WebdriverIO driver sessions.

## 5. Test Categories

### 5.1 Rust Backend Tests

#### Unit Tests

Test individual functions in isolation within the same file.

```rust
// src-tauri/src/prompt_builder.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_completion_detects_marker() {
        let output = "Task done <promise>COMPLETE</promise>";
        assert!(PromptBuilder::check_completion(output));
    }

    #[test]
    fn test_check_completion_no_marker() {
        let output = "Still working...";
        assert!(!PromptBuilder::check_completion(output));
    }
}
```

#### Integration Tests

Test multiple modules working together.

```rust
// src-tauri/tests/integration.rs
use ralph4days_lib::loop_engine::LoopEngine;
use tempfile::TempDir;

#[test]
fn test_start_validates_project_path() {
    let engine = LoopEngine::new();
    let result = engine.start(
        mock_app_handle(),
        PathBuf::from("/nonexistent"),
        10,
    );
    assert!(matches!(result, Err(RalphError::ProjectNotFound(_))));
}
```

### 5.2 Frontend Tests

#### Unit Tests (Vitest)

Test React components and hooks in isolation.

```typescript
// src/stores/useLoopStore.test.ts
import { describe, it, expect, beforeEach } from 'vitest';
import { useLoopStore } from './useLoopStore';

describe('useLoopStore', () => {
  beforeEach(() => {
    useLoopStore.getState().reset();
  });

  it('adds output lines with incrementing IDs', () => {
    const { addOutput } = useLoopStore.getState();
    addOutput('Line 1');
    addOutput('Line 2');

    const { output } = useLoopStore.getState();
    expect(output).toHaveLength(2);
    expect(output[0].id).toBeLessThan(output[1].id);
  });
});
```

#### Integration Tests (Vitest + Tauri Mocks)

Test frontend-backend communication.

```typescript
// src/__tests__/integration.test.ts
import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { mockIPC, clearMocks } from '@tauri-apps/api/mocks';
import { invoke } from '@tauri-apps/api/core';

describe('IPC Integration', () => {
  beforeEach(() => {
    mockIPC((cmd, args) => {
      switch (cmd) {
        case 'get_loop_state':
          return {
            state: 'idle',
            current_iteration: 0,
            max_iterations: 0,
          };
        case 'start_loop':
          return null;
        default:
          throw new Error(`Unknown command: ${cmd}`);
      }
    });
  });

  afterEach(() => {
    clearMocks();
  });

  it('fetches loop state from backend', async () => {
    const state = await invoke('get_loop_state');
    expect(state.state).toBe('idle');
  });
});
```

### 5.3 E2E Tests (WebdriverIO)

Test complete user flows through the UI.

```javascript
// e2e-tauri/terminal.spec.js
describe('Terminal flow', () => {
  it('opens terminal UI in Tauri runtime', async () => {
    const runtime = await browser.execute(() => {
      return Boolean(window.__TAURI__ || window.__TAURI_IPC__)
    })
    expect(runtime).toBe(true)

    const terminalHost = await $('[data-testid="workspace-terminal-host"]')
    await terminalHost.waitForDisplayed({ timeout: 30000 })
    expect(await terminalHost.isDisplayed()).toBe(true)
  })
})
```

### 5.4 Visual Tests (WebdriverIO)

Test UI appearance against baseline screenshots.

```javascript
// e2e-tauri/visual.spec.js
describe('Visual checks', () => {
  it('captures a dashboard baseline', async () => {
    const root = await $('[data-testid="workspace-root"]')
    await expect(root).toHaveElementScreenshot('dashboard.png')
  })
})
```

**Visual test guidelines:**
- Store baselines in dedicated WebdriverIO snapshot folders.
- Use your configured WebdriverIO screenshot diff flow to update baselines.
- Use `maxDiffPixelRatio: 0.01` for tolerance
- Disable animations before screenshots

### 5.5 Monkey Tests (Gremlins.js)

Chaos testing with random user interactions.

```javascript
// e2e-tauri/monkey.spec.js
describe('Chaos Testing', () => {
  it('app survives randomized interactions', async () => {
    const app = await $('[data-testid="app-root"]')
    expect(await app.isExisting()).toBe(true)

    // Inject Gremlins.js
    await browser.execute(() => {
      const script = document.createElement('script')
      script.src = 'https://unpkg.com/gremlins.js@2.2.0/dist/gremlins.min.js'
      document.body.appendChild(script)
      return new Promise(resolve => {
        script.onload = resolve
      })
    });

    // Configure and unleash gremlins
    await browser.execute(() => {
      return new Promise(resolve => {
        const gremlins = window.gremlins
        if (!gremlins || !gremlins.createHorde) {
          resolve(null)
          return
        }
        gremlins.createHorde({
          species: [
            gremlins.species.clicker(),
            gremlins.species.formFiller(),
            gremlins.species.scroller(),
            gremlins.species.typer(),
          ],
          mogwais: [
            gremlins.mogwais.alert(),
            gremlins.mogwais.gizmo(),
          ],
          strategies: [
            gremlins.strategies.distribution({
              distribution: [0.3, 0.3, 0.2, 0.2],
              delay: 50,
            }),
          ],
        }).unleash({ nb: 500 }).then(resolve)
      })
    })

    expect(await app.isDisplayed()).toBe(true)
  });
});
```

**Monkey test guidelines:**
- Run in isolation (not with other tests)
- Set reasonable iteration count (500-1000)
- Check for console errors after completion
- Verify app is still responsive

## 6. Test Requirements

### REQ-060-01: Rust Unit Test Coverage

Every public function in the Rust backend MUST have at least one unit test.

| Traces To | `src-tauri/src/**/*.rs` |
| Tested By | `cargo test` |
| Rationale | Verify core logic |

### REQ-060-02: Frontend Unit Test Coverage

Every React component and hook MUST have at least one unit test.

| Traces To | `src/**/*.tsx`, `src/**/*.ts` |
| Tested By | `bun test` |
| Rationale | Verify UI logic |

### REQ-060-03: E2E Tests for Critical Paths

The following user flows MUST have E2E tests:

| Flow | Test File |
|------|-----------|
| Start loop | `e2e-tauri/terminal.spec.js` |
| Pause/Resume | `e2e-tauri/terminal.spec.js` |
| Stop loop | `e2e-tauri/terminal.spec.js` |
| Output streaming | `e2e-tauri/terminal.spec.js` |

| Traces To | `e2e-tauri/**/*.spec.js` |
| Tested By | `bunx wdio run wdio.conf.js` |
| Rationale | Verify user-facing functionality |

### REQ-060-04: Visual Regression for UI States

Visual tests MUST cover all loop states:

| State | Screenshot |
|-------|------------|
| Idle | `idle-state.png` |
| Running | `running-state.png` |
| Paused | `paused-state.png` |
| Rate Limited | `rate-limited-state.png` |
| Complete | `complete-state.png` |
| Aborted | `aborted-state.png` |

| Traces To | `e2e-tauri/visual.spec.js` |
| Tested By | `bunx wdio run wdio.conf.js --spec e2e-tauri/visual.spec.js` |
| Rationale | Prevent UI regressions |

### REQ-060-05: Monkey Test Before Release

Monkey tests MUST pass with 500+ interactions before any release.

| Traces To | `e2e-tauri/monkey.spec.js` |
| Tested By | `bunx wdio run wdio.conf.js --spec e2e-tauri/monkey.spec.js` |
| Rationale | Verify crash resistance |

### REQ-060-06: No Flaky Tests

Tests MUST pass 100% of the time or be marked with skip reason.

| Traces To | All test files |
| Tested By | CI repeated runs |
| Rationale | Flaky tests erode confidence |

### REQ-060-07: Test Independence

Tests MUST NOT depend on other tests or execution order.

| Traces To | All test files |
| Tested By | Parallel test execution |
| Rationale | Enable parallel CI |

## 7. CI Integration

### 7.1 GitHub Actions Workflow

```yaml
# .github/workflows/test.yml
name: Test
on: [push, pull_request]

jobs:
  rust-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --manifest-path src-tauri/Cargo.toml

  frontend-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: bun/action-setup@v2
      - run: bun install
      - run: bun test:run

  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: bun/action-setup@v2
      - run: bun install
      - run: bun build
      - run: bunx wdio run wdio.conf.js

  visual-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: bun/action-setup@v2
      - run: bun install
      - run: bun build
      - run: bunx wdio run wdio.conf.js --spec e2e-tauri/visual.spec.js
```

### 7.2 Required CI Checks

| Check | Command | Blocking |
|-------|---------|----------|
| Rust tests | `cargo test` | Yes |
| Frontend tests | `bun test:run` | Yes |
| E2E tests | `bunx wdio run wdio.conf.js` | Yes |
| Visual tests | `bunx wdio run wdio.conf.js --spec e2e-tauri/visual.spec.js` | Yes |
| Monkey tests | `bunx wdio run wdio.conf.js --spec e2e-tauri/monkey.spec.js` | No (warning) |

## 8. Project Setup

### 8.1 Install Dependencies

```bash
# Frontend testing
bun add -D vitest @testing-library/react @testing-library/jest-dom jsdom

// Install Tauri e2e dependencies
cargo install tauri-driver --locked
```

### 8.2 Vitest Configuration

```typescript
// vitest.config.ts
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.test.{ts,tsx}'],
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
});
```

### 8.3 WebdriverIO Configuration

```javascript
// wdio.conf.js
// See repository root: `wdio.conf.js`.
```

### 8.4 Package.json Scripts

```json
{
  "scripts": {
    "test": "vitest",
    "test:run": "vitest run",
    "test:e2e": "bunx wdio run wdio.conf.js",
    "test:visual": "bunx wdio run wdio.conf.js --spec e2e-tauri/visual.spec.js",
    "test:monkey": "bunx wdio run wdio.conf.js --spec e2e-tauri/monkey.spec.js",
    "test:all": "bun test:run && bun test:e2e"
  }
}
```

## 9. Directory Structure

```
ralph4days/
├── src/
│   ├── __tests__/           # Frontend integration tests
│   │   └── integration.test.ts
│   ├── stores/
│   │   ├── useLoopStore.ts
│   │   └── useLoopStore.test.ts
│   ├── components/
│   │   ├── StatusBadge.tsx
│   │   └── StatusBadge.test.tsx
│   └── test/
│       └── setup.ts         # Vitest setup
├── src-tauri/
│   ├── src/
│   │   ├── loop_engine.rs   # Unit tests inline
│   │   └── prompt_builder.rs
│   └── tests/
│       └── integration.rs   # Rust integration tests
├── e2e-tauri/
│   ├── terminal.spec.js     # E2E flow
│   ├── monkey.spec.js      # Chaos tests
│   └── visual.spec.js      # Visual tests
├── vitest.config.ts
└── wdio.conf.js
```

## 10. Traceability Matrix

| Req ID | Requirement Summary | Implementation | Test | Status |
|--------|---------------------|----------------|------|--------|
| REQ-060-01 | Rust unit test coverage | `src-tauri/src/**/*.rs` | `cargo test` | ◐ |
| REQ-060-02 | Frontend unit test coverage | `src/**/*.test.ts` | `bun test` | ✗ |
| REQ-060-03 | E2E tests for critical paths | `e2e-tauri/**/*.spec.js` | `bunx wdio run wdio.conf.js` | ✗ |
| REQ-060-04 | Visual regression for states | `e2e-tauri/visual.spec.js` | `bunx wdio run wdio.conf.js --spec e2e-tauri/visual.spec.js` | ✗ |
| REQ-060-05 | Monkey test before release | `e2e-tauri/monkey.spec.js` | `bunx wdio run wdio.conf.js --spec e2e-tauri/monkey.spec.js` | ✗ |
| REQ-060-06 | No flaky tests | All test files | CI runs | — |
| REQ-060-07 | Test independence | All test files | Parallel runs | — |

## Change Log

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-02-05 | Initial specification with tool research |
