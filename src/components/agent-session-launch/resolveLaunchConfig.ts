import type { AgentSessionLaunchConfig, Effort } from '@/lib/agent-session-launch-config'
import { terminalBridgeListModelFormTree } from '@/lib/terminal/terminalBridgeClient'

function asEffort(value: string): Effort | null {
  if (value === 'low' || value === 'medium' || value === 'high') return value
  return null
}

function resolveFallbackEffort(effortOptions: string[]): Effort | null {
  const medium = asEffort('medium')
  if (medium && effortOptions.includes(medium)) return medium
  return effortOptions[0] ? asEffort(effortOptions[0]) : null
}

export async function resolveLaunchConfigAgainstCatalog(
  config: AgentSessionLaunchConfig
): Promise<AgentSessionLaunchConfig> {
  const tree = await terminalBridgeListModelFormTree()
  const provider = tree.providers.find(p => p.agent === config.agent)
  const models = provider?.models ?? []
  if (models.length === 0) return config

  const selectedModel = models.find(model => model.name === config.model) ?? models[0]
  if (!selectedModel) return config

  const effortOptions = selectedModel.effortOptions ?? []
  const effortValid = effortOptions.length === 0 || effortOptions.includes(config.effort)
  const nextEffort = effortValid ? config.effort : (resolveFallbackEffort(effortOptions) ?? config.effort)

  return {
    ...config,
    model: selectedModel.name,
    effort: nextEffort
  }
}
