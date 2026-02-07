import { useInvoke } from '@/hooks/useInvoke'
import type { FeatureData as Feature, GroupStats, ProjectProgress } from '@/types/generated'

export function useFeatureStats() {
  const { data: features = [], isLoading: featuresLoading, error: featuresError } = useInvoke<Feature[]>('get_features')
  const { data: featureStats = [], isLoading: statsLoading } = useInvoke<GroupStats[]>('get_feature_stats')
  const { data: progress } = useInvoke<ProjectProgress>('get_project_progress')

  const statsMap = new Map<string, GroupStats>()
  for (const stat of featureStats) {
    statsMap.set(stat.name, stat)
  }

  return {
    features,
    statsMap,
    progress: {
      total: progress?.totalTasks ?? 0,
      done: progress?.doneTasks ?? 0,
      percent: progress?.progressPercent ?? 0
    },
    isLoading: featuresLoading || statsLoading,
    error: featuresError ? String(featuresError) : null
  }
}
