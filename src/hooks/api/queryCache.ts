import type { QueryClient } from '@tanstack/react-query'
import { buildInvokeQueryKey, type InvokeQueryDomain } from './useInvoke'

interface QueryCacheParams<TData> {
  queryClient: QueryClient
  queryDomain: InvokeQueryDomain
  command: string
  args: Record<string, unknown>
  data: TData
}

export function replaceQueryDataInCache<TData>({
  queryClient,
  queryDomain,
  command,
  args,
  data
}: QueryCacheParams<TData>): void {
  const queryKey = buildInvokeQueryKey(command, args, queryDomain)
  const current = queryClient.getQueryData<TData>(queryKey)
  if (current === undefined) {
    throw new Error(`[query-cache] ${command} cache is missing for ${queryDomain} domain`)
  }
  queryClient.setQueryData<TData>(queryKey, data)
}

export function replaceQueryDataInCacheOptimistically<TData>({
  queryClient,
  queryDomain,
  command,
  args,
  data
}: QueryCacheParams<TData>): () => void {
  const queryKey = buildInvokeQueryKey(command, args, queryDomain)
  const current = queryClient.getQueryData<TData>(queryKey)
  if (current === undefined) {
    throw new Error(`[query-cache] ${command} cache is missing for ${queryDomain} domain`)
  }

  queryClient.setQueryData<TData>(queryKey, data)

  return () => {
    queryClient.setQueryData<TData>(queryKey, current)
  }
}
