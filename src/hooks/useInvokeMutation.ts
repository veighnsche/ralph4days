import { type UseMutationOptions, useMutation, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

/**
 * Thin wrapper around TanStack Query useMutation + Tauri invoke.
 * Mirrors useInvoke for reads — this handles writes with automatic cache invalidation.
 */
export function useInvokeMutation<TArgs = void, TResult = void>(
  command: string,
  options?: {
    invalidateKeys?: string[][];
  } & Omit<UseMutationOptions<TResult, Error, TArgs>, "mutationFn">
) {
  const queryClient = useQueryClient();
  const { invalidateKeys, onSuccess: userOnSuccess, ...restOptions } = options ?? {};

  return useMutation<TResult, Error, TArgs>({
    mutationFn: async (args: TArgs) => {
      try {
        return await invoke<TResult>(command, args as Record<string, unknown>);
      } catch (err) {
        throw err instanceof Error ? err : new Error(String(err));
      }
    },
    ...restOptions,
    onSuccess: async (...args) => {
      if (invalidateKeys) {
        await Promise.all(invalidateKeys.map((key) => queryClient.invalidateQueries({ queryKey: key })));
      }
      userOnSuccess?.(...args);
    },
  });
}
