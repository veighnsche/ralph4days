import { useEffect, useMemo } from 'react'
import { useAgentSessionLaunchPreferences } from '@/hooks/preferences'
import { useWorkspaceStore, type WorkspaceTab } from '@/stores/useWorkspaceStore'
import { createTerminalTabFromLaunch } from '../../terminal/factory'
import { AGENT_PROVIDER_META } from '../constants'
import { resolveFallbackEffort } from '../state'
import { useAgentSessionConfigStore } from '../store'
import { useModelFormTreeByAgent } from './useModelFormTreeByAgent'

export function useAgentSessionConfigController(tab: WorkspaceTab) {
  const closeTab = useWorkspaceStore(s => s.closeTab)
  const openTab = useWorkspaceStore(s => s.openTab)

  const saveAgent = useAgentSessionLaunchPreferences(s => s.setAgent)
  const saveModel = useAgentSessionLaunchPreferences(s => s.setModel)
  const saveEffort = useAgentSessionLaunchPreferences(s => s.setEffort)
  const saveThinking = useAgentSessionLaunchPreferences(s => s.setThinking)
  const savePermissionLevel = useAgentSessionLaunchPreferences(s => s.setPermissionLevel)

  const { formTreeByAgent, formTreeLoading, formTreeError } = useModelFormTreeByAgent()

  const agent = useAgentSessionConfigStore(state => state.agent)
  const model = useAgentSessionConfigStore(state => state.model)
  const effort = useAgentSessionConfigStore(state => state.effort)
  const thinking = useAgentSessionConfigStore(state => state.thinking)
  const permissionLevel = useAgentSessionConfigStore(state => state.permissionLevel)
  const setModel = useAgentSessionConfigStore(state => state.setModel)
  const setEffort = useAgentSessionConfigStore(state => state.setEffort)
  const setModels = useAgentSessionConfigStore(state => state.setModels)
  const setLoadingModels = useAgentSessionConfigStore(state => state.setLoadingModels)
  const setError = useAgentSessionConfigStore(state => state.setError)

  const models = useAgentSessionConfigStore(state => state.models)
  const loadingModels = useAgentSessionConfigStore(state => state.loadingModels)
  const error = useAgentSessionConfigStore(state => state.error)
  const selectedModel = useMemo(() => models.find(nextModel => nextModel.name === model) ?? null, [models, model])

  useEffect(() => {
    saveAgent(agent)
  }, [agent, saveAgent])

  useEffect(() => {
    saveModel(model)
  }, [model, saveModel])

  useEffect(() => {
    saveEffort(effort)
  }, [effort, saveEffort])

  useEffect(() => {
    saveThinking(thinking)
  }, [saveThinking, thinking])

  useEffect(() => {
    savePermissionLevel(permissionLevel)
  }, [permissionLevel, savePermissionLevel])

  useEffect(() => {
    const nextModels = formTreeByAgent[agent] ?? []
    setModels(nextModels)
    setLoadingModels(formTreeLoading && nextModels.length === 0)
    setError(formTreeError)
  }, [agent, formTreeByAgent, formTreeError, formTreeLoading, setError, setLoadingModels, setModels])

  useEffect(() => {
    if (models.length === 0 || selectedModel) return
    const firstModel = models[0]?.name
    if (!firstModel) return
    setModel(firstModel)
  }, [models, selectedModel, setModel])

  useEffect(() => {
    if (!selectedModel) return
    const effortOptions = selectedModel.effortOptions ?? []
    if (effortOptions.length === 0 || effortOptions.includes(effort)) return
    const fallbackEffort = resolveFallbackEffort(effortOptions)
    if (!fallbackEffort) return
    setEffort(fallbackEffort)
  }, [effort, selectedModel, setEffort])

  const runSession = () => {
    const selectedModelDisplay = selectedModel?.display ?? model
    openTab(
      createTerminalTabFromLaunch(
        {
          agent,
          model,
          effort,
          thinking,
          permissionLevel
        },
        {
          title: `${AGENT_PROVIDER_META[agent].label} (${selectedModelDisplay})`
        }
      )
    )
    closeTab(tab.id)
  }

  return {
    loadingModels,
    error,
    runSession
  }
}
