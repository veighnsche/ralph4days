# Test Fixtures

Development fixture for testing Ralph Loop.

## Single Task Fixture

**Location**: `fixtures/single-task/`
**Purpose**: Minimal smoke test - one simple file creation task
**Expected**: Complete in 1 iteration

### Task

Create `hello.txt` containing "Hello, World!"

### Project Structure

```
single-task/
├── .ralph/
│   ├── prd.md          # Task list (checkbox format)
│   └── CLAUDE.md       # Project context for Claude
└── README.md           # Human-readable docs
```

### Required File: prd.md

Ralph expects `.ralph/prd.md` with tasks in checkbox format:

```markdown
# Project Name

## Tasks

- [ ] Task 1 description
- [ ] Task 2 description
- [x] Completed task

## Success Criteria

- Acceptance criteria here
```

### Runtime Files (created during execution)

```
.ralph/
├── progress.txt        # Iteration log (appended after each iteration)
└── learnings.txt       # Patterns learned (updated by Opus reviews)
```

## Usage

```bash
# Start Ralph with this fixture
bun tauri dev
# Then select: fixtures/single-task

# Or use the scanner to find it automatically
```

## Verification

After Ralph completes:
1. Check that `hello.txt` exists in `fixtures/single-task/`
2. Verify it contains "Hello, World!"
3. Check that `prd.md` has `[x]` checkbox
4. Check `progress.txt` was created with iteration log
