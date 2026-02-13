import { useMemo } from 'react'
import { useInvoke } from '@/hooks/api'
import { computeDisciplineStats, computeProjectProgress } from '@/lib/stats'
import type { Task } from '@/types/generated'
import { useDisciplines } from './useDisciplines'

export function useDisciplineStats() {
  const { disciplines, error: disciplinesError, isLoading: disciplinesLoading } = useDisciplines()
  const { data: tasks = [], isLoading: tasksLoading, error: tasksError } = useInvoke<Task[]>('get_tasks')

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
