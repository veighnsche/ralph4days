import { type InvokeQueryDomain, useInvoke } from '@/hooks/api'
import type { StackMetadataData } from '@/types/generated'

const EMPTY_STACKS: StackMetadataData[] = []

export function useStackMetadata(queryDomain: InvokeQueryDomain = 'app') {
  const { data, error, isLoading } = useInvoke<StackMetadataData[]>('stacks_metadata_list', undefined, {
    queryDomain,
    staleTime: 5 * 60 * 1000
  })

  return {
    stacks: data ?? EMPTY_STACKS,
    error: error ? String(error) : null,
    isLoading
  }
}
