import { type UseQueryOptions, useQuery } from '@tanstack/react-query'
import { tauriInvoke } from '@/lib/tauri/invoke'

const isTauri = typeof window !== 'undefined' && '__TAURI__' in window

export type InvokeQueryDomain = 'app' | 'workspace'

type LegacyQueryKey = readonly unknown[]

interface UseInvokeOptions<TResult, TSelected>
  extends Omit<UseQueryOptions<TResult, Error, TSelected>, 'queryKey' | 'queryFn'> {
  queryDomain?: InvokeQueryDomain
}

export function buildInvokeQueryKey(
  command: string,
  args?: Record<string, unknown>,
  queryDomain: InvokeQueryDomain = 'app'
) {
  return args ? ([queryDomain, command, args] as const) : ([queryDomain, command] as const)
}

export function buildInvalidateQueryKey(key: LegacyQueryKey, queryDomain: InvokeQueryDomain = 'app') {
  if (key[0] === 'app' || key[0] === 'workspace') {
    return key
  }

  if (typeof key[0] !== 'string') {
    return [queryDomain, ...key]
  }

  const args =
    key[1] && typeof key[1] === 'object' && !Array.isArray(key[1]) ? (key[1] as Record<string, unknown>) : undefined

  return key.length > 1
    ? buildInvokeQueryKey(key[0], args, queryDomain)
    : buildInvokeQueryKey(key[0], undefined, queryDomain)
}

// WHY: Query key includes args for automatic deduplication of identical requests
export function useInvoke<TResult, TSelected = TResult>(
  command: string,
  args?: Record<string, unknown>,
  options?: UseInvokeOptions<TResult, TSelected>
) {
  const { enabled: callerEnabled, queryDomain = 'app', ...rest } = options ?? {}
  return useQuery<TResult, Error, TSelected>({
    queryKey: buildInvokeQueryKey(command, args, queryDomain),
    queryFn: () => (args ? tauriInvoke<TResult>(command, args) : tauriInvoke<TResult>(command)),
    enabled: isTauri && (callerEnabled ?? true),
    ...rest
  })
}

export function useWorkspaceInvoke<TResult, TSelected = TResult>(
  command: string,
  args?: Record<string, unknown>,
  options?: Omit<UseInvokeOptions<TResult, TSelected>, 'queryDomain'>
) {
  return useInvoke<TResult, TSelected>(command, args, {
    ...options,
    queryDomain: 'workspace'
  })
}
