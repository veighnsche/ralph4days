import { useDisciplines } from '@/hooks/disciplines'
import { useAgentSessionLaunchPreferences } from '@/hooks/preferences'
import type { Agent, Effort } from '@/lib/agent-session-launch-config'
import type { Task } from '@/types/generated'
import { useModelFormTreeByAgent } from '../../tabs/agent-session-config/hooks/useModelFormTreeByAgent'

export type LaunchSource = 'task' | 'discipline' | 'default' | 'unset'
type LaunchSourceWithDefault = Exclude<LaunchSource, 'unset'>

type ResolvedValueWithSource<T> = {
  value: T | undefined
  source: LaunchSourceWithDefault
}

function asAgent(value?: string): Agent | undefined {
  return value === 'claude' || value === 'codex' ? value : undefined
}

function asEffort(value?: string): Effort | undefined {
  return value === 'low' || value === 'medium' || value === 'high' ? value : undefined
}

function resolveWithTaskDisciplineDefault<T>(
  taskValue: T | undefined,
  disciplineValue: T | undefined,
  fallback: T | undefined
): ResolvedValueWithSource<T> {
  if (taskValue !== undefined) return { value: taskValue, source: 'task' as const }
  if (disciplineValue !== undefined) return { value: disciplineValue, source: 'discipline' as const }
  return { value: fallback, source: 'default' as const }
}

export function useResolvedTaskLaunch(task: Task): {
  resolvedAgent: Agent | undefined
  resolvedModel: string | undefined
  resolvedEffort: Effort | undefined
  resolvedThinking: boolean | undefined
  resolvedModelSupportsEffort: boolean
  agentSource: LaunchSource
  modelSource: LaunchSource
  effortSource: LaunchSource
  thinkingSource: LaunchSource
} {
  const { disciplines } = useDisciplines()
  const prefAgent = useAgentSessionLaunchPreferences(state => state.agent)
  const prefModel = useAgentSessionLaunchPreferences(state => state.model)
  const prefEffort = useAgentSessionLaunchPreferences(state => state.effort)
  const prefThinking = useAgentSessionLaunchPreferences(state => state.thinking)
  const resolveDefaultModel = useAgentSessionLaunchPreferences(state => state.getDefaultModel)
  const { formTreeByAgent } = useModelFormTreeByAgent()

  const disciplineConfig = disciplines.find(discipline => discipline.name === task.discipline)
  const taskAgent = asAgent(task.agent)
  const disciplineAgent = asAgent(disciplineConfig?.agent)
  const resolvedAgent = resolveWithTaskDisciplineDefault(taskAgent, disciplineAgent, prefAgent)
  const resolvedModelFallback =
    resolvedAgent.value == null
      ? undefined
      : resolvedAgent.value === prefAgent
        ? prefModel
        : resolveDefaultModel(resolvedAgent.value)
  const resolvedModel = resolveWithTaskDisciplineDefault(task.model, disciplineConfig?.model, resolvedModelFallback)

  const modelsForResolvedAgent = resolvedModel.value == null ? [] : (formTreeByAgent[resolvedModel.value] ?? [])
  const resolvedModelOption =
    resolvedModel.value == null ? undefined : modelsForResolvedAgent.find(model => model.name === resolvedModel.value)
  const resolvedModelSupportsEffort = (resolvedModelOption?.effortOptions?.length ?? 0) > 0

  const resolvedEffort = resolveWithTaskDisciplineDefault(
    asEffort(task.effort),
    asEffort(disciplineConfig?.effort),
    prefEffort
  )
  const resolvedThinking = resolveWithTaskDisciplineDefault(task.thinking, disciplineConfig?.thinking, prefThinking)

  const agentSource =
    resolvedAgent.source === 'task' ? 'task' : resolvedAgent.source === 'discipline' ? 'discipline' : 'default'
  const modelSource = resolvedModel.source
  const effortSource = resolvedEffort.source
  const thinkingSource = resolvedThinking.source

  return {
    resolvedAgent: resolvedAgent.value,
    resolvedModel: resolvedModel.value,
    resolvedEffort: resolvedModelSupportsEffort ? resolvedEffort.value : undefined,
    resolvedThinking: resolvedThinking.value,
    resolvedModelSupportsEffort,
    agentSource,
    modelSource,
    effortSource,
    thinkingSource
  }
}
