export const VALID_AGENTS = ['claude', 'codex'] as const

export type Agent = (typeof VALID_AGENTS)[number]
export type Model = string
export type Effort = 'low' | 'medium' | 'high'
export type PermissionLevel = 'safe' | 'balanced' | 'auto' | 'full_auto'

export type AgentSessionLaunchConfig = {
  agent: Agent
  model: Model
  effort: Effort
  thinking: boolean
  permissionLevel: PermissionLevel
}

export const DEFAULT_MODELS_BY_AGENT: Record<Agent, string> = {
  claude: 'claude-sonnet-4',
  codex: 'gpt-5-codex'
}

export function getDefaultModel(agent: Agent): Model {
  return DEFAULT_MODELS_BY_AGENT[agent]
}
