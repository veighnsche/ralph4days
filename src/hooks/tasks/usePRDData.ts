import { type InvokeQueryDomain, useInvoke } from '@/hooks/api'
import type { TaskListItem } from '@/types/generated'

function isTaskListItemShape(value: unknown): value is TaskListItem {
  if (!value || typeof value !== 'object') return false
  const candidate = value as Partial<TaskListItem>
  return (
    typeof candidate.id === 'number' &&
    typeof candidate.title === 'string' &&
    typeof candidate.subsystem === 'string' &&
    typeof candidate.discipline === 'string' &&
    Array.isArray(candidate.tags) &&
    Array.isArray(candidate.dependsOn) &&
    typeof candidate.acceptanceCriteriaCount === 'number' &&
    typeof candidate.signalCount === 'number'
  )
}

export function usePRDData(queryDomain: InvokeQueryDomain = 'workspace') {
  const { data, isLoading, error, refetch } = useInvoke<TaskListItem[], TaskListItem[]>(
    'get_task_list_items',
    undefined,
    {
      queryDomain,
      select: nextData => {
        if (!Array.isArray(nextData)) {
          throw new Error('Expected task list to be an array')
        }
        for (const [index, task] of nextData.entries()) {
          if (!isTaskListItemShape(task)) {
            throw new Error(`Invalid task list item payload at index ${index.toString()}`)
          }
        }
        return nextData
      }
    }
  )
  const tasks = data ?? null

  return {
    tasks,
    isLoading,
    error: error ? `Failed to load tasks: ${error.message}` : null,
    refetch
  }
}
