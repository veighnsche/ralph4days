import { useMemo } from 'react'
import { useInvoke } from '@/hooks/useInvoke'
import { computeFeatureStats, computeProjectProgress } from '@/lib/stats'
import type { FeatureData as Feature, Task } from '@/types/generated'

export function useFeatureStats() {
  const { data: features = [], isLoading: featuresLoading, error: featuresError } = useInvoke<Feature[]>('get_features')
  const { data: tasks = [], isLoading: tasksLoading } = useInvoke<Task[]>('get_tasks')

  const statsMap = useMemo(() => computeFeatureStats(tasks, features), [tasks, features])
  const progress = useMemo(() => computeProjectProgress(tasks), [tasks])

  return {
    features,
    statsMap,
    progress: {
      total: progress.totalTasks,
      done: progress.doneTasks,
      percent: progress.progressPercent
    },
    isLoading: featuresLoading || tasksLoading,
    error: featuresError ? String(featuresError) : null
  }
}
