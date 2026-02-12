import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import { useModelConstraints } from './useModelConstraints'
import { useModelFormTreeByAgent } from './useModelFormTreeByAgent'
import { useRunSessionAction } from './useRunSessionAction'
import { useSyncLaunchPreferences } from './useSyncLaunchPreferences'

export function useAgentSessionConfigController(tab: WorkspaceTab) {
  useSyncLaunchPreferences()
  const { formTreeByAgent, formTreeLoading, formTreeError } = useModelFormTreeByAgent()
  const { models, loadingModels, error, selectedModel, selectedModelEffortValid } = useModelConstraints({
    formTreeByAgent,
    formTreeLoading,
    formTreeError
  })
  const canRun = !!selectedModel && selectedModelEffortValid
  const runSession = useRunSessionAction(tab, { selectedModelDisplay: selectedModel?.display, canRun })

  return {
    models,
    loadingModels,
    error,
    canRun,
    runSession
  }
}
