export interface PRDTask {
  id: string;
  title: string;
  description?: string;
  status: "pending" | "in_progress" | "done" | "blocked" | "skipped";
  priority?: "low" | "medium" | "high" | "critical";
  tags?: string[];
  depends_on?: string[];
  blocked_by?: string;
  created?: string;
  updated?: string;
  completed?: string;
  acceptance_criteria?: string[];
}

export interface PRDProject {
  title: string;
  description?: string;
  created?: string;
}

export interface PRDData {
  schema_version: string;
  project: PRDProject;
  tasks: PRDTask[];
  _counters?: Record<string, Record<string, number>>;
}

export type StatusFilter = "all" | "pending" | "in_progress" | "done" | "blocked" | "skipped";
export type PriorityFilter = "all" | "low" | "medium" | "high" | "critical";
