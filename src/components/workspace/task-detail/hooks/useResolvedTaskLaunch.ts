import { useWorkspaceInvoke } from '@/hooks/api'
import { useAgentSessionLaunchPreferences } from '@/hooks/preferences'
import type {
  Task,
  TerminalBridgeLaunchDefaults,
  TerminalBridgeLaunchSource,
  TerminalBridgeResolvedLaunchConfig
} from '@/types/generated'

export type LaunchSource = TerminalBridgeLaunchSource
type ResolvedAgent = 'claude' | 'codex' | 'shell'
type ResolvedEffort = 'low' | 'medium' | 'high'

function parseResolvedAgent(value: TerminalBridgeResolvedLaunchConfig['agent']): ResolvedAgent | undefined {
  if (value === 'claude' || value === 'codex' || value === 'shell') return value
  if (value == null) return undefined
  throw new Error(`ralph invariant violated: invalid resolved agent '${value}'`)
}

function parseResolvedEffort(value: TerminalBridgeResolvedLaunchConfig['effort']): ResolvedEffort | undefined {
  if (value === 'low' || value === 'medium' || value === 'high') return value
  if (value == null) return undefined
  throw new Error(`ralph invariant violated: invalid resolved effort '${value}'`)
}

const FALLBACK_SOURCES: Pick<
  TerminalBridgeResolvedLaunchConfig,
  'agentSource' | 'modelSource' | 'effortSource' | 'thinkingSource' | 'permissionLevelSource'
> = {
  agentSource: 'unset',
  modelSource: 'unset',
  effortSource: 'unset',
  thinkingSource: 'unset',
  permissionLevelSource: 'unset'
}

export function useResolvedTaskLaunch(task: Task): {
  resolvedAgent: ResolvedAgent | undefined
  resolvedModel: string | undefined
  resolvedEffort: ResolvedEffort | undefined
  resolvedThinking: boolean | undefined
  resolvedModelSupportsEffort: boolean
  agentSource: LaunchSource
  modelSource: LaunchSource
  effortSource: LaunchSource
  thinkingSource: LaunchSource
  resolveError: Error | null
  isLoading: boolean
} {
  const defaultAgent = useAgentSessionLaunchPreferences(state => state.agent)
  const defaultModel = useAgentSessionLaunchPreferences(state => state.model)
  const defaultEffort = useAgentSessionLaunchPreferences(state => state.effort)
  const defaultThinking = useAgentSessionLaunchPreferences(state => state.thinking)
  const defaultPermissionLevel = useAgentSessionLaunchPreferences(state => state.permissionLevel)

  const defaults: TerminalBridgeLaunchDefaults = {
    agent: defaultAgent,
    model: defaultModel,
    effort: defaultEffort,
    thinking: defaultThinking,
    permissionLevel: defaultPermissionLevel
  }

  const { data, error, isLoading } = useWorkspaceInvoke<TerminalBridgeResolvedLaunchConfig>(
    'terminal_resolve_task_launch_config',
    { taskId: task.id, defaults },
    {
      staleTime: 15_000
    }
  )

  return {
    resolvedAgent: parseResolvedAgent(data?.agent),
    resolvedModel: data?.model ?? undefined,
    resolvedEffort: data?.modelSupportsEffort ? parseResolvedEffort(data.effort) : undefined,
    resolvedThinking: data?.thinking ?? undefined,
    resolvedModelSupportsEffort: data?.modelSupportsEffort ?? false,
    agentSource: data?.agentSource ?? FALLBACK_SOURCES.agentSource,
    modelSource: data?.modelSource ?? FALLBACK_SOURCES.modelSource,
    effortSource: data?.effortSource ?? FALLBACK_SOURCES.effortSource,
    thinkingSource: data?.thinkingSource ?? FALLBACK_SOURCES.thinkingSource,
    resolveError: error,
    isLoading
  }
}
