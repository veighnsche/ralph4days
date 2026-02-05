import { type UseQueryOptions, useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

const isTauri = typeof window !== "undefined" && "__TAURI__" in window;

/**
 * Thin wrapper around TanStack Query + Tauri invoke.
 * Query key = [command, args] for automatic deduplication.
 */
export function useInvoke<TResult, TSelected = TResult>(
  command: string,
  args?: Record<string, unknown>,
  options?: Omit<UseQueryOptions<TResult, Error, TSelected>, "queryKey" | "queryFn">
) {
  return useQuery<TResult, Error, TSelected>({
    queryKey: args ? [command, args] : [command],
    queryFn: () => invoke<TResult>(command, args),
    enabled: isTauri,
    ...options,
  });
}
