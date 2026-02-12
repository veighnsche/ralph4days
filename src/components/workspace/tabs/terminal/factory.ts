import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import type { TerminalTabParams } from './schema'

function agentLabel(agent: string | undefined): string {
  return agent === 'codex' ? 'Codex' : 'Claude'
}

export function createTerminalTab(input: TerminalTabParams = {}): Omit<WorkspaceTab, 'id'> {
  const title = input.title ?? `${agentLabel(input.agent)} (${input.model ?? 'default'})`
  return {
    type: 'terminal',
    title,
    closeable: true,
    params: {
      agent: input.agent,
      model: input.model,
      effort: input.effort,
      thinking: input.thinking,
      permissionLevel: input.permissionLevel,
      taskId: input.taskId,
      initPrompt: input.initPrompt
    }
  }
}
