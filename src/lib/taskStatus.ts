import type { TaskStatus } from '@/types/generated'

export type InferredTaskStatus =
  | 'draft'
  | 'ready'
  | 'waiting_on_deps'
  | 'externally_blocked'
  | 'in_progress'
  | 'done'
  | 'skipped'

type TaskDependencyStatus = { id: number; status: TaskStatus }
type TaskWithDependencies = { status: TaskStatus; dependsOn: number[] }

export function computeInferredStatus(
  task: TaskWithDependencies,
  allTasks: TaskDependencyStatus[]
): InferredTaskStatus {
  switch (task.status) {
    case 'draft':
      return 'draft'
    case 'in_progress':
      return 'in_progress'
    case 'done':
      return 'done'
    case 'skipped':
      return 'skipped'
    case 'blocked':
      return 'externally_blocked'
    case 'pending': {
      const allDepsMet = task.dependsOn.every(depId => allTasks.find(t => t.id === depId)?.status === 'done')
      return allDepsMet ? 'ready' : 'waiting_on_deps'
    }
  }
}

export function shouldShowInferredStatus(actualStatus: TaskStatus, inferredStatus: InferredTaskStatus): boolean {
  // WHY: Non-pending statuses map 1:1 to inferred (in_progress → in_progress), so don't duplicate
  if (actualStatus === 'pending') {
    return inferredStatus === 'ready' || inferredStatus === 'waiting_on_deps'
  }
  return false
}

export function getInferredStatusExplanation(
  actualStatus: TaskStatus,
  inferredStatus: InferredTaskStatus,
  dependsOnCount = 0
): string {
  if (actualStatus === 'pending') {
    if (inferredStatus === 'ready') {
      return 'All dependencies met - ready to work on'
    }
    if (inferredStatus === 'waiting_on_deps') {
      const depText = dependsOnCount === 1 ? '1 dependency' : `${dependsOnCount} dependencies`
      return `Waiting on ${depText}`
    }
  }

  return ''
}
