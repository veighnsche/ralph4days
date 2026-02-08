import { useInvoke } from '@/hooks/api'
import type { StackMetadataData } from '@/types/generated'

export function useStackMetadata() {
  const { data, error, isLoading } = useInvoke<StackMetadataData[]>('get_stack_metadata', undefined, {
    staleTime: 5 * 60 * 1000
  })

  return {
    stacks: data ?? [],
    error: error ? String(error) : null,
    isLoading
  }
}
