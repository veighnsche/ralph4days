import type { Task } from '@/types/generated'

export interface UpdateTaskParams {
  id: number
  subsystem: string
  discipline: string
  title: string
  description?: string
  priority?: Task['priority']
  tags: string[]
  depends_on: number[]
  acceptance_criteria: string[]
  context_files: string[]
  output_artifacts: string[]
  hints?: string
  estimated_turns?: number
  provenance?: Task['provenance']
  agent?: string
  model?: string
  effort?: string
  thinking?: boolean
}

export interface UpdateTaskVariables {
  params: UpdateTaskParams
}

export function buildOptimisticTaskFromUpdateTask(currentTask: Task, params: UpdateTaskParams): Task {
  if (currentTask.id !== params.id) {
    throw new Error(`[task-cache] Cannot build optimistic task for id ${params.id}; current task is ${currentTask.id}`)
  }

  return {
    ...currentTask,
    subsystem: params.subsystem,
    discipline: params.discipline,
    title: params.title,
    description: params.description,
    priority: params.priority,
    tags: params.tags,
    dependsOn: params.depends_on,
    acceptanceCriteria: params.acceptance_criteria,
    contextFiles: params.context_files,
    outputArtifacts: params.output_artifacts,
    hints: params.hints,
    estimatedTurns: params.estimated_turns,
    provenance: params.provenance,
    agent: params.agent,
    model: params.model,
    effort: params.effort,
    thinking: params.thinking
  }
}
