import { useInvoke } from '@/hooks/api'
import type { TaskSignalSummary } from '@/types/generated'

export function useSignalSummaries(taskIds: number[]) {
  const { data, isLoading } = useInvoke<Record<number, TaskSignalSummary>>(
    'tasks_signal_summaries_get',
    { taskIds },
    { enabled: taskIds.length > 0 }
  )

  return {
    summaries: data ?? {},
    isLoading
  }
}
