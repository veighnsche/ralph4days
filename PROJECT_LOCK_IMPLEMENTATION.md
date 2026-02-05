# Project Lock Implementation - Complete

## Summary

Successfully implemented project locking functionality for Ralph Loop. The application now enforces ONE project per session with two startup modes:

1. **CLI Argument Mode**: `ralph --project /path/to/project` locks immediately
2. **Interactive Mode**: ProjectPicker modal appears if no CLI arg provided

## Files Modified

### Backend (Rust)

1. **src-tauri/Cargo.toml**
   - Added `tauri-plugin-cli = "2"` dependency

2. **src-tauri/tauri.conf.json**
   - Added CLI plugin configuration with `--project` argument

3. **src-tauri/src/lib.rs**
   - Added `tauri::Manager` and `tauri_plugin_cli::CliExt` imports
   - Added `.setup()` hook to parse CLI arguments
   - Auto-validates and locks project if `--project` arg provided
   - Exits with error if CLI arg points to invalid project
   - Registered new commands: `validate_project_path`, `set_locked_project`, `get_locked_project`

4. **src-tauri/src/commands.rs**
   - Expanded `AppState` to include `locked_project: Mutex<Option<PathBuf>>`
   - Added `validate_project_path()` - validates project structure
   - Added `set_locked_project()` - locks project for session (one-time operation)
   - Added `get_locked_project()` - retrieves currently locked project
   - Modified `start_loop()` - removed `project_path` parameter, reads from locked state

### Frontend (TypeScript/React)

5. **src/components/ProjectPicker.tsx** (NEW)
   - Full-screen modal for project selection
   - Scans for Ralph projects automatically
   - Dropdown for discovered projects
   - Manual path input with folder browser
   - Real-time validation (debounced 500ms)
   - Visual feedback (validating/valid/error states)
   - Cannot be dismissed without valid selection

6. **src/App.tsx**
   - Added two-phase initialization: check locked project → show picker or main UI
   - Queries `get_locked_project()` on mount
   - Renders `<ProjectPicker />` if no project locked
   - Renders main UI if project locked
   - Passes `lockedProject` prop to `<LoopControls />`

7. **src/components/LoopControls.tsx**
   - Removed all project selection UI (dropdown, input, folder browser, scanning logic)
   - Added `lockedProject` prop
   - Displays locked project as read-only (path + name/.ralph)
   - Removed `projectPath` parameter from `start_loop` IPC call
   - Simplified to: locked project display + max iterations + control buttons

## Validation Logic

The `validate_project_path()` function checks:

1. Path exists and is a directory
2. `.ralph/` directory exists
3. `.ralph/prd.md` file exists
4. Canonicalizes path (resolves symlinks)

## Error Messages

- **Path not found**: "Directory not found: {path}"
- **Not a directory**: "Not a directory: {path}"
- **No .ralph folder**: "No .ralph folder found. Is this a Ralph project?"
- **Missing PRD**: ".ralph/prd.md not found. Create PRD first."
- **Already locked**: "Project already locked for this session"
- **No locked project**: "No project locked (bug, restart app)"

## Usage

### Launch with CLI Argument
```bash
ralph --project /path/to/project
```

If valid: App launches with project locked, main UI shows immediately.
If invalid: Error printed to stderr, app exits with code 1.

### Launch Without Argument
```bash
ralph
```

ProjectPicker modal appears:
1. Scans home directory (5 levels deep, max 100 projects)
2. Shows dropdown if multiple projects found
3. Auto-selects if only one project found
4. User can manually type path or browse
5. Validation happens on change (debounced)
6. "Lock Project" button enabled when valid
7. Clicking "Lock Project" transitions to main UI

### Main UI (Locked State)
```
┌─────────────────────────────────────────┐
│ Ralph Loop                       [Idle] │
│                                         │
│ Locked Project:                         │
│ ┌─────────────────────────────────────┐ │
│ │ /home/user/my-project               │ │
│ │ my-project/.ralph                   │ │
│ └─────────────────────────────────────┘ │
│                                         │
│ Max Iterations: [100]                   │
│                                         │
│ [Start] [Pause] [Resume] [Stop]         │
└─────────────────────────────────────────┘
```

## Testing

### Build Verification
```bash
# Backend compiles
cargo check --manifest-path src-tauri/Cargo.toml

# Frontend builds
bun run build

# Full debug build
bun tauri build --debug
```

All checks passed ✓

### Manual Testing Checklist

- [ ] Launch without args → ProjectPicker appears
- [ ] Scan finds existing .ralph projects
- [ ] Dropdown shows discovered projects
- [ ] Manual path input validates correctly
- [ ] Folder browser selects valid path
- [ ] Invalid path shows appropriate error
- [ ] Valid path enables "Lock Project" button
- [ ] Locking project transitions to main UI
- [ ] Locked project displays correctly (read-only)
- [ ] Start loop works without passing project path
- [ ] Launch with `--project /valid/path` → skips picker
- [ ] Launch with `--project /invalid/path` → error + exit
- [ ] Cannot change project during session
- [ ] Close and reopen → must select project again

### Test Projects

Created test project at `/tmp/test-ralph-project`:
```bash
mkdir -p /tmp/test-ralph-project/.ralph
echo "# Test PRD" > /tmp/test-ralph-project/.ralph/prd.md
```

## Known Limitations

1. **No persistence**: Last-used project is not remembered across sessions
2. **No switching**: Cannot change project without restarting app
3. **No project creation**: Must create `.ralph/` structure manually first

## Future Enhancements (Out of Scope)

- Remember last-used project in config file
- "Switch Project" button with confirmation dialog
- Recent projects list in ProjectPicker
- Project creation wizard
- Multi-project workspace mode

## Architecture Notes

- **Backend-first state**: Locked project stored in Rust `AppState` as source of truth
- **Frontend queries backend**: React queries `get_locked_project()` on mount
- **One-time lock**: `set_locked_project()` returns error if already locked
- **No race conditions**: Backend state prevents desync issues
- **CLI args parsed in setup()**: Happens before window shows
- **Blocking validation**: CLI mode validates and locks synchronously

## Traceability

This implementation follows the plan documented in the project transcript at:
`/home/vince/.claude/projects/-home-vince-Projects-ralph4days/dd15393e-de52-4404-9fad-bc73e60cbe4f.jsonl`

All requirements from the plan have been implemented.
