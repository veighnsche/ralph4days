# TESTING STRATEGY - AUTOMATION vs TAURI WEBDRIVER

## Current State (NEEDS REFACTORING)

Ralph4days has **three test types** with misaligned tooling:

| Test Type | Current Tool | Status | Purpose |
|-----------|-------------|---------|---------|
| **Rust Backend** | `cargo test` | ✅ Works | Unit tests for Rust logic |
| **Frontend Unit** | Vitest | ✅ Works | React components/hooks |
| **E2E Tests** | Automation runner → localhost:1420 | ⚠️ WRONG | Tests web UI in browser |

### The Problem

**Automation runner is testing the WRONG thing:**
```
Current Setup:
  Automation runner → Chromium browser → http://localhost:1420 → Web UI only

What Gets Tested:
  ✅ React component rendering
  ✅ Frontend state management
  ❌ Tauri IPC calls (mocked/missing)
  ❌ Rust backend integration
  ❌ PTY sessions
  ❌ File system operations
  ❌ Native desktop features

Result: False confidence. Tests pass but real app could be broken.
```

**Port 1420 issue:** Running full dev server for tests = production-like server just for E2E = not ideal.

## Future Architecture (TODO)

### 1. Automation runner → Storybook Only

**Purpose:** Visual regression testing of isolated components

```bash
# Storybook serves component stories
npm run storybook

# Automation runner tests Storybook pages
npx automation-runner test storybook/
```

**What it tests:**
- ✅ Component visual appearance
- ✅ Responsive design
- ✅ Accessibility
- ✅ UI states (loading, error, success)

**What it DOESN'T test:**
- ❌ Integration with Tauri backend
- ❌ Real IPC communication

### 2. Tauri WebDriver → Real E2E Tests

**Purpose:** Test the actual compiled Tauri desktop application

```bash
# Builds and launches REAL Tauri app
tauri-driver &
npx wdio run wdio.conf.ts
```

**What it tests:**
- ✅ **Actual Tauri .exe/.app** (not browser)
- ✅ **Real IPC calls** (React → Tauri → Rust)
- ✅ **PTY sessions** (Claude CLI spawning)
- ✅ **File system** (.ralph/db/ operations)
- ✅ **Full user workflows** (end-to-end)

**Example test:**
```typescript
describe('Terminal PTY Session', () => {
  it('launches Claude CLI when opening terminal tab', async () => {
    // This tests the REAL app with REAL backend
    await $('#new-terminal-btn').click();

    // Wait for actual PTY session to spawn
    await $('.xterm').waitForDisplayed({ timeout: 5000 });

    // Can verify Claude CLI process is running
    const output = await $('.xterm').getText();
    expect(output).toContain('Claude Code');
  });
});
```

## Implementation Plan (TODO)

### Phase 1: Fix Current Setup ✅ DONE
- [x] Install automation-runner browsers (for Storybook later)
- [x] Keep Vitest for frontend unit tests
- [x] Document this strategy

### Phase 2: Configure Automation runner for Storybook Only 🔜

**Files to create:**
```
.storybook/
  └── test-runner.ts          # Automation runner config for Storybook
automation-runner-storybook.config.ts # Separate config from E2E
package.json                   # New scripts
```

**New scripts:**
```json
{
  "test:storybook": "test-storybook",
  "storybook:ci": "concurrently -k -s first -n SB,TEST \"npm run storybook --ci\" \"npm run test:storybook\""
}
```

**Dependencies:**
```bash
bun add -d @storybook/test-runner
```

**Benefits:**
- Tests components in isolation
- Visual regression testing
- No backend coupling

### Phase 3: Add Tauri WebDriver for E2E 🔜

**Files to create:**
```
e2e-tauri/
  ├── pty-session.spec.ts       # Test PTY/terminal features
  ├── loop-engine.spec.ts       # Test loop execution
  ├── task-creation.spec.ts     # Test IPC → Rust → DB
  └── project-picker.spec.ts    # Test project validation
wdio.conf.ts                    # WebdriverIO config
```

**Dependencies:**
```bash
cargo install tauri-driver
bun add -d @wdio/cli @wdio/local-runner @wdio/mocha-framework @wdio/spec-reporter
```

**Config template:**
```typescript
// wdio.conf.ts
export const config = {
  specs: ['./e2e-tauri/**/*.spec.ts'],
  capabilities: [{
    browserName: 'tauri',
    'tauri:options': {
      application: './src-tauri/target/release/ralph4days',
    }
  }],
  services: ['tauri-driver'],
  framework: 'mocha',
  reporters: ['spec'],
}
```

**New scripts:**
```json
{
  "test:e2e-tauri": "wdio run wdio.conf.ts",
  "test:e2e:watch": "wdio run wdio.conf.ts --watch"
}
```

## Final Test Structure

```
just test              # Run all tests
├─ just test-rust      # Cargo test (backend unit)
├─ just test-frontend  # Vitest (React unit)
├─ just test-storybook # Automation runner → Storybook (visual)
└─ just test-e2e       # Tauri WebDriver (full integration)
```

## Decision Matrix

| Need to test... | Use... |
|-----------------|--------|
| Rust function | `cargo test` |
| React hook | Vitest |
| Component UI | Automation runner → Storybook |
| IPC call | Tauri WebDriver |
| PTY session | Tauri WebDriver |
| File operations | Tauri WebDriver |
| Full workflow | Tauri WebDriver |

## References

- **Tauri Testing Guide:** https://tauri.app/v1/guides/testing/webdriver/introduction
- **Storybook Test Runner:** https://storybook.js.org/docs/writing-tests/test-runner
- **WebdriverIO:** https://webdriver.io/docs/what-is-webdriverio

## Next Steps

1. **Short term:** Use current Vitest + Rust tests (no Automation runner)
2. **Medium term:** Add Storybook visual tests with Automation runner
3. **Long term:** Implement Tauri WebDriver for real E2E coverage

## Related Files

- `justfile` - Test commands
- `vitest.config.ts` - Vitest configuration
- `automation-runner.config.ts` - Currently misconfigured, will split into Storybook-only
- `.storybook/` - Storybook configuration (for visual tests)

---

**Status:** 📝 Planning phase - implementation pending
**Priority:** Medium (can manually test for now)
**Complexity:** High (Tauri WebDriver setup is non-trivial)
