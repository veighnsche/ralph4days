import type { Task, TasksUpdateArgs } from '@/types/generated'

export type UpdateTaskVariables = TasksUpdateArgs

export function buildUpdateArgsFromTask(task: Task, overrides: Partial<TasksUpdateArgs> = {}): TasksUpdateArgs {
  if (overrides.id != null && overrides.id !== task.id) {
    throw new Error(
      `[task-cache] buildUpdateArgsFromTask cannot override id ${task.id}; got ${overrides.id.toString()}`
    )
  }

  const base: TasksUpdateArgs = {
    id: task.id,
    subsystem: task.subsystem,
    discipline: task.discipline,
    title: task.title,
    description: task.description,
    priority: task.priority,
    tags: task.tags,
    dependsOn: task.dependsOn,
    acceptanceCriteria: task.acceptanceCriteria,
    contextFiles: task.contextFiles,
    outputArtifacts: task.outputArtifacts,
    hints: task.hints,
    estimatedTurns: task.estimatedTurns,
    provenance: task.provenance,
    agent: task.agent,
    model: task.model,
    effort: task.effort,
    thinking: task.thinking
  }

  // Ensure id always matches the task; overrides are allowed for all other fields.
  return { ...base, ...overrides, id: task.id }
}

export function buildOptimisticTaskFromUpdateTask(currentTask: Task, args: TasksUpdateArgs): Task {
  if (currentTask.id !== args.id) {
    throw new Error(`[task-cache] Cannot build optimistic task for id ${args.id}; current task is ${currentTask.id}`)
  }

  return {
    ...currentTask,
    subsystem: args.subsystem,
    discipline: args.discipline,
    title: args.title,
    description: args.description,
    priority: args.priority,
    tags: args.tags,
    dependsOn: args.dependsOn,
    acceptanceCriteria: args.acceptanceCriteria,
    contextFiles: args.contextFiles,
    outputArtifacts: args.outputArtifacts,
    hints: args.hints,
    estimatedTurns: args.estimatedTurns,
    provenance: args.provenance,
    agent: args.agent,
    model: args.model,
    effort: args.effort,
    thinking: args.thinking
  }
}
