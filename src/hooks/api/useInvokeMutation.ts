import { type QueryClient, type UseMutationOptions, useMutation, useQueryClient } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { buildInvalidateQueryKey, type InvokeQueryDomain } from './useInvoke'

type MutationCacheUpdater<TArgs, TResult> = (params: {
  queryClient: QueryClient
  data: TResult
  variables: TArgs
  queryDomain: InvokeQueryDomain
}) => void | Promise<void>

// WHY: Mirror of useInvoke for writes; allows either invalidation or explicit cache patching.
export function useInvokeMutation<TArgs = void, TResult = void>(
  command: string,
  options?: {
    invalidateKeys?: readonly unknown[][]
    queryDomain?: InvokeQueryDomain
    updateCache?: MutationCacheUpdater<TArgs, TResult>
  } & Omit<UseMutationOptions<TResult, Error, TArgs>, 'mutationFn'>
) {
  const queryClient = useQueryClient()
  const { invalidateKeys, queryDomain = 'app', updateCache, onSuccess: userOnSuccess, ...restOptions } = options ?? {}

  return useMutation<TResult, Error, TArgs>({
    mutationFn: async (args: TArgs) => {
      try {
        return await invoke<TResult>(command, args as Record<string, unknown>)
      } catch (err) {
        throw err instanceof Error ? err : new Error(String(err))
      }
    },
    ...restOptions,
    onSuccess: async (data, variables, onMutateResult, context) => {
      if (invalidateKeys) {
        await Promise.all(
          invalidateKeys.map(key => {
            const canonicalKey = buildInvalidateQueryKey(key, queryDomain)
            return queryClient.invalidateQueries({ queryKey: canonicalKey })
          })
        )
      }
      if (updateCache) {
        await updateCache({
          queryClient,
          data,
          variables,
          queryDomain
        })
      }
      userOnSuccess?.(data, variables, onMutateResult, context)
    }
  })
}
