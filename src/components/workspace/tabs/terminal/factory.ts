import type { AgentSessionLaunchConfig } from '@/components/agent-session-launch'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import { TERMINAL_AGENT_LABELS, TERMINAL_TAB_DEFAULT_TITLE } from './constants'
import type { TerminalTabInput } from './schema'

function agentLabel(agent: string | undefined): string {
  return agent === 'codex' ? TERMINAL_AGENT_LABELS.codex : TERMINAL_AGENT_LABELS.claude
}

export function createTerminalTab(input: TerminalTabInput = {}): Omit<WorkspaceTab, 'id'> {
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

export function createDefaultTerminalTab(title = TERMINAL_TAB_DEFAULT_TITLE): Omit<WorkspaceTab, 'id'> {
  return createTerminalTab({ title })
}

export function createTerminalTabFromTask(taskId: number, title?: string): Omit<WorkspaceTab, 'id'> {
  return createTerminalTab({
    taskId,
    title: title ?? `Task #${taskId.toString().padStart(3, '0')}`
  })
}

export function createTerminalTabFromLaunch(
  launch: AgentSessionLaunchConfig,
  options: { title?: string; initPrompt?: string } = {}
): Omit<WorkspaceTab, 'id'> {
  return createTerminalTab({
    ...launch,
    title: options.title,
    initPrompt: options.initPrompt
  })
}
