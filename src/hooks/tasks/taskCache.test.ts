import { QueryClient } from '@tanstack/react-query'
import { describe, expect, it } from 'vitest'
import { buildInvokeQueryKey } from '@/hooks/api'
import type { Task, TaskListItem } from '@/types/generated'
import {
  buildTaskListItemFromTask,
  patchTaskInTaskDetailCache,
  patchTaskInTaskDetailCacheOptimistically,
  patchTaskListItemInTaskListCache,
  patchTaskListItemInTaskListCacheOptimistically
} from './taskCache'

function createTask(id: number, overrides: Partial<Task> = {}): Task {
  return {
    id,
    subsystem: 'core-platform',
    discipline: 'frontend',
    title: `Task ${id}`,
    status: 'pending',
    tags: [],
    dependsOn: [],
    created: '2026-01-01',
    updated: '2026-01-01',
    completed: undefined,
    acceptanceCriteria: [],
    contextFiles: [],
    outputArtifacts: [],
    signals: [],
    subsystemDisplayName: 'Core Platform',
    subsystemAcronym: 'CP',
    disciplineDisplayName: 'Frontend',
    disciplineAcronym: 'FE',
    disciplineIcon: 'bot',
    disciplineColor: '#43a047',
    ...overrides
  }
}

function createTaskListItem(id: number, overrides: Partial<TaskListItem> = {}): TaskListItem {
  return {
    id,
    subsystem: 'core-platform',
    discipline: 'frontend',
    title: `Task ${id}`,
    status: 'pending',
    tags: [],
    dependsOn: [],
    acceptanceCriteriaCount: 0,
    signalCount: 0,
    subsystemDisplayName: 'Core Platform',
    subsystemAcronym: 'CP',
    disciplineDisplayName: 'Frontend',
    disciplineAcronym: 'FE',
    disciplineIcon: 'bot',
    disciplineColor: '#43a047',
    ...overrides
  }
}

describe('taskCache', () => {
  it('maps a Task payload into a TaskListItem payload', () => {
    const task = createTask(11, {
      priority: 'high',
      acceptanceCriteria: ['[ ] one', '[x] two'],
      signals: [{ id: 1, author: 'human', body: 'hello' } as Task['signals'][number]]
    })

    expect(buildTaskListItemFromTask(task)).toEqual(
      createTaskListItem(11, {
        priority: 'high',
        acceptanceCriteriaCount: 2,
        signalCount: 1
      })
    )
  })

  it('patches the workspace get_task cache with a returned task payload', () => {
    const queryClient = new QueryClient()
    const queryKey = buildInvokeQueryKey('get_task', { id: 11 }, 'workspace')
    const original = createTask(11, { title: 'Before' })
    const updated = createTask(11, { title: 'After' })

    queryClient.setQueryData(queryKey, original)
    patchTaskInTaskDetailCache(queryClient, updated, 'workspace')

    expect(queryClient.getQueryData<Task>(queryKey)).toEqual(updated)
  })

  it('optimistically patches and rolls back a task in workspace get_task cache', () => {
    const queryClient = new QueryClient()
    const queryKey = buildInvokeQueryKey('get_task', { id: 11 }, 'workspace')
    const original = createTask(11, { title: 'Before' })
    const optimistic = createTask(11, { title: 'Optimistic' })

    queryClient.setQueryData(queryKey, original)

    const rollback = patchTaskInTaskDetailCacheOptimistically(queryClient, optimistic, 'workspace')

    expect(queryClient.getQueryData<Task>(queryKey)).toEqual(optimistic)

    rollback()

    expect(queryClient.getQueryData<Task>(queryKey)).toEqual(original)
  })

  it('patches the workspace get_task_list_items cache with a returned list item payload', () => {
    const queryClient = new QueryClient()
    const queryKey = buildInvokeQueryKey('get_task_list_items', undefined, 'workspace')
    const original = [createTaskListItem(10), createTaskListItem(11)]
    const updated = createTaskListItem(11, { status: 'done' })

    queryClient.setQueryData(queryKey, original)
    patchTaskListItemInTaskListCache(queryClient, updated, 'workspace')

    expect(queryClient.getQueryData<TaskListItem[]>(queryKey)).toEqual([createTaskListItem(10), updated])
  })

  it('optimistically patches and rolls back a list item in workspace get_task_list_items cache', () => {
    const queryClient = new QueryClient()
    const queryKey = buildInvokeQueryKey('get_task_list_items', undefined, 'workspace')
    const original = [createTaskListItem(10), createTaskListItem(11)]
    const optimistic = createTaskListItem(11, { priority: 'high' })

    queryClient.setQueryData(queryKey, original)

    const rollback = patchTaskListItemInTaskListCacheOptimistically(queryClient, optimistic, 'workspace')

    expect(queryClient.getQueryData<TaskListItem[]>(queryKey)).toEqual([createTaskListItem(10), optimistic])

    rollback()

    expect(queryClient.getQueryData<TaskListItem[]>(queryKey)).toEqual(original)
  })
})
