import { type QueryClient, type UseMutationOptions, useMutation, useQueryClient } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { buildInvalidateQueryKey, type InvokeQueryDomain } from './useInvoke'

type MutationCacheUpdater<TArgs, TResult> = (params: {
  queryClient: QueryClient
  data: TResult
  variables: TArgs
  queryDomain: InvokeQueryDomain
}) => void | Promise<void>

type MutationRollback = () => void | Promise<void>

type MutationOptimisticUpdater<TArgs> = (params: {
  queryClient: QueryClient
  variables: TArgs
  queryDomain: InvokeQueryDomain
}) => MutationRollback | undefined | Promise<MutationRollback | undefined>

interface InternalMutationContext<TUserOnMutateResult> {
  rollback?: MutationRollback
  userOnMutateResult: TUserOnMutateResult
}

// WHY: Mirror of useInvoke for writes; allows either invalidation or explicit cache patching.
export function useInvokeMutation<TArgs = void, TResult = void, TUserOnMutateResult = unknown>(
  command: string,
  options?: {
    invalidateKeys?: readonly unknown[][]
    queryDomain?: InvokeQueryDomain
    updateCache?: MutationCacheUpdater<TArgs, TResult>
    optimisticUpdate?: MutationOptimisticUpdater<TArgs>
  } & Omit<UseMutationOptions<TResult, Error, TArgs, TUserOnMutateResult>, 'mutationFn'>
) {
  const queryClient = useQueryClient()
  const {
    invalidateKeys,
    queryDomain = 'app',
    updateCache,
    optimisticUpdate,
    onMutate: userOnMutate,
    onError: userOnError,
    onSuccess: userOnSuccess,
    onSettled: userOnSettled,
    ...restOptions
  } = options ?? {}

  return useMutation<TResult, Error, TArgs, InternalMutationContext<TUserOnMutateResult>>({
    mutationFn: async (args: TArgs) => {
      try {
        return await invoke<TResult>(command, args as Record<string, unknown>)
      } catch (err) {
        throw err instanceof Error ? err : new Error(String(err))
      }
    },
    ...restOptions,
    onMutate: async (variables, context) => {
      const userOnMutateResult = (await userOnMutate?.(variables, context)) as TUserOnMutateResult
      const rollback = optimisticUpdate
        ? await optimisticUpdate({
            queryClient,
            variables,
            queryDomain
          })
        : undefined

      return {
        rollback,
        userOnMutateResult
      }
    },
    onError: async (error, variables, onMutateResult, context) => {
      if (onMutateResult?.rollback) {
        await onMutateResult.rollback()
      }

      userOnError?.(error, variables, onMutateResult?.userOnMutateResult as TUserOnMutateResult, context)
    },
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
      userOnSuccess?.(data, variables, onMutateResult?.userOnMutateResult as TUserOnMutateResult, context)
    },
    onSettled: async (data, error, variables, onMutateResult, context) => {
      userOnSettled?.(data, error, variables, onMutateResult?.userOnMutateResult as TUserOnMutateResult, context)
    }
  })
}
