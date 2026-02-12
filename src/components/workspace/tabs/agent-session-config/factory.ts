import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import type { AgentSessionConfigTabParams } from './schema'

export function createAgentSessionConfigTab(input: AgentSessionConfigTabParams): Omit<WorkspaceTab, 'id'> {
  return {
    type: 'agent-session-config',
    title: 'Start Agent Session',
    closeable: true,
    params: {
      agent: input.agent,
      model: input.model,
      effort: input.effort,
      thinking: input.thinking,
      permissionLevel: input.permissionLevel
    } satisfies AgentSessionConfigTabParams
  }
}
