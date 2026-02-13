import type { QueryClient } from '@tanstack/react-query'
import { type InvokeQueryDomain, replaceListItemInArray, replaceListItemInQueryCache } from '@/hooks/api'
import type { Task } from '@/types/generated'

export function replaceTaskInList(tasks: Task[], updatedTask: Task): Task[] {
  return replaceListItemInArray({
    items: tasks,
    item: updatedTask,
    getKey: task => task.id,
    entityLabel: 'Task'
  })
}

export function patchTaskInTasksCache(
  queryClient: QueryClient,
  updatedTask: Task,
  queryDomain: InvokeQueryDomain
): void {
  replaceListItemInQueryCache({
    queryClient,
    queryDomain,
    command: 'get_tasks',
    item: updatedTask,
    getKey: task => task.id,
    entityLabel: 'Task'
  })
}
