import type { PRDTask } from "@/types/prd";

/**
 * Generate the display ID for a task
 * Format: feature/discipline/number
 */
export function getTaskDisplayId(task: PRDTask): string {
  return `${task.feature}/${task.discipline}/${task.id}`;
}

/**
 * Get the discipline from a task (no parsing needed!)
 */
export function getTaskDiscipline(task: PRDTask): PRDTask["discipline"] {
  return task.discipline;
}
