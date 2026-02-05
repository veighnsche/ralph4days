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
| **E2E Tests** | Playwright | Cross-platform, reliable, built-in visual comparisons |
| **Visual Regression** | Playwright Visual Comparisons | Integrated, no additional tooling |
| **Monkey/Chaos Testing** | Gremlins.js + Playwright | Free, proven, web-native |
| **Mutation Testing** | `cargo-mutants` (Rust) | Verifies test quality |

### 4.2 Tool Selection Rationale

#### E2E Testing: Playwright over WebdriverIO

| Consideration | Playwright | WebdriverIO + tauri-driver |
|--------------|------------|---------------------------|
| macOS support | ✓ Full | ✗ Not supported |
| Setup complexity | Low | High (requires tauri-driver) |
| Visual testing | Built-in | Requires additional tools |
| Cross-platform CI | ✓ | Linux/Windows only |

**Decision:** Use **Playwright** for E2E testing. WebdriverIO with tauri-driver lacks macOS support due to missing WKWebView driver, making it unsuitable for cross-platform development.

**Limitation:** Playwright tests the webview layer, not the full Rust binary. Use `@tauri-apps/api/mocks` to mock IPC calls for frontend testing. True full-stack E2E requires WebdriverIO on Linux/Windows CI runners.

#### Visual Testing: Playwright over Chromatic/Percy

| Consideration | Playwright Visual | Chromatic/Percy |
|--------------|-------------------|-----------------|
| Cost | Free | $$$$ (SaaS) |
| Setup | Built-in | Requires account |
| Integration | Native | Separate service |
| Offline | ✓ | ✗ |

**Decision:** Use **Playwright Visual Comparisons** for visual regression. It's free, built-in, and sufficient for desktop app testing. Consider Lost Pixel (open-source) if cloud-based comparison is needed later.

#### Monkey Testing: Gremlins.js

**Decision:** Use **Gremlins.js** for chaos testing. It's specifically designed for frontend monkey testing, simulates random user actions, and integrates easily with Playwright.

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

### 5.3 E2E Tests (Playwright)

Test complete user flows through the UI.

```typescript
// e2e/controls.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Loop Controls', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('start button disabled without project path', async ({ page }) => {
    const startBtn = page.getByRole('button', { name: 'Start' });
    await expect(startBtn).toBeDisabled();
  });

  test('start button enabled with project path', async ({ page }) => {
    await page.fill('input[placeholder*="path"]', '/tmp/test-project');
    const startBtn = page.getByRole('button', { name: 'Start' });
    await expect(startBtn).toBeEnabled();
  });

  test('status badge shows idle initially', async ({ page }) => {
    await expect(page.locator('text=Idle')).toBeVisible();
  });
});
```

### 5.4 Visual Tests (Playwright)

Test UI appearance against baseline screenshots.

```typescript
// e2e/visual/states.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Visual States', () => {
  test('idle state appearance', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveScreenshot('idle-state.png');
  });

  test('controls panel appearance', async ({ page }) => {
    await page.goto('/');
    const controls = page.locator('[data-testid="controls-panel"]');
    await expect(controls).toHaveScreenshot('controls-panel.png');
  });
});
```

**Visual test guidelines:**
- Store baselines in `e2e/visual/*.spec.ts-snapshots/`
- Update baselines with `pnpm exec playwright test --update-snapshots`
- Use `maxDiffPixelRatio: 0.01` for tolerance
- Disable animations before screenshots

### 5.5 Monkey Tests (Gremlins.js)

Chaos testing with random user interactions.

```typescript
// e2e/monkey.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Chaos Testing', () => {
  test('app survives 500 random interactions', async ({ page }) => {
    await page.goto('/');

    // Inject Gremlins.js
    await page.addScriptTag({
      url: 'https://unpkg.com/gremlins.js@2.2.0/dist/gremlins.min.js',
    });

    // Configure and unleash gremlins
    await page.evaluate(() => {
      return new Promise<void>((resolve) => {
        (window as any).gremlins.createHorde({
          species: [
            (window as any).gremlins.species.clicker(),
            (window as any).gremlins.species.formFiller(),
            (window as any).gremlins.species.scroller(),
            (window as any).gremlins.species.typer(),
          ],
          mogwais: [
            (window as any).gremlins.mogwais.alert(),
            (window as any).gremlins.mogwais.gizmo(),
          ],
          strategies: [
            (window as any).gremlins.strategies.distribution({
              distribution: [0.3, 0.3, 0.2, 0.2],
              delay: 50,
            }),
          ],
        }).unleash({ nb: 500 }).then(resolve);
      });
    });

    // Verify app didn't crash
    await expect(page.locator('[data-testid="app-root"]')).toBeVisible();

    // Check for console errors
    const errors: string[] = [];
    page.on('pageerror', (err) => errors.push(err.message));
    expect(errors).toHaveLength(0);
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
| Tested By | `pnpm test` |
| Rationale | Verify UI logic |

### REQ-060-03: E2E Tests for Critical Paths

The following user flows MUST have E2E tests:

| Flow | Test File |
|------|-----------|
| Start loop | `e2e/controls.spec.ts` |
| Pause/Resume | `e2e/controls.spec.ts` |
| Stop loop | `e2e/controls.spec.ts` |
| Output streaming | `e2e/output.spec.ts` |

| Traces To | `e2e/*.spec.ts` |
| Tested By | `pnpm exec playwright test` |
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

| Traces To | `e2e/visual/*.spec.ts` |
| Tested By | `pnpm exec playwright test e2e/visual/` |
| Rationale | Prevent UI regressions |

### REQ-060-05: Monkey Test Before Release

Monkey tests MUST pass with 500+ interactions before any release.

| Traces To | `e2e/monkey.spec.ts` |
| Tested By | `pnpm exec playwright test e2e/monkey.spec.ts` |
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
      - uses: pnpm/action-setup@v2
      - run: pnpm install
      - run: pnpm test:run

  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v2
      - run: pnpm install
      - run: pnpm exec playwright install --with-deps
      - run: pnpm build
      - run: pnpm exec playwright test

  visual-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v2
      - run: pnpm install
      - run: pnpm exec playwright install --with-deps
      - run: pnpm build
      - run: pnpm exec playwright test e2e/visual/
```

### 7.2 Required CI Checks

| Check | Command | Blocking |
|-------|---------|----------|
| Rust tests | `cargo test` | Yes |
| Frontend tests | `pnpm test:run` | Yes |
| E2E tests | `playwright test` | Yes |
| Visual tests | `playwright test e2e/visual/` | Yes |
| Monkey tests | `playwright test e2e/monkey.spec.ts` | No (warning) |

## 8. Project Setup

### 8.1 Install Dependencies

```bash
# Frontend testing
pnpm add -D vitest @testing-library/react @testing-library/jest-dom jsdom
pnpm add -D @playwright/test

# Install Playwright browsers
pnpm exec playwright install
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

### 8.3 Playwright Configuration

```typescript
// playwright.config.ts
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:1420',
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: {
    command: 'pnpm dev',
    url: 'http://localhost:1420',
    reuseExistingServer: !process.env.CI,
  },
});
```

### 8.4 Package.json Scripts

```json
{
  "scripts": {
    "test": "vitest",
    "test:run": "vitest run",
    "test:e2e": "playwright test",
    "test:visual": "playwright test e2e/visual/",
    "test:monkey": "playwright test e2e/monkey.spec.ts",
    "test:all": "pnpm test:run && pnpm test:e2e"
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
├── e2e/
│   ├── controls.spec.ts     # E2E tests
│   ├── output.spec.ts
│   ├── monkey.spec.ts       # Chaos tests
│   └── visual/
│       ├── states.spec.ts   # Visual tests
│       └── states.spec.ts-snapshots/
├── vitest.config.ts
└── playwright.config.ts
```

## 10. Traceability Matrix

| Req ID | Requirement Summary | Implementation | Test | Status |
|--------|---------------------|----------------|------|--------|
| REQ-060-01 | Rust unit test coverage | `src-tauri/src/**/*.rs` | `cargo test` | ◐ |
| REQ-060-02 | Frontend unit test coverage | `src/**/*.test.ts` | `pnpm test` | ✗ |
| REQ-060-03 | E2E tests for critical paths | `e2e/*.spec.ts` | `playwright test` | ✗ |
| REQ-060-04 | Visual regression for states | `e2e/visual/*.spec.ts` | `playwright test` | ✗ |
| REQ-060-05 | Monkey test before release | `e2e/monkey.spec.ts` | `playwright test` | ✗ |
| REQ-060-06 | No flaky tests | All test files | CI runs | — |
| REQ-060-07 | Test independence | All test files | Parallel runs | — |

## Change Log

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-02-05 | Initial specification with tool research |
