import { useEffect } from 'react'
import type { TerminalBridgeModelOption } from '@/types/generated'
import { resolveFallbackEffort } from '../state'
import { useAgentSessionConfigActions, useAgentSessionConfigLaunchState } from './useAgentSessionConfigTabState'

export function useModelConstraints({
  formTreeByAgent,
  formTreeLoading,
  formTreeError
}: {
  formTreeByAgent: Record<string, TerminalBridgeModelOption[]>
  formTreeLoading: boolean
  formTreeError: string | null
}) {
  const { agent, model, effort } = useAgentSessionConfigLaunchState()
  const { setModel, setEffort } = useAgentSessionConfigActions()
  const models = formTreeByAgent[agent] ?? []
  const loadingModels = formTreeLoading && models.length === 0
  const error = formTreeError

  const selectedModel = models.find(nextModel => nextModel.name === model) ?? null

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

  return { models, loadingModels, error, selectedModel }
}
