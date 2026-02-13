import { useMemo } from 'react'
import { useInvoke } from '@/hooks/api'
import { computeProjectProgress, computeSubsystemStats } from '@/lib/stats'
import type { SubsystemData, Task } from '@/types/generated'

export function useSubsystemStats() {
  const {
    data: subsystems = [],
    isLoading: subsystemsLoading,
    error: subsystemsError
  } = useInvoke<SubsystemData[]>('get_subsystems')
  const { data: tasks = [], isLoading: tasksLoading } = useInvoke<Task[]>('get_tasks')

  const statsMap = useMemo(() => computeSubsystemStats(tasks, subsystems), [tasks, subsystems])
  const progress = useMemo(() => computeProjectProgress(tasks), [tasks])

  return {
    subsystems,
    statsMap,
    progress: {
      total: progress.totalTasks,
      done: progress.doneTasks,
      percent: progress.progressPercent
    },
    isLoading: subsystemsLoading || tasksLoading,
    error: subsystemsError ? String(subsystemsError) : null
  }
}
