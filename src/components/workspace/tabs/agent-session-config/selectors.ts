import type { AgentSessionConfigStoreState } from './store'

export const agentSessionConfigSelectors = {
  agent: (state: AgentSessionConfigStoreState) => state.agent,
  model: (state: AgentSessionConfigStoreState) => state.model,
  effort: (state: AgentSessionConfigStoreState) => state.effort,
  thinking: (state: AgentSessionConfigStoreState) => state.thinking,
  permissionLevel: (state: AgentSessionConfigStoreState) => state.permissionLevel,
  setAgent: (state: AgentSessionConfigStoreState) => state.setAgent,
  setModel: (state: AgentSessionConfigStoreState) => state.setModel,
  setEffort: (state: AgentSessionConfigStoreState) => state.setEffort,
  setThinking: (state: AgentSessionConfigStoreState) => state.setThinking,
  setPermissionLevel: (state: AgentSessionConfigStoreState) => state.setPermissionLevel
} as const
