import { QueryClient } from '@tanstack/react-query'
import { describe, expect, it } from 'vitest'
import { buildInvokeQueryKey } from '@/hooks/api'
import type { Task } from '@/types/generated'
import { patchTaskInTasksCache, replaceTaskInList } from './taskCache'

function createTask(id: number, overrides: Partial<Task> = {}): Task {
  return {
    id,
    subsystem: 'core-platform',
    discipline: 'frontend',
    title: `Task ${id}`,
    status: 'pending',
    tags: [],
    dependsOn: [],
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

describe('taskCache', () => {
  it('replaces a single task while preserving list order', () => {
    const original = [createTask(1), createTask(2), createTask(3)]
    const updatedTask = createTask(2, { priority: 'high', title: 'Updated title' })

    const next = replaceTaskInList(original, updatedTask)

    expect(next.map(task => task.id)).toEqual([1, 2, 3])
    expect(next[1]).toEqual(updatedTask)
    expect(next[0]).toBe(original[0])
    expect(next[2]).toBe(original[2])
  })

  it('fails loudly when the updated task is missing from the cached list', () => {
    const original = [createTask(1), createTask(2)]

    expect(() => replaceTaskInList(original, createTask(999))).toThrowError('[list-cache] Task 999 missing from cache')
  })

  it('patches the workspace get_tasks cache with a returned task payload', () => {
    const queryClient = new QueryClient()
    const queryKey = buildInvokeQueryKey('get_tasks', undefined, 'workspace')
    const original = [createTask(10), createTask(11)]
    const updated = createTask(11, { status: 'done' })

    queryClient.setQueryData(queryKey, original)
    patchTaskInTasksCache(queryClient, updated, 'workspace')

    expect(queryClient.getQueryData<Task[]>(queryKey)).toEqual([createTask(10), updated])
  })

  it('fails loudly when get_tasks cache is missing for the mutation domain', () => {
    const queryClient = new QueryClient()

    expect(() => patchTaskInTasksCache(queryClient, createTask(1), 'workspace')).toThrowError(
      '[list-cache] get_tasks cache is missing for workspace domain'
    )
  })
})
