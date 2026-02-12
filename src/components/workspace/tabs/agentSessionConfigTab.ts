import { AgentSessionConfigTabContent } from '@/components/workspace/AgentSessionConfigTabContent'
import type { Agent, Effort } from '@/hooks/preferences'
import { terminalBridgeListModelFormTree } from '@/lib/terminal/terminalBridgeClient'
import { useWorkspaceStore, type WorkspaceTab } from '@/stores/useWorkspaceStore'
import type { TerminalBridgeModelOption } from '@/types/generated'

export type AgentSessionConfigTabInput = {
  agent: Agent
  model: string
  effort: Effort
  thinking: boolean
}

async function loadFormTrees(tabId: string) {
  const { setTabData } = useWorkspaceStore.getState()
  setTabData(tabId, {
    formTreeLoading: true,
    formTreeError: null
  })
  const formTreeByAgent: Record<string, TerminalBridgeModelOption[]> = {}
  let firstError: string | null = null
  try {
    const result = await terminalBridgeListModelFormTree()
    for (const provider of result.providers) {
      formTreeByAgent[provider.agent] = provider.models
    }
  } catch (error) {
    firstError = `Failed to load model form tree: ${String(error)}`
  }

  setTabData(tabId, {
    formTreeByAgent,
    formTreeLoading: false,
    formTreeError: firstError
  })
}

export function createAgentSessionConfigTab(input: AgentSessionConfigTabInput): Omit<WorkspaceTab, 'id'> {
  return {
    type: 'agent-session-config',
    component: AgentSessionConfigTabContent,
    title: 'Start Agent Session',
    closeable: true,
    lifecycle: {
      onMount: tab => {
        void loadFormTrees(tab.id)
      },
      onUnmount: () => {},
      onActivate: () => {},
      onDeactivate: () => {}
    },
    data: {
      agent: input.agent,
      model: input.model,
      effort: input.effort,
      thinking: input.thinking,
      formTreeByAgent: {},
      formTreeLoading: true,
      formTreeError: null
    }
  }
}
