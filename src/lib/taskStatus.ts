import type { InferredTaskStatus, TaskStatus } from "@/types/prd";

/**
 * Determines if the inferred status should be displayed.
 * Only show inferred status when it provides additional meaningful information
 * beyond the actual status.
 */
export function shouldShowInferredStatus(actualStatus: TaskStatus, inferredStatus: InferredTaskStatus): boolean {
  // If actual status is "pending", show inferred status if it's different
  // (e.g., "ready" or "waiting_on_deps")
  if (actualStatus === "pending") {
    return inferredStatus === "ready" || inferredStatus === "waiting_on_deps";
  }

  // For all other actual statuses, don't show inferred status
  // because they map 1:1 (in_progress -> in_progress, done -> done, etc.)
  return false;
}

/**
 * Get a human-readable explanation of why a task has a particular inferred status.
 */
export function getInferredStatusExplanation(
  actualStatus: TaskStatus,
  inferredStatus: InferredTaskStatus,
  dependsOnCount: number = 0
): string {
  if (actualStatus === "pending") {
    if (inferredStatus === "ready") {
      return "All dependencies met - ready to work on";
    }
    if (inferredStatus === "waiting_on_deps") {
      const depText = dependsOnCount === 1 ? "1 dependency" : `${dependsOnCount} dependencies`;
      return `Waiting on ${depText}`;
    }
  }

  return "";
}
