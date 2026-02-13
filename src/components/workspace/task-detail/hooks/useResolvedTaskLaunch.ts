import { useMemo } from 'react'
import { useDisciplines } from '@/hooks/disciplines'
import { useAgentSessionLaunchPreferences } from '@/hooks/preferences'
import type { Agent, Effort } from '@/lib/agent-session-launch-config'
import type { Task } from '@/types/generated'
import { useModelFormTreeByAgent } from '../../tabs/agent-session-config/hooks/useModelFormTreeByAgent'

export type LaunchSource = 'task' | 'discipline' | 'default' | 'unset'

function asAgent(value?: string): Agent | undefined {
  return value === 'claude' || value === 'codex' ? value : undefined
}

function asEffort(value?: string): Effort | undefined {
  return value === 'low' || value === 'medium' || value === 'high' ? value : undefined
}

export function useResolvedTaskLaunch(task: Task) {
  const { disciplines } = useDisciplines()
  const prefAgent = useAgentSessionLaunchPreferences(state => state.agent)
  const prefModel = useAgentSessionLaunchPreferences(state => state.model)
  const prefEffort = useAgentSessionLaunchPreferences(state => state.effort)
  const prefThinking = useAgentSessionLaunchPreferences(state => state.thinking)
  const resolveDefaultModel = useAgentSessionLaunchPreferences(state => state.getDefaultModel)
  const { formTreeByAgent } = useModelFormTreeByAgent()

  const disciplineConfig = useMemo(
    () => disciplines.find(d => d.name === task.discipline),
    [disciplines, task.discipline]
  )

  const taskAgent = asAgent(task.agent)
  const disciplineAgent = asAgent(disciplineConfig?.agent)
  const agentSource: LaunchSource = taskAgent ? 'task' : disciplineAgent ? 'discipline' : 'default'
  const resolvedAgent = taskAgent ?? disciplineAgent ?? prefAgent

  const modelSource: LaunchSource =
    task.model != null ? 'task' : disciplineConfig?.model != null ? 'discipline' : 'default'
  const resolvedModel =
    task.model ??
    disciplineConfig?.model ??
    (resolvedAgent === prefAgent ? prefModel : resolveDefaultModel(resolvedAgent))

  const modelsForResolvedAgent = formTreeByAgent[resolvedAgent] ?? []
  const resolvedModelOption = modelsForResolvedAgent.find(model => model.name === resolvedModel)
  const resolvedModelSupportsEffort = (resolvedModelOption?.effortOptions?.length ?? 0) > 0

  const effortSource: LaunchSource = task.effort ? 'task' : disciplineConfig?.effort ? 'discipline' : 'default'
  const resolvedEffortRaw = asEffort(task.effort) ?? asEffort(disciplineConfig?.effort) ?? prefEffort
  const resolvedEffort = resolvedModelSupportsEffort ? resolvedEffortRaw : undefined

  const thinkingSource: LaunchSource =
    task.thinking !== undefined ? 'task' : disciplineConfig?.thinking !== undefined ? 'discipline' : 'default'
  const resolvedThinking = task.thinking ?? disciplineConfig?.thinking ?? prefThinking

  return {
    resolvedAgent,
    resolvedModel,
    resolvedEffort,
    resolvedThinking,
    resolvedModelSupportsEffort,
    agentSource,
    modelSource,
    effortSource,
    thinkingSource
  }
}
