import type { QueryClient } from '@tanstack/react-query'
import { buildInvokeQueryKey, type InvokeQueryDomain } from '@/hooks/api'
import type { Task, TaskListItem } from '@/types/generated'

export function buildTaskListItemFromTask(task: Task): TaskListItem {
  const tags = Array.isArray(task.tags) ? task.tags : []
  const dependsOn = Array.isArray(task.dependsOn) ? task.dependsOn : []
  const acceptanceCriteria = Array.isArray(task.acceptanceCriteria) ? task.acceptanceCriteria : []
  const signals = Array.isArray(task.signals) ? task.signals : []

  return {
    id: task.id,
    subsystem: task.subsystem,
    discipline: task.discipline,
    title: task.title,
    description: task.description,
    status: task.status,
    priority: task.priority,
    tags,
    dependsOn,
    acceptanceCriteriaCount: acceptanceCriteria.length,
    signalCount: signals.length,
    provenance: task.provenance,
    subsystemDisplayName: task.subsystemDisplayName,
    subsystemAcronym: task.subsystemAcronym,
    disciplineDisplayName: task.disciplineDisplayName,
    disciplineAcronym: task.disciplineAcronym,
    disciplineIcon: task.disciplineIcon,
    disciplineColor: task.disciplineColor
  }
}

export function patchTaskInTaskDetailCache(
  queryClient: QueryClient,
  updatedTask: Task,
  queryDomain: InvokeQueryDomain
): void {
  const queryKey = buildInvokeQueryKey('tasks_get', { id: updatedTask.id }, queryDomain)
  if (queryClient.getQueryData<Task>(queryKey) === undefined) return
  queryClient.setQueryData<Task>(queryKey, updatedTask)
}

export function patchTaskInTaskDetailCacheOptimistically(
  queryClient: QueryClient,
  optimisticTask: Task,
  queryDomain: InvokeQueryDomain
): () => void {
  const queryKey = buildInvokeQueryKey('tasks_get', { id: optimisticTask.id }, queryDomain)
  const current = queryClient.getQueryData<Task>(queryKey)
  if (current === undefined) return () => {}

  queryClient.setQueryData<Task>(queryKey, optimisticTask)

  return () => {
    queryClient.setQueryData<Task>(queryKey, current)
  }
}

export function patchTaskListItemInTaskListCache(
  queryClient: QueryClient,
  updatedTaskListItem: TaskListItem,
  queryDomain: InvokeQueryDomain
): void {
  const queryKey = buildInvokeQueryKey('tasks_list_items', undefined, queryDomain)
  const currentItems = queryClient.getQueryData<TaskListItem[]>(queryKey)
  if (!currentItems) return

  const itemIndex = currentItems.findIndex(item => item.id === updatedTaskListItem.id)
  if (itemIndex === -1) return

  const nextItems = [...currentItems]
  nextItems[itemIndex] = updatedTaskListItem
  queryClient.setQueryData<TaskListItem[]>(queryKey, nextItems)
}

export function patchTaskListItemInTaskListCacheOptimistically(
  queryClient: QueryClient,
  optimisticTaskListItem: TaskListItem,
  queryDomain: InvokeQueryDomain
): () => void {
  const queryKey = buildInvokeQueryKey('tasks_list_items', undefined, queryDomain)
  const currentItems = queryClient.getQueryData<TaskListItem[]>(queryKey)
  if (!currentItems) return () => {}

  const itemIndex = currentItems.findIndex(item => item.id === optimisticTaskListItem.id)
  if (itemIndex === -1) return () => {}

  const currentItem = currentItems[itemIndex]
  const nextItems = [...currentItems]
  nextItems[itemIndex] = optimisticTaskListItem
  queryClient.setQueryData<TaskListItem[]>(queryKey, nextItems)

  return () => {
    const rollbackItems = queryClient.getQueryData<TaskListItem[]>(queryKey)
    if (!rollbackItems) return
    const rollbackIndex = rollbackItems.findIndex(item => item.id === currentItem.id)
    if (rollbackIndex === -1) return

    const nextRollbackItems = [...rollbackItems]
    nextRollbackItems[rollbackIndex] = currentItem
    queryClient.setQueryData<TaskListItem[]>(queryKey, nextRollbackItems)
  }
}
