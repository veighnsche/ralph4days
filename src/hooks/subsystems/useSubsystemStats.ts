import { useMemo } from 'react'
import { type InvokeQueryDomain, useInvoke } from '@/hooks/api'
import { computeProjectProgress, computeSubsystemStats } from '@/lib/stats'
import type { SubsystemData, TaskListItem } from '@/types/generated'

export function useSubsystemStats(queryDomain: InvokeQueryDomain = 'workspace') {
  const {
    data: subsystems = [],
    isLoading: subsystemsLoading,
    error: subsystemsError
  } = useInvoke<SubsystemData[]>('get_subsystems', undefined, { queryDomain })
  const { data: tasks = [], isLoading: tasksLoading } = useInvoke<TaskListItem[]>('get_task_list_items', undefined, {
    queryDomain
  })

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
