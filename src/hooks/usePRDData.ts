import yaml from "js-yaml";
import { validatePRDData } from "@/lib/validation";
import type { PRDData } from "@/types/prd";
import { useInvoke } from "./useInvoke";

export function usePRDData() {
  const { data, isLoading, error, refetch } = useInvoke<string, PRDData>("get_prd_content", undefined, {
    staleTime: 0,
    select: (content) => {
      const parsed = yaml.load(content);
      return validatePRDData(parsed);
    },
  });

  return {
    prdData: data ?? null,
    isLoading,
    error: error ? `Failed to load PRD: ${error.message}` : null,
    refetch,
  };
}
