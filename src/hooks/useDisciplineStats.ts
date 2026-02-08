import { useMemo } from 'react'
import { useDisciplines } from '@/hooks/useDisciplines'
import { useInvoke } from '@/hooks/useInvoke'
import { computeDisciplineStats, computeProjectProgress } from '@/lib/stats'
import type { Task } from '@/types/generated'

export function useDisciplineStats() {
  const { disciplines, error: disciplinesError } = useDisciplines()
  const { data: tasks = [], isLoading: tasksLoading } = useInvoke<Task[]>('get_tasks')

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
    isLoading: disciplines.length === 0 || tasksLoading,
    error: disciplinesError ? String(disciplinesError) : null
  }
}
