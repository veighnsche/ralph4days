import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import { useModelConstraints } from './useModelConstraints'
import { useModelFormTreeByAgent } from './useModelFormTreeByAgent'
import { useRunSessionAction } from './useRunSessionAction'
import { useSyncLaunchPreferences } from './useSyncLaunchPreferences'

export function useAgentSessionConfigController(tab: WorkspaceTab) {
  useSyncLaunchPreferences()
  const { formTreeByAgent, formTreeLoading, formTreeError } = useModelFormTreeByAgent()
  const { models, loadingModels, error, selectedModel } = useModelConstraints({
    formTreeByAgent,
    formTreeLoading,
    formTreeError
  })
  const runSession = useRunSessionAction(tab, selectedModel?.display)

  return {
    models,
    loadingModels,
    error,
    runSession
  }
}
