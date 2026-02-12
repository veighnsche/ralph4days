import { Bot, Brain } from 'lucide-react'
import { Fragment, useEffect, useMemo, useState } from 'react'
import { InlineError } from '@/components/shared'
import { Button } from '@/components/ui/button'
import { Field, FieldLabel } from '@/components/ui/field'
import { FormDescription, FormHeader, FormTitle } from '@/components/ui/form-header'
import { ScrollArea } from '@/components/ui/scroll-area'
import { SelectableCard } from '@/components/ui/selectable-card'
import { Separator } from '@/components/ui/separator'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import {
  type Agent,
  type AgentSessionLaunchConfig,
  type Effort,
  type PermissionLevel,
  useAgentSessionLaunchPreferences
} from '@/hooks/preferences'
import { useTabMeta } from '@/hooks/workspace'
import { terminalBridgeListModelFormTree } from '@/lib/terminal/terminalBridgeClient'
import { useWorkspaceStore, type WorkspaceTab } from '@/stores/useWorkspaceStore'
import type { TerminalBridgeModelOption } from '@/types/generated'
import { createTerminalTab } from './TerminalTabContent'

const AGENT_OPTIONS: Agent[] = ['claude', 'codex']

const AGENT_PROVIDER_META = {
  claude: {
    label: 'Claude',
    logoSrc: '/reference-logos/anthropic.svg',
    logoAlt: 'Anthropic logo'
  },
  codex: {
    label: 'Codex',
    logoSrc: '/reference-logos/openai.svg',
    logoAlt: 'OpenAI logo'
  }
} as const

const PERMISSION_LEVEL_OPTIONS: Array<{ value: PermissionLevel; label: string }> = [
  { value: 'safe', label: 'Safe' },
  { value: 'balanced', label: 'Balanced' },
  { value: 'auto', label: 'Auto' },
  { value: 'full_auto', label: 'Full Auto' }
]

function asEffort(value: string | undefined): Effort | null {
  if (value === 'low' || value === 'medium' || value === 'high') return value
  return null
}

function resolveFallbackEffort(effortOptions: string[]): Effort | null {
  const medium = asEffort('medium')
  if (medium && effortOptions.includes(medium)) return medium
  return asEffort(effortOptions[0])
}

function AgentProviderCard({
  agentOption,
  selected,
  onSelect
}: {
  agentOption: Agent
  selected: boolean
  onSelect: () => void
}) {
  return (
    <SelectableCard
      selected={selected}
      radius="lg"
      role="radio"
      aria-checked={selected}
      onClick={onSelect}
      className="aspect-[5/7] w-40 shrink-0 overflow-hidden px-3 py-2 text-left text-sm font-medium">
      <span className="absolute -left-10 -top-10 inline-flex min-w-28 items-center justify-center">
        {agentOption === 'codex' ? (
          <span
            aria-hidden="true"
            className="h-48 w-48 bg-current text-foreground opacity-95"
            style={{
              WebkitMaskImage: 'url(/reference-logos/openai.svg)',
              maskImage: 'url(/reference-logos/openai.svg)',
              WebkitMaskRepeat: 'no-repeat',
              maskRepeat: 'no-repeat',
              WebkitMaskPosition: 'center',
              maskPosition: 'center',
              WebkitMaskSize: 'contain',
              maskSize: 'contain'
            }}
          />
        ) : (
          <img
            src={AGENT_PROVIDER_META[agentOption].logoSrc}
            alt={AGENT_PROVIDER_META[agentOption].logoAlt}
            className="h-48 w-auto opacity-95"
          />
        )}
      </span>
      <span className="absolute bottom-3 left-1/2 -translate-x-1/2 text-center">
        {AGENT_PROVIDER_META[agentOption].label}
      </span>
    </SelectableCard>
  )
}

function AgentProviderPicker({ agent, onSelect }: { agent: Agent; onSelect: (value: Agent) => void }) {
  return (
    <Field className="gap-0">
      <FieldLabel>Agent Provider</FieldLabel>
      <div className="flex items-center justify-start gap-3" role="radiogroup" aria-label="Agent provider">
        {AGENT_OPTIONS.map(agentOption => (
          <AgentProviderCard
            key={agentOption}
            agentOption={agentOption}
            selected={agent === agentOption}
            onSelect={() => onSelect(agentOption)}
          />
        ))}
      </div>
    </Field>
  )
}

function ModelCard({
  modelOption,
  selected,
  loading,
  onSelect
}: {
  modelOption: TerminalBridgeModelOption
  selected: boolean
  loading: boolean
  onSelect: () => void
}) {
  return (
    <SelectableCard
      selected={selected}
      role="radio"
      aria-checked={selected}
      onClick={onSelect}
      disabled={loading}
      className="flex w-full flex-col items-start gap-0.5 px-3 py-2 text-sm font-medium">
      <span>{modelOption.display || modelOption.name}</span>
      <span className="text-xs font-normal text-muted-foreground">{modelOption.description}</span>
      {modelOption.sessionModel && modelOption.sessionModel !== modelOption.name && (
        <span className="text-[11px] font-normal text-muted-foreground/90">Runs as: {modelOption.sessionModel}</span>
      )}
    </SelectableCard>
  )
}

function ModelEffortRow({
  modelOption,
  selected,
  effort,
  onModelSelect,
  onEffortSelect
}: {
  modelOption: TerminalBridgeModelOption
  selected: boolean
  effort: Effort
  onModelSelect: (value: string) => void
  onEffortSelect: (value: Effort) => void
}) {
  const effortOptions = modelOption.effortOptions ?? []
  if (effortOptions.length === 0) return null

  return (
    <ToggleGroup
      type="single"
      value={selected ? effort : ''}
      onValueChange={value => {
        if (value === '') return
        if (!selected) onModelSelect(modelOption.name)
        const nextEffort = asEffort(value)
        if (nextEffort) onEffortSelect(nextEffort)
      }}
      variant="outline"
      className="w-full"
      aria-label="Effort">
      {effortOptions.map(effortOption => (
        <ToggleGroupItem key={effortOption} value={effortOption} className="flex-1 capitalize">
          {effortOption}
        </ToggleGroupItem>
      ))}
    </ToggleGroup>
  )
}

function ModelPicker({
  models,
  model,
  effort,
  loadingModels,
  thinking,
  onModelSelect,
  onEffortSelect,
  onThinkingChange
}: {
  models: TerminalBridgeModelOption[]
  model: string
  effort: Effort
  loadingModels: boolean
  thinking: boolean
  onModelSelect: (value: string) => void
  onEffortSelect: (value: Effort) => void
  onThinkingChange: (value: boolean) => void
}) {
  return (
    <div className="flex flex-col gap-1" role="radiogroup" aria-label="Model">
      <ModelSectionHeader thinking={thinking} onThinkingChange={onThinkingChange} />
      {models.map(modelOption => {
        const selected = model === modelOption.name
        return (
          <Fragment key={modelOption.name}>
            <ModelCard
              modelOption={modelOption}
              selected={selected}
              loading={loadingModels}
              onSelect={() => onModelSelect(modelOption.name)}
            />
            <ModelEffortRow
              modelOption={modelOption}
              selected={selected}
              effort={effort}
              onModelSelect={onModelSelect}
              onEffortSelect={onEffortSelect}
            />
          </Fragment>
        )
      })}
    </div>
  )
}

function ModelSectionHeader({
  thinking,
  onThinkingChange
}: {
  thinking: boolean
  onThinkingChange: (value: boolean) => void
}) {
  return (
    <div className="flex items-center justify-between">
      <div className="text-sm font-semibold leading-none">Model</div>
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <SelectableCard
              selected={thinking}
              variant="icon"
              role="switch"
              aria-checked={thinking}
              aria-label="Extended Thinking"
              onClick={() => onThinkingChange(!thinking)}
              className="shrink-0 text-muted-foreground data-[selected=true]:text-foreground">
              <Brain className="h-4 w-4" aria-hidden="true" />
            </SelectableCard>
          </TooltipTrigger>
          <TooltipContent side="top">Extended Thinking</TooltipContent>
        </Tooltip>
      </TooltipProvider>
    </div>
  )
}

function PermissionLevelControls({
  permissionLevel,
  onSelect
}: {
  permissionLevel: PermissionLevel
  onSelect: (value: PermissionLevel) => void
}) {
  return (
    <ToggleGroup
      type="single"
      value={permissionLevel}
      onValueChange={value => {
        if (value === '') return
        onSelect(value as PermissionLevel)
      }}
      variant="outline"
      aria-label="Permission Level">
      {PERMISSION_LEVEL_OPTIONS.map(option => (
        <ToggleGroupItem key={option.value} value={option.value}>
          {option.label}
        </ToggleGroupItem>
      ))}
    </ToggleGroup>
  )
}

export type AgentSessionConfigTabParams = Partial<AgentSessionLaunchConfig>

function parseAgentConfigField(value: unknown): Agent {
  if (value !== 'claude' && value !== 'codex') {
    throw new Error('Invalid agent session config tab params.agent')
  }
  return value
}

function parseModelConfigField(value: unknown): string {
  if (typeof value !== 'string' || value.trim() === '') {
    throw new Error('Invalid agent session config tab params.model')
  }
  return value
}

function parseEffortConfigField(value: unknown): Effort {
  if (value !== 'low' && value !== 'medium' && value !== 'high') {
    throw new Error('Invalid agent session config tab params.effort')
  }
  return value
}

function parseThinkingConfigField(value: unknown): boolean {
  if (typeof value !== 'boolean') {
    throw new Error('Invalid agent session config tab params.thinking')
  }
  return value
}

function parsePermissionLevelConfigField(value: unknown): PermissionLevel {
  if (value !== 'safe' && value !== 'balanced' && value !== 'auto' && value !== 'full_auto') {
    throw new Error('Invalid agent session config tab params.permissionLevel')
  }
  return value
}

function parseAgentSessionConfigTabParams(params: unknown): AgentSessionConfigTabParams {
  if (params == null) return {}
  if (typeof params !== 'object' || Array.isArray(params)) {
    throw new Error('Invalid agent session config tab params: expected object')
  }
  const candidate = params as Record<string, unknown>
  const parsed: AgentSessionConfigTabParams = {}
  if (candidate.agent !== undefined) parsed.agent = parseAgentConfigField(candidate.agent)
  if (candidate.model !== undefined) parsed.model = parseModelConfigField(candidate.model)
  if (candidate.effort !== undefined) parsed.effort = parseEffortConfigField(candidate.effort)
  if (candidate.thinking !== undefined) parsed.thinking = parseThinkingConfigField(candidate.thinking)
  if (candidate.permissionLevel !== undefined) {
    parsed.permissionLevel = parsePermissionLevelConfigField(candidate.permissionLevel)
  }
  return parsed
}

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

function useAgentSessionConfigController(tab: WorkspaceTab) {
  const closeTab = useWorkspaceStore(s => s.closeTab)
  const openTab = useWorkspaceStore(s => s.openTab)
  const initialParams = parseAgentSessionConfigTabParams(tab.params)

  const {
    setAgent: saveAgent,
    setModel: saveModel,
    setEffort: saveEffort,
    setThinking: saveThinking,
    setPermissionLevel: savePermissionLevel,
    getDefaultModel,
    effort: preferredEffort,
    thinking: preferredThinking,
    permissionLevel: preferredPermissionLevel
  } = useAgentSessionLaunchPreferences()

  const [formTreeByAgent, setFormTreeByAgent] = useState<Record<string, TerminalBridgeModelOption[]>>({})
  const [formTreeLoading, setFormTreeLoading] = useState(true)
  const [formTreeError, setFormTreeError] = useState<string | null>(null)

  const [launchConfig, setLaunchConfig] = useState<AgentSessionLaunchConfig>(() => {
    const initialAgent = initialParams.agent ?? 'claude'
    return {
      agent: initialAgent,
      model: initialParams.model ?? getDefaultModel(initialAgent),
      effort: initialParams.effort ?? preferredEffort,
      thinking: initialParams.thinking ?? preferredThinking,
      permissionLevel: initialParams.permissionLevel ?? preferredPermissionLevel
    }
  })
  const { agent, model, effort, thinking, permissionLevel } = launchConfig

  useEffect(() => {
    let cancelled = false
    const load = async () => {
      setFormTreeLoading(true)
      setFormTreeError(null)
      try {
        const result = await terminalBridgeListModelFormTree()
        if (cancelled) return
        const nextByAgent: Record<string, TerminalBridgeModelOption[]> = {}
        for (const provider of result.providers) {
          nextByAgent[provider.agent] = provider.models
        }
        setFormTreeByAgent(nextByAgent)
        setFormTreeLoading(false)
      } catch (error) {
        if (cancelled) return
        setFormTreeByAgent({})
        setFormTreeLoading(false)
        setFormTreeError(`Failed to load model form tree: ${String(error)}`)
      }
    }
    void load()
    return () => {
      cancelled = true
    }
  }, [])

  const models = formTreeByAgent[agent] ?? []
  const loadingModels = formTreeLoading && models.length === 0
  const error = formTreeError

  const selectedModel = useMemo(() => models.find(nextModel => nextModel.name === model) ?? null, [models, model])

  const selectAgent = (value: Agent) => {
    setLaunchConfig(current => ({ ...current, agent: value }))
    saveAgent(value)
  }

  const selectModel = (value: string) => {
    setLaunchConfig(current => ({ ...current, model: value }))
    saveModel(value)
  }

  const selectEffort = (value: Effort) => {
    setLaunchConfig(current => ({ ...current, effort: value }))
    saveEffort(value)
  }

  const updateThinking = (value: boolean) => {
    setLaunchConfig(current => ({ ...current, thinking: value }))
    saveThinking(value)
  }

  const selectPermissionLevel = (value: PermissionLevel) => {
    setLaunchConfig(current => ({ ...current, permissionLevel: value }))
    savePermissionLevel(value)
  }

  useEffect(() => {
    if (models.length === 0 || selectedModel) return
    const firstModel = models[0]?.name
    if (!firstModel) return
    setLaunchConfig(current => ({ ...current, model: firstModel }))
    saveModel(firstModel)
  }, [models, saveModel, selectedModel])

  useEffect(() => {
    if (!selectedModel) return
    const effortOptions = selectedModel.effortOptions ?? []
    if (effortOptions.length === 0 || effortOptions.includes(effort)) return
    const fallbackEffort = resolveFallbackEffort(effortOptions)
    if (!fallbackEffort) return
    setLaunchConfig(current => ({ ...current, effort: fallbackEffort }))
    saveEffort(fallbackEffort)
  }, [effort, saveEffort, selectedModel])

  const runSession = () => {
    const selectedModelDisplay = selectedModel?.display || model
    openTab(
      createTerminalTab({
        agent,
        model,
        effort,
        thinking,
        permissionLevel,
        title: `${AGENT_PROVIDER_META[agent].label} (${selectedModelDisplay})`
      })
    )
    closeTab(tab.id)
  }

  return {
    agent,
    model,
    effort,
    thinking,
    permissionLevel,
    models,
    loadingModels,
    error,
    selectAgent,
    selectModel,
    selectEffort,
    updateThinking,
    selectPermissionLevel,
    runSession
  }
}

export function AgentSessionConfigTabContent({ tab }: { tab: WorkspaceTab }) {
  useTabMeta(tab.id, 'Start Agent Session', Bot)

  const {
    agent,
    model,
    effort,
    thinking,
    permissionLevel,
    models,
    loadingModels,
    error,
    selectAgent,
    selectModel,
    selectEffort,
    updateThinking,
    selectPermissionLevel,
    runSession
  } = useAgentSessionConfigController(tab)

  return (
    <div className="h-full flex flex-col">
      <div className="px-4">
        <FormHeader>
          <FormTitle>Start Agent Session</FormTitle>
          <FormDescription>Configure launch options, then start an agent session.</FormDescription>
        </FormHeader>
      </div>
      <Separator />
      <ScrollArea className="flex-1 min-h-0">
        <div className="px-4 py-4 space-y-4">
          <AgentProviderPicker agent={agent} onSelect={selectAgent} />
          <ModelPicker
            models={models}
            model={model}
            effort={effort}
            loadingModels={loadingModels}
            thinking={thinking}
            onModelSelect={selectModel}
            onEffortSelect={selectEffort}
            onThinkingChange={updateThinking}
          />
        </div>
      </ScrollArea>
      <Separator />
      {error && (
        <div className="px-3 pt-1.5">
          <InlineError error={error} />
        </div>
      )}
      <div className="px-3 py-1.5 flex justify-end gap-2">
        <PermissionLevelControls permissionLevel={permissionLevel} onSelect={selectPermissionLevel} />
        <Button type="button" onClick={runSession} disabled={loadingModels || !model || models.length === 0}>
          Run
        </Button>
      </div>
    </div>
  )
}
