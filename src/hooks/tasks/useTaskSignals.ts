import { QUERY_KEYS } from '@/constants/cache'
import { useInvoke, useInvokeMutation } from '@/hooks/api'
import type { TaskSignal } from '@/types/generated'

export function useTaskSignals(taskId: number) {
  const { data, isLoading } = useInvoke<TaskSignal[]>('get_task_signals', { taskId })

  const answerMutation = useInvokeMutation<{ signalId: number; answer: string }>('answer_ask', {
    invalidateKeys: QUERY_KEYS.SIGNALS
  })

  return {
    signals: data ?? [],
    isLoading,
    answerAsk: (signalId: number, answer: string) => answerMutation.mutate({ signalId, answer }),
    isAnswering: answerMutation.isPending
  }
}
