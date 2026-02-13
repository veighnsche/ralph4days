import { useMemo } from 'react'
import { type InvokeQueryDomain, useInvoke } from '@/hooks/api'
import { computeDisciplineStats, computeProjectProgress } from '@/lib/stats'
import type { Task } from '@/types/generated'
import { useDisciplines } from './useDisciplines'

export function useDisciplineStats(queryDomain: InvokeQueryDomain = 'app') {
  const { disciplines, error: disciplinesError, isLoading: disciplinesLoading } = useDisciplines(queryDomain)
  const {
    data: tasks = [],
    isLoading: tasksLoading,
    error: tasksError
  } = useInvoke<Task[]>('get_tasks', undefined, {
    queryDomain
  })

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
