import type { Discipline, PRDData, PRDTask } from "@/types/prd";

const VALID_DISCIPLINES: readonly Discipline[] = [
  "frontend",
  "backend",
  "database",
  "testing",
  "infra",
  "security",
  "docs",
  "design",
  "promo",
  "api",
] as const;

function isDiscipline(value: unknown): value is Discipline {
  return typeof value === "string" && VALID_DISCIPLINES.includes(value as Discipline);
}

export function validateTask(task: unknown): task is PRDTask {
  if (!task || typeof task !== "object") {
    throw new Error("Task must be an object");
  }

  const t = task as Record<string, unknown>;

  // Validate required fields
  if (typeof t.id !== "number") {
    throw new Error(`Task ID must be a number, got: ${typeof t.id} (${JSON.stringify(t.id)})`);
  }

  if (typeof t.feature !== "string") {
    throw new Error(`Task feature must be a string, got: ${typeof t.feature}`);
  }

  if (!isDiscipline(t.discipline)) {
    throw new Error(`Task discipline must be one of ${VALID_DISCIPLINES.join(", ")}, got: ${t.discipline}`);
  }

  if (typeof t.title !== "string") {
    throw new Error(`Task title must be a string, got: ${typeof t.title}`);
  }

  return true;
}

export function validatePRDData(data: unknown): PRDData {
  if (!data || typeof data !== "object") {
    throw new Error("PRD data must be an object");
  }

  const d = data as Record<string, unknown>;

  if (!d.tasks || !Array.isArray(d.tasks)) {
    throw new Error("PRD data must have a tasks array");
  }

  // Validate each task
  for (let i = 0; i < d.tasks.length; i++) {
    try {
      validateTask(d.tasks[i]);
    } catch (err) {
      throw new Error(`Task ${i} invalid: ${err instanceof Error ? err.message : String(err)}`);
    }
  }

  // Type assertion is safe here because we've validated all required fields
  return data as PRDData;
}
