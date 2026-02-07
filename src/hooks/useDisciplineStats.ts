import { useDisciplines } from '@/hooks/useDisciplines'
import { useInvoke } from '@/hooks/useInvoke'
import type { GroupStats, ProjectProgress } from '@/types/prd'

export function useDisciplineStats() {
  const { disciplines } = useDisciplines()
  const { data: disciplineStats = [], isLoading: statsLoading } = useInvoke<GroupStats[]>('get_discipline_stats')
  const { data: progress } = useInvoke<ProjectProgress>('get_project_progress')

  const statsMap = new Map<string, GroupStats>()
  for (const stat of disciplineStats) {
    statsMap.set(stat.name, stat)
  }

  return {
    disciplines,
    statsMap,
    progress: {
      total: progress?.totalTasks ?? 0,
      done: progress?.doneTasks ?? 0,
      percent: progress?.progressPercent ?? 0
    },
    isLoading: statsLoading || disciplines.length === 0
  }
}
