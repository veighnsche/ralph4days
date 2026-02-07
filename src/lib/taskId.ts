import type { Task } from "@/types/prd";

/**
 * Generate the display ID for a task
 * Format: feature/discipline/number
 */
export function getTaskDisplayId(task: Task): string {
  return `${task.feature}/${task.discipline}/${task.id}`;
}

/**
 * Get the discipline from a task (no parsing needed!)
 */
export function getTaskDiscipline(task: Task): Task["discipline"] {
  return task.discipline;
}
