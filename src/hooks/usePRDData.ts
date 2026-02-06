import type { EnrichedTask } from "@/types/prd";
import { useInvoke } from "./useInvoke";

export function usePRDData() {
  const { data, isLoading, error, refetch } = useInvoke<EnrichedTask[]>("get_enriched_tasks", undefined, {
    staleTime: 0,
  });

  return {
    tasks: data ?? null,
    isLoading,
    error: error ? `Failed to load tasks: ${error.message}` : null,
    refetch,
  };
}
