import { defineWorkspaceTabModule } from '../contracts'
import { AgentSessionConfigTabContent } from './content'
import { createAgentSessionConfigTab } from './factory'
import { parseAgentSessionConfigTabParams } from './schema'

export const agentSessionConfigTabModule = defineWorkspaceTabModule({
  type: 'agent-session-config',
  component: AgentSessionConfigTabContent,
  parseParams: parseAgentSessionConfigTabParams,
  createTab: createAgentSessionConfigTab
})
