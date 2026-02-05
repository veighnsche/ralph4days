/**
 * Mock Backend Service for Ralph4days
 *
 * Provides mock implementations of all Tauri commands for frontend development
 * without the Tauri backend running. Uses localStorage to persist state.
 */

import type { PRDData } from "@/types/prd";

const MOCK_PRD_KEY = "ralph_mock_prd_data";
const MOCK_LOCKED_PROJECT_KEY = "ralph_mock_locked_project";

// Default mock PRD data
const DEFAULT_MOCK_PRD: PRDData = {
  schema_version: "1.0",
  project: {
    title: "Mock Project",
    description: "This is a mock project for frontend development",
    created: "2026-02-01",
  },
  tasks: [
    {
      id: "ui/frontend/001",
      title: "Design main dashboard layout",
      description: "Create responsive dashboard with sidebar and main content area",
      status: "done",
      priority: "high",
      tags: ["design", "layout"],
      created: "2026-02-01",
      updated: "2026-02-03",
      completed: "2026-02-03",
      acceptance_criteria: [
        "Responsive on mobile, tablet, desktop",
        "Sidebar collapses on mobile",
        "Dark mode support",
      ],
    },
    {
      id: "ui/frontend/002",
      title: "Implement task list component",
      description: "Build reusable task list with filtering and sorting",
      status: "in_progress",
      priority: "high",
      tags: ["component", "ui"],
      created: "2026-02-02",
      updated: "2026-02-05",
      acceptance_criteria: ["Filter by status, priority", "Sort by date, priority"],
    },
    {
      id: "ui/frontend/003",
      title: "Add task detail sidebar",
      description: "Show full task details in a sliding sidebar",
      status: "pending",
      priority: "medium",
      tags: ["component", "ui"],
      created: "2026-02-03",
      depends_on: ["ui/frontend/002"],
    },
    {
      id: "api/backend/001",
      title: "Setup REST API endpoints",
      description: "Create basic CRUD endpoints for tasks",
      status: "done",
      priority: "critical",
      tags: ["api", "backend"],
      created: "2026-02-01",
      completed: "2026-02-02",
    },
    {
      id: "api/backend/002",
      title: "Add authentication middleware",
      description: "Implement JWT-based authentication",
      status: "blocked",
      priority: "high",
      tags: ["security", "backend"],
      created: "2026-02-02",
      blocked_by: "Waiting for security review",
    },
    {
      id: "data/database/001",
      title: "Design database schema",
      description: "Create tables for users, tasks, projects",
      status: "done",
      priority: "critical",
      tags: ["database", "schema"],
      created: "2026-02-01",
      completed: "2026-02-01",
    },
    {
      id: "tests/testing/001",
      title: "Write unit tests for task CRUD",
      description: "Add comprehensive test coverage for task operations",
      status: "pending",
      priority: "medium",
      tags: ["testing", "backend"],
      created: "2026-02-04",
    },
  ],
  _counters: {
    ui: { frontend: 3 },
    api: { backend: 2 },
    data: { database: 1 },
    tests: { testing: 1 },
  },
};

// Mock backend implementation
export const mockBackend = {
  // Get PRD content as YAML string
  get_prd_content(): Promise<string> {
    const stored = localStorage.getItem(MOCK_PRD_KEY);
    const prdData: PRDData = stored ? JSON.parse(stored) : DEFAULT_MOCK_PRD;

    // Convert to YAML-like string (simplified)
    const yaml = `schema_version: "${prdData.schema_version}"
project:
  title: "${prdData.project.title}"
  description: "${prdData.project.description || ""}"
  created: "${prdData.project.created || ""}"

tasks:
${prdData.tasks
  .map(
    (task) => `  - id: "${task.id}"
    title: "${task.title}"
    description: "${task.description || ""}"
    status: "${task.status}"
    priority: "${task.priority || ""}"
    tags: [${task.tags?.map((t) => `"${t}"`).join(", ") || ""}]
    created: "${task.created || ""}"
    updated: "${task.updated || ""}"
    completed: "${task.completed || ""}"
    ${task.depends_on?.length ? `depends_on: [${task.depends_on.map((d) => `"${d}"`).join(", ")}]` : ""}
    ${task.blocked_by ? `blocked_by: "${task.blocked_by}"` : ""}
    ${task.acceptance_criteria?.length ? `acceptance_criteria:\n${task.acceptance_criteria.map((ac) => `      - "${ac}"`).join("\n")}` : ""}`
  )
  .join("\n")}
`;

    return Promise.resolve(yaml);
  },

  // Get available disciplines
  get_available_disciplines(): Promise<string[]> {
    return Promise.resolve([
      "frontend",
      "backend",
      "database",
      "testing",
      "infrastructure",
      "security",
      "documentation",
      "design",
      "marketing",
      "api",
    ]);
  },

  // Get next task ID preview
  get_next_task_id(feature: string, discipline: string): Promise<string> {
    const stored = localStorage.getItem(MOCK_PRD_KEY);
    const prdData: PRDData = stored ? JSON.parse(stored) : DEFAULT_MOCK_PRD;

    const normalizedFeature = feature.toLowerCase().replace(/\s+/g, "-");
    const counters = prdData._counters || {};
    const featureCounters = counters[normalizedFeature] || {};
    const nextNumber = (featureCounters[discipline] || 0) + 1;

    return Promise.resolve(`${normalizedFeature}/${discipline}/${String(nextNumber).padStart(3, "0")}`);
  },

  // Create task
  create_task(
    feature: string,
    discipline: string,
    title: string,
    description: string | null,
    priority: string | null,
    tags: string[]
  ): Promise<string> {
    const stored = localStorage.getItem(MOCK_PRD_KEY);
    const prdData: PRDData = stored ? JSON.parse(stored) : DEFAULT_MOCK_PRD;

    const normalizedFeature = feature.toLowerCase().replace(/\s+/g, "-");
    const counters = prdData._counters || {};
    const featureCounters = counters[normalizedFeature] || {};
    const nextNumber = (featureCounters[discipline] || 0) + 1;
    const taskId = `${normalizedFeature}/${discipline}/${String(nextNumber).padStart(3, "0")}`;

    const newTask = {
      id: taskId,
      title,
      description: description || undefined,
      status: "pending" as const,
      priority: (priority as "low" | "medium" | "high" | "critical" | undefined) || undefined,
      tags: tags.length > 0 ? tags : undefined,
      created: new Date().toISOString().split("T")[0],
    };

    prdData.tasks.push(newTask);

    // Update counters
    if (!prdData._counters) {
      prdData._counters = {};
    }
    if (!prdData._counters[normalizedFeature]) {
      prdData._counters[normalizedFeature] = {};
    }
    prdData._counters[normalizedFeature][discipline] = nextNumber;

    localStorage.setItem(MOCK_PRD_KEY, JSON.stringify(prdData));

    return Promise.resolve(taskId);
  },

  // Get existing features
  get_existing_features(): Promise<string[]> {
    const stored = localStorage.getItem(MOCK_PRD_KEY);
    const prdData: PRDData = stored ? JSON.parse(stored) : DEFAULT_MOCK_PRD;

    const features = new Set<string>();
    for (const task of prdData.tasks) {
      const feature = task.id.split("/")[0];
      if (feature) {
        features.add(feature);
      }
    }

    return Promise.resolve(Array.from(features).sort());
  },

  // Validate project path (always succeeds in mock mode)
  validate_project_path(_path: string): Promise<void> {
    return Promise.resolve();
  },

  // Set locked project
  set_locked_project(path: string): Promise<void> {
    localStorage.setItem(MOCK_LOCKED_PROJECT_KEY, path);
    return Promise.resolve();
  },

  // Get locked project
  get_locked_project(): Promise<string | null> {
    const locked = localStorage.getItem(MOCK_LOCKED_PROJECT_KEY);
    return Promise.resolve(locked);
  },

  // Scan for projects (returns mock projects)
  scan_for_ralph_projects(): Promise<Array<{ name: string; path: string }>> {
    return Promise.resolve([
      { name: "Mock Project 1", path: "/mock/project1" },
      { name: "Mock Project 2", path: "/mock/project2" },
      { name: "Demo Project", path: "/mock/demo" },
    ]);
  },

  // Get current directory
  get_current_dir(): Promise<string> {
    return Promise.resolve("/mock/home");
  },

  // Initialize ralph project
  initialize_ralph_project(_path: string, _projectTitle: string): Promise<void> {
    return Promise.resolve();
  },

  // Loop control commands (no-ops in mock mode)
  start_loop(_maxIterations: number): Promise<void> {
    console.warn("[Mock Backend] Loop commands are no-ops in mock mode");
    return Promise.resolve();
  },

  pause_loop(): Promise<void> {
    console.warn("[Mock Backend] Loop commands are no-ops in mock mode");
    return Promise.resolve();
  },

  resume_loop(): Promise<void> {
    console.warn("[Mock Backend] Loop commands are no-ops in mock mode");
    return Promise.resolve();
  },

  stop_loop(): Promise<void> {
    console.warn("[Mock Backend] Loop commands are no-ops in mock mode");
    return Promise.resolve();
  },

  get_loop_state(): Promise<{ status: string; iteration: number }> {
    return Promise.resolve({ status: "idle", iteration: 0 });
  },

  // Reset mock data to defaults
  resetMockData(): void {
    localStorage.setItem(MOCK_PRD_KEY, JSON.stringify(DEFAULT_MOCK_PRD));
    localStorage.removeItem(MOCK_LOCKED_PROJECT_KEY);
  },
};

/**
 * Check if running in Tauri environment
 */
export function isTauriEnvironment(): boolean {
  return typeof window !== "undefined" && "__TAURI__" in window;
}

/**
 * Universal invoke function that uses Tauri when available, otherwise uses mock backend
 */
export async function universalInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauriEnvironment()) {
    // Use real Tauri invoke
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<T>(command, args);
  }

  // Use mock backend
  const mockFn = mockBackend[command as keyof typeof mockBackend] as (...args: unknown[]) => Promise<T>;
  if (!mockFn) {
    throw new Error(`Mock backend does not implement command: ${command}`);
  }

  // Call mock function with spread args
  return mockFn(...Object.values(args || {}));
}
