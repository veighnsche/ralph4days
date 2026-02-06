# TODO: Remove Deprecated Iteration Logic

**Summary**: The loop iteration counting system is deprecated. Loops should either be **infinite** (run until manually stopped) or **single-run** (execute once). The concept of "run N iterations" is being removed.

## Required Changes

### Backend (Rust)

#### `src-tauri/src/types.rs`
- [ ] **LoopConfig**: Remove `max_iterations: u32`, add `loop_enabled: bool`
- [ ] **LoopStatus**: Remove `max_iterations: u32` field
- [ ] **LoopStatus**: Keep `current_iteration: u32` for display purposes only (not for loop control)
- [ ] Update Default implementations to use `loop_enabled: false` instead of `max_iterations: 100`

#### `src-tauri/src/loop_engine.rs`
- [ ] **start()**: Change parameter from `max_iterations: u32` to `loop_enabled: bool`
- [ ] Remove the iteration limit check (lines ~203-220) that compares `current_iteration >= config.max_iterations`
- [ ] For infinite loops (`loop_enabled: true`), only stop when:
  - Manually stopped by user
  - All tasks completed
  - Stagnation detected
  - Fatal error occurs
- [ ] For single-run (`loop_enabled: false`), stop after first iteration

#### `src-tauri/src/commands.rs`
- [ ] **start_loop**: Change signature to accept `loop_enabled: bool` instead of `max_iterations: u32`
- [ ] Update all calls to `loop_engine.start()` to pass boolean instead of number

### Frontend (TypeScript/React)

#### `src/stores/useLoopStore.ts`
- [ ] **LoopStore interface**: Replace `maxIterations: number` with `loopEnabled: boolean`
- [ ] **LoopStatus interface**: Remove `max_iterations: number` (keep `current_iteration` for display)
- [ ] Replace `setMaxIterations: (max: number) => void` with `setLoopEnabled: (enabled: boolean) => void`
- [ ] Update initial state: `loopEnabled: false` instead of `maxIterations: 1`
- [ ] Update implementation to use boolean instead of number

#### `src/components/LoopToggle.tsx`
- [ ] **Interface**: Replace `maxIterations: number` with `isLoopEnabled: boolean`
- [ ] **Interface**: Replace `setMaxIterations` with `setLoopEnabled`
- [ ] Update toggle logic to use boolean directly
- [ ] Update tooltip to say "Runs infinitely" instead of "Runs N iterations"
- [ ] Remove the hack that uses `100` to represent "infinite"

#### `src/components/BottomBar.tsx`
- [ ] Replace `maxIterations`/`setMaxIterations` with `loopEnabled`/`setLoopEnabled`
- [ ] Update `start_loop` invoke to pass `loopEnabled` boolean
- [ ] Remove the log message that shows "Max iterations: N"

#### `src/components/LoopCountBadge.tsx`
- [ ] **DELETE THIS FILE** - Completely replaced by `LoopToggle.tsx`

#### `src/components/LoopCountBadge.stories.tsx`
- [ ] **DELETE THIS FILE** - Related to deleted component

#### `src/stores/useLoopStore.test.ts`
- [ ] Update tests to use `loopEnabled` instead of `maxIterations`
- [ ] Remove tests that verify iteration counting logic

### Testing

After implementing changes:
- [ ] Test infinite loop mode (should run until manually stopped)
- [ ] Test single-run mode (should stop after one iteration)
- [ ] Verify UI toggle works correctly
- [ ] Verify backend properly handles both modes
- [ ] Update any integration tests that reference iterations

## Architecture Notes

**Old System** (deprecated):
```
maxIterations: 1 | 2 | 3 | ... | 1000
- Run for exactly N iterations
- Count down remaining iterations
- Stop when current_iteration >= max_iterations
```

**New System** (target):
```
loopEnabled: true | false
- true  → Run infinitely until manually stopped
- false → Run once and stop
- No iteration counting for loop control
- current_iteration kept for display only
```

## Migration Strategy

1. Add new boolean fields alongside old numeric fields
2. Update backend to use boolean for loop control, ignore numeric values
3. Update frontend to use boolean, keep numeric for backwards compat
4. Once stable, remove all numeric iteration logic
5. Delete deprecated components and tests
