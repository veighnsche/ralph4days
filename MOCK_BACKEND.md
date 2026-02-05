# Mock Backend for Frontend Development

The Ralph4days frontend includes a built-in mock backend that allows you to develop and test the UI without running the Tauri backend.

## How It Works

The mock backend automatically activates when running the frontend dev server outside of Tauri:

```bash
npm run dev  # or: just dev-frontend
```

When `window.__TAURI__` is not detected, the app uses mock data from `src/services/mockBackend.ts`.

## Features

### ✅ What Works in Mock Mode

- **PRD Viewer**: Browse sample tasks across all disciplines (frontend, backend, database, testing, etc.)
- **Task Creation**: Create new tasks (stored in localStorage)
- **Task Filtering**: Filter by status, priority, tags
- **Task Details**: View task details in sidebar
- **Discipline Colors**: See discipline-specific icons and colors
- **Visual Indicator**: Amber warning banner shows when using mock data

### ⚠️ What Doesn't Work

- **Loop Execution**: Loop control buttons are no-ops (logs warning to console)
- **Real File Operations**: No actual `.ralph/prd.yaml` files are read/written
- **Window Title**: Window title updates are skipped in mock mode

## Sample Data

The mock backend includes 7 sample tasks:

- **ui/frontend/001** - Design main dashboard layout (Done)
- **ui/frontend/002** - Implement task list component (In Progress)
- **ui/frontend/003** - Add task detail sidebar (Pending)
- **api/backend/001** - Setup REST API endpoints (Done)
- **api/backend/002** - Add authentication middleware (Blocked)
- **data/database/001** - Design database schema (Done)
- **tests/testing/001** - Write unit tests for task CRUD (Pending)

## Data Persistence

Mock data persists in browser localStorage:

- **Key**: `ralph_mock_prd_data`
- **Location**: Browser LocalStorage
- **Reset**: Clear browser storage or use dev tools

To reset to default sample data:

```javascript
// In browser console:
localStorage.removeItem('ralph_mock_prd_data');
location.reload();
```

## Creating Tasks

Tasks created in mock mode:
1. Generate proper 3-tier IDs (feature/discipline/number)
2. Update internal counters
3. Persist to localStorage
4. Appear immediately in the UI

## Development Workflow

1. **Start frontend only**:
   ```bash
   npm run dev
   ```

2. **See mock data banner** at top of PRD viewer

3. **Develop UI features** using sample data

4. **Switch to live mode** when ready to test with real backend:
   ```bash
   just dev  # Starts both frontend + Tauri backend
   ```

## Architecture

### universalInvoke()

All components use `universalInvoke()` instead of direct Tauri `invoke()`:

```typescript
import { universalInvoke } from "@/services/mockBackend";

// Automatically uses Tauri or mock backend
const data = await universalInvoke<string>("get_prd_content");
```

### isTauriEnvironment()

Check if running in Tauri:

```typescript
import { isTauriEnvironment } from "@/services/mockBackend";

if (isTauriEnvironment()) {
  // Real Tauri backend available
} else {
  // Using mock backend
}
```

## Debugging

Mock backend logs to console:

```
[Mock Backend] Loop commands are no-ops in mock mode
```

Enable verbose logging in `src/services/mockBackend.ts` if needed.

## Testing Benefits

- ✅ Test UI without running Rust backend
- ✅ Faster development iteration
- ✅ No need to manage `.ralph/` project structure
- ✅ Consistent sample data across sessions
- ✅ Easy to reproduce UI bugs
- ✅ Test error states and edge cases
