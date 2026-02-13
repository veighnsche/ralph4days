import { type UseMutationOptions, useMutation, useQueryClient } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { buildInvalidateQueryKey, type InvokeQueryDomain } from './useInvoke'

// WHY: Mirror of useInvoke for writes; auto-invalidates cache after mutations
export function useInvokeMutation<TArgs = void, TResult = void>(
  command: string,
  options?: {
    invalidateKeys?: readonly unknown[][]
    queryDomain?: InvokeQueryDomain
  } & Omit<UseMutationOptions<TResult, Error, TArgs>, 'mutationFn'>
) {
  const queryClient = useQueryClient()
  const { invalidateKeys, queryDomain = 'app', onSuccess: userOnSuccess, ...restOptions } = options ?? {}

  return useMutation<TResult, Error, TArgs>({
    mutationFn: async (args: TArgs) => {
      try {
        return await invoke<TResult>(command, args as Record<string, unknown>)
      } catch (err) {
        throw err instanceof Error ? err : new Error(String(err))
      }
    },
    ...restOptions,
    onSuccess: async (...args) => {
      if (invalidateKeys) {
        await Promise.all(
          invalidateKeys.map(key => {
            const canonicalKey = buildInvalidateQueryKey(key, queryDomain)
            return queryClient.invalidateQueries({ queryKey: canonicalKey })
          })
        )
      }
      userOnSuccess?.(...args)
    }
  })
}
