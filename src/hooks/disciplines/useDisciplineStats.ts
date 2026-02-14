import { useMemo } from 'react'
import { type InvokeQueryDomain, useInvoke } from '@/hooks/api'
import { computeDisciplineStats, computeProjectProgress } from '@/lib/stats'
import type { TaskListItem } from '@/types/generated'
import { useDisciplines } from './useDisciplines'

const EMPTY_TASKS: TaskListItem[] = []

export function useDisciplineStats(queryDomain: InvokeQueryDomain = 'workspace') {
  const { disciplines, error: disciplinesError, isLoading: disciplinesLoading } = useDisciplines(queryDomain)
  const {
    data: tasksData,
    isLoading: tasksLoading,
    error: tasksError
  } = useInvoke<TaskListItem[]>('tasks_list_items', undefined, {
    queryDomain
  })
  const tasks = tasksData ?? EMPTY_TASKS

  const statsMap = useMemo(() => computeDisciplineStats(tasks, disciplines), [tasks, disciplines])
  const progress = useMemo(() => computeProjectProgress(tasks), [tasks])

  return {
    disciplines,
    statsMap,
    progress: {
      total: progress.totalTasks,
      done: progress.doneTasks,
      percent: progress.progressPercent
    },
    isLoading: disciplinesLoading || tasksLoading,
    error: disciplinesError ?? (tasksError ? String(tasksError) : null)
  }
}
