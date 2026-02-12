import { AgentSessionConfigTabContent } from '@/components/workspace/AgentSessionConfigTabContent'
import type { Agent, Effort } from '@/hooks/preferences'
import { terminalBridgeListModels } from '@/lib/terminal/terminalBridgeClient'
import { useWorkspaceStore, type WorkspaceTab } from '@/stores/useWorkspaceStore'

export type AgentSessionConfigTabInput = {
  agent: Agent
  model: string
  effort: Effort
  thinking: boolean
}

async function loadFormTree(tabId: string, agent: Agent) {
  const { setTabData } = useWorkspaceStore.getState()
  setTabData(tabId, {
    agentSessionFormTreeLoading: true,
    agentSessionFormTreeError: null
  })
  try {
    const result = await terminalBridgeListModels(agent)
    setTabData(tabId, {
      agentSessionFormTree: {
        agent: result.agent,
        models: result.models
      },
      agentSessionFormTreeLoading: false,
      agentSessionFormTreeError: null
    })
  } catch (error) {
    setTabData(tabId, {
      agentSessionFormTree: undefined,
      agentSessionFormTreeLoading: false,
      agentSessionFormTreeError: `Failed to load model list: ${String(error)}`
    })
  }
}

export function refreshAgentSessionConfigFormTree(tabId: string, agent: Agent) {
  void loadFormTree(tabId, agent)
}

export function createAgentSessionConfigTab(input: AgentSessionConfigTabInput): Omit<WorkspaceTab, 'id'> {
  return {
    type: 'agent-session-config',
    component: AgentSessionConfigTabContent,
    title: 'Start Agent Session',
    closeable: true,
    lifecycle: {
      onMount: tab => {
        const tabAgent = (tab.data?.agent as Agent | undefined) ?? input.agent
        refreshAgentSessionConfigFormTree(tab.id, tabAgent)
      },
      onUnmount: () => {},
      onActivate: () => {},
      onDeactivate: () => {}
    },
    data: {
      agent: input.agent,
      model: input.model,
      effort: input.agent === 'claude' ? input.effort : undefined,
      thinking: input.thinking,
      agentSessionFormTreeLoading: true,
      agentSessionFormTreeError: null
    }
  }
}
