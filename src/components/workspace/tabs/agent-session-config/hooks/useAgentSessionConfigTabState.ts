import { agentSessionConfigSelectors } from '../selectors'
import { useAgentSessionConfigStore } from '../store'

export function useAgentSessionConfigLaunchState() {
  const agent = useAgentSessionConfigStore(agentSessionConfigSelectors.agent)
  const model = useAgentSessionConfigStore(agentSessionConfigSelectors.model)
  const effort = useAgentSessionConfigStore(agentSessionConfigSelectors.effort)
  const thinking = useAgentSessionConfigStore(agentSessionConfigSelectors.thinking)
  const permissionLevel = useAgentSessionConfigStore(agentSessionConfigSelectors.permissionLevel)
  return { agent, model, effort, thinking, permissionLevel }
}

export function useAgentSessionConfigActions() {
  const setAgent = useAgentSessionConfigStore(agentSessionConfigSelectors.setAgent)
  const setModel = useAgentSessionConfigStore(agentSessionConfigSelectors.setModel)
  const setEffort = useAgentSessionConfigStore(agentSessionConfigSelectors.setEffort)
  const setThinking = useAgentSessionConfigStore(agentSessionConfigSelectors.setThinking)
  const setPermissionLevel = useAgentSessionConfigStore(agentSessionConfigSelectors.setPermissionLevel)
  return {
    setAgent,
    setModel,
    setEffort,
    setThinking,
    setPermissionLevel
  }
}
