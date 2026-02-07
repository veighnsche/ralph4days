import { type UseQueryOptions, useQuery } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'

const isTauri = typeof window !== 'undefined' && '__TAURI__' in window

// WHY: Query key includes args for automatic deduplication of identical requests
export function useInvoke<TResult, TSelected = TResult>(
  command: string,
  args?: Record<string, unknown>,
  options?: Omit<UseQueryOptions<TResult, Error, TSelected>, 'queryKey' | 'queryFn'>
) {
  return useQuery<TResult, Error, TSelected>({
    queryKey: args ? [command, args] : [command],
    queryFn: () => invoke<TResult>(command, args),
    enabled: isTauri,
    ...options
  })
}
