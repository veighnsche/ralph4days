import { useQuery } from '@tanstack/react-query'
import { terminalBridgeListModelFormTree } from '@/lib/terminal/terminalBridgeClient'
import type { TerminalBridgeModelOption } from '@/types/generated'
import { MODEL_FORM_TREE_QUERY_KEY } from '../constants'
import { groupModelsByAgent } from '../state'

export function useModelFormTreeByAgent() {
  const query = useQuery({
    queryKey: MODEL_FORM_TREE_QUERY_KEY,
    queryFn: terminalBridgeListModelFormTree,
    staleTime: 0,
    gcTime: 5 * 60 * 1000,
    retry: false,
    refetchOnWindowFocus: false,
    select: result => groupModelsByAgent(result.providers)
  })

  const formTreeByAgent: Record<string, TerminalBridgeModelOption[]> = query.data ?? {}
  const formTreeLoading = query.isPending
  const formTreeError = query.error ? `Failed to load model form tree: ${String(query.error)}` : null

  return { formTreeByAgent, formTreeLoading, formTreeError }
}
