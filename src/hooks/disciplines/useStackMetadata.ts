import { type InvokeQueryDomain, useInvoke } from '@/hooks/api'
import type { StackMetadataData } from '@/types/generated'

export function useStackMetadata(queryDomain: InvokeQueryDomain = 'app') {
  const { data, error, isLoading } = useInvoke<StackMetadataData[]>('get_stack_metadata', undefined, {
    queryDomain,
    staleTime: 5 * 60 * 1000
  })

  return {
    stacks: data ?? [],
    error: error ? String(error) : null,
    isLoading
  }
}
