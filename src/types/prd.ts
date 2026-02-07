/** Task status stored in YAML */
export type TaskStatus = "pending" | "in_progress" | "done" | "blocked" | "skipped";

/** Inferred task status (computed from TaskStatus + dependency graph) */
export type InferredTaskStatus =
  | "ready" // pending + all deps met + not blocked
  | "waiting_on_deps" // pending + some deps not done
  | "externally_blocked" // status == blocked (manual)
  | "in_progress" // status == in_progress
  | "done" // status == done
  | "skipped"; // status == skipped

export type TaskProvenance = "agent" | "human" | "system";

export interface McpServerConfig {
  name: string;
  command: string;
  args?: string[];
  env?: Record<string, string>;
}

export type CommentAuthor = "human" | "agent";

export interface TaskComment {
  id: number;
  author: CommentAuthor;
  agent_task_id?: number; // snake_case — matches Rust serde output (no rename_all on TaskComment)
  body: string;
  created?: string;
}

export interface PRDTask {
  id: number;
  feature: string;
  discipline: string;
  title: string;
  description?: string;
  status: TaskStatus;
  priority?: "low" | "medium" | "high" | "critical";
  tags?: string[];
  dependsOn?: number[];
  blockedBy?: string;
  created?: string;
  updated?: string;
  completed?: string;
  acceptanceCriteria?: string[];
  // Execution context
  contextFiles?: string[];
  outputArtifacts?: string[];
  hints?: string;
  estimatedTurns?: number;
  // Provenance & history
  provenance?: TaskProvenance;
  comments?: TaskComment[];
}

/** Task with pre-joined feature/discipline display data from backend */
export interface EnrichedTask extends PRDTask {
  inferredStatus: InferredTaskStatus;
  featureDisplayName: string;
  featureAcronym: string;
  disciplineDisplayName: string;
  disciplineAcronym: string;
  disciplineIcon: string;
  disciplineColor: string;
}

/** Stats for a group of tasks (feature or discipline) */
export interface GroupStats {
  name: string;
  displayName: string;
  total: number;
  done: number;
  pending: number;
  inProgress: number;
  blocked: number;
  skipped: number;
}

/** Overall project progress */
export interface ProjectProgress {
  totalTasks: number;
  doneTasks: number;
  progressPercent: number;
}

/** Project info from metadata */
export interface ProjectInfo {
  title: string;
  description?: string;
  created?: string;
}

export type StatusFilter =
  | "all"
  // Actual statuses (match TaskStatus)
  | "pending"
  | "in_progress"
  | "blocked"
  | "done"
  | "skipped"
  // Inferred statuses (additional computed states)
  | "ready" // Inferred: pending + all deps met
  | "waiting_on_deps"; // Inferred: pending + unmet deps
export type PriorityFilter = "all" | "low" | "medium" | "high" | "critical";

export interface Feature {
  name: string;
  displayName: string;
  acronym?: string;
  description?: string;
  created?: string;
  // Knowledge context
  knowledgePaths?: string[];
  contextFiles?: string[];
}
