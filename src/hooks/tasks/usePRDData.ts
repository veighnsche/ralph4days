import { type InvokeQueryDomain, useInvoke } from '@/hooks/api'
import type { Task } from '@/types/generated'

function isTaskShape(value: unknown): value is Task {
  if (!value || typeof value !== 'object') return false
  const candidate = value as Partial<Task>
  return (
    typeof candidate.id === 'number' &&
    typeof candidate.title === 'string' &&
    typeof candidate.subsystem === 'string' &&
    typeof candidate.discipline === 'string' &&
    Array.isArray(candidate.tags)
  )
}

export function usePRDData(queryDomain: InvokeQueryDomain = 'app') {
  const { data, isLoading, error, refetch } = useInvoke<Task[]>('get_tasks', undefined, {
    queryDomain
  })
  const tasks = Array.isArray(data) ? data.filter(isTaskShape) : null

  return {
    tasks,
    isLoading,
    error: error ? `Failed to load tasks: ${error.message}` : null,
    refetch
  }
}
