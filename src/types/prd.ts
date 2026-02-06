export interface PRDTask {
  id: number;
  feature: string;
  discipline: string;
  title: string;
  description?: string;
  status: "pending" | "in_progress" | "done" | "blocked" | "skipped";
  priority?: "low" | "medium" | "high" | "critical";
  tags?: string[];
  dependsOn?: number[];
  blockedBy?: string;
  created?: string;
  updated?: string;
  completed?: string;
  acceptanceCriteria?: string[];
}

/** Task with pre-joined feature/discipline display data from backend */
export interface EnrichedTask extends PRDTask {
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

export type StatusFilter = "all" | "pending" | "in_progress" | "done" | "blocked" | "skipped";
export type PriorityFilter = "all" | "low" | "medium" | "high" | "critical";

export interface Feature {
  name: string;
  displayName: string;
  acronym?: string;
  description?: string;
  created?: string;
}
