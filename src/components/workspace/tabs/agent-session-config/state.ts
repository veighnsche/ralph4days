import type { Agent, AgentSessionLaunchConfig, Effort } from '@/hooks/preferences'
import type { TerminalBridgeModelOption } from '@/types/generated'
import type { AgentSessionConfigTabParams } from './schema'

type LaunchPreferenceDefaults = Pick<AgentSessionLaunchConfig, 'effort' | 'thinking' | 'permissionLevel'>

export function asEffort(value: string | undefined): Effort | null {
  if (value === 'low' || value === 'medium' || value === 'high') return value
  return null
}

export function resolveFallbackEffort(effortOptions: string[]): Effort | null {
  const medium = asEffort('medium')
  if (medium && effortOptions.includes(medium)) return medium
  return asEffort(effortOptions[0])
}

export function buildInitialLaunchConfig(
  initialParams: AgentSessionConfigTabParams,
  getDefaultModel: (agent: Agent) => string,
  defaults: LaunchPreferenceDefaults
): AgentSessionLaunchConfig {
  const agent = initialParams.agent ?? 'claude'
  return {
    agent,
    model: initialParams.model ?? getDefaultModel(agent),
    effort: initialParams.effort ?? defaults.effort,
    thinking: initialParams.thinking ?? defaults.thinking,
    permissionLevel: initialParams.permissionLevel ?? defaults.permissionLevel
  }
}

export function groupModelsByAgent(
  providers: Array<{ agent: string; models: TerminalBridgeModelOption[] }>
): Record<string, TerminalBridgeModelOption[]> {
  const grouped: Record<string, TerminalBridgeModelOption[]> = {}
  for (const provider of providers) {
    grouped[provider.agent] = provider.models
  }
  return grouped
}
