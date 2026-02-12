import { Bot } from 'lucide-react'
import { Fragment, useEffect, useState } from 'react'
import { InlineError } from '@/components/shared'
import { Button } from '@/components/ui/button'
import { Field, FieldDescription, FieldLabel } from '@/components/ui/field'
import { FormDescription, FormHeader, FormTitle } from '@/components/ui/form-header'
import { ScrollArea } from '@/components/ui/scroll-area'
import { SelectableCard } from '@/components/ui/selectable-card'
import { Separator } from '@/components/ui/separator'
import { Switch } from '@/components/ui/switch'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { type Agent, type Effort, type PermissionLevel, useAgentSessionLaunchPreferences } from '@/hooks/preferences'
import { useTabMeta } from '@/hooks/workspace'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'
import { TerminalTabContent } from './TerminalTabContent'

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

export function AgentSessionConfigTabContent({ tab }: { tab: WorkspaceTab }) {
  useTabMeta(tab.id, 'Start Agent Session', Bot)
  const closeTab = useWorkspaceStore(s => s.closeTab)
  const openTab = useWorkspaceStore(s => s.openTab)
  const {
    setAgent: saveAgent,
    setModel: saveModel,
    setEffort: saveEffort,
    setThinking: saveThinking,
    setPermissionLevel: savePermissionLevel,
    getDefaultModel,
    effort: preferredEffort,
    permissionLevel: preferredPermissionLevel
  } = useAgentSessionLaunchPreferences()

  const [agent, setAgent] = useState<Agent>((tab.data?.agent as Agent | undefined) ?? 'claude')
  const [model, setModel] = useState(tab.data?.model ?? getDefaultModel(agent))
  const [effort, setEffort] = useState<Effort>((tab.data?.effort as Effort | undefined) ?? preferredEffort)
  const [thinking, setThinking] = useState(tab.data?.thinking ?? false)
  const [permissionLevel, setPermissionLevel] = useState<PermissionLevel>(
    (tab.data?.permissionLevel as PermissionLevel | undefined) ?? preferredPermissionLevel
  )
  const models = tab.data?.formTreeByAgent?.[agent] ?? []
  const loadingModels = (tab.data?.formTreeLoading ?? false) && models.length === 0
  const error = tab.data?.formTreeError ?? null

  const handleAgentSelect = (value: Agent) => {
    setAgent(value)
    saveAgent(value)
  }

  const handleModelSelect = (value: string) => {
    setModel(value)
    saveModel(value)
  }

  const handleThinkingChange = (value: boolean) => {
    setThinking(value)
    saveThinking(value)
  }
  const handleEffortSelect = (value: Effort) => {
    setEffort(value)
    saveEffort(value)
  }
  const handlePermissionLevelSelect = (value: PermissionLevel) => {
    setPermissionLevel(value)
    savePermissionLevel(value)
  }

  useEffect(() => {
    if (models.length === 0) return

    if (!models.some(nextModel => nextModel.name === model)) {
      const firstModel = models[0]?.name
      if (firstModel != null) {
        setModel(firstModel)
        saveModel(firstModel)
      }
      return
    }

    const selectedModel = models.find(nextModel => nextModel.name === model) ?? models[0]
    const effortOptions = selectedModel?.effortOptions ?? []
    if (effortOptions.length > 0 && !effortOptions.includes(effort)) {
      const fallbackEffort = effortOptions.includes('medium') ? 'medium' : effortOptions[0]
      if (fallbackEffort === 'low' || fallbackEffort === 'medium' || fallbackEffort === 'high') {
        setEffort(fallbackEffort)
        saveEffort(fallbackEffort)
      }
    }
  }, [effort, model, models, saveEffort, saveModel])

  const handleRun = () => {
    const agentLabel = agent === 'codex' ? 'Codex' : 'Claude'
    const selectedModelDisplay = models.find(nextModel => nextModel.name === model)?.display || model
    openTab({
      type: 'terminal',
      component: TerminalTabContent,
      title: `${agentLabel} (${selectedModelDisplay})`,
      closeable: true,
      data: { agent, model, effort, thinking, permissionLevel }
    })
    closeTab(tab.id)
  }

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
          <Field>
            <FieldLabel>Agent Provider</FieldLabel>
            <div className="flex items-center justify-start gap-3" role="radiogroup" aria-label="Agent provider">
              {(['claude', 'codex'] as const).map(agentOption => (
                <SelectableCard
                  key={agentOption}
                  selected={agent === agentOption}
                  radius="lg"
                  role="radio"
                  aria-checked={agent === agentOption}
                  onClick={() => handleAgentSelect(agentOption)}
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
              ))}
            </div>
          </Field>

          <Field>
            <FieldLabel>Model</FieldLabel>
            <div className="grid grid-cols-2 gap-3">
              <div className="flex flex-col gap-1" role="radiogroup" aria-label="Model">
                {models.map(modelOption => {
                  const isSelected = model === modelOption.name
                  const effortOptions = modelOption.effortOptions ?? []
                  const showInlineEffort = effortOptions.length > 0

                  return (
                    <Fragment key={modelOption.name}>
                      <SelectableCard
                        selected={isSelected}
                        role="radio"
                        aria-checked={isSelected}
                        onClick={() => handleModelSelect(modelOption.name)}
                        disabled={loadingModels}
                        className="flex w-full flex-col items-start gap-0.5 px-3 py-2 text-sm font-medium">
                        <span>{modelOption.display || modelOption.name}</span>
                        <span className="text-xs font-normal text-muted-foreground">{modelOption.description}</span>
                        {modelOption.sessionModel && modelOption.sessionModel !== modelOption.name && (
                          <span className="text-[11px] font-normal text-muted-foreground/90">
                            Runs as: {modelOption.sessionModel}
                          </span>
                        )}
                      </SelectableCard>
                      {showInlineEffort && (
                        <ToggleGroup
                          type="single"
                          value={isSelected ? effort : ''}
                          onValueChange={value => {
                            if (value === '') return
                            if (!isSelected) handleModelSelect(modelOption.name)
                            handleEffortSelect(value as Effort)
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
                      )}
                    </Fragment>
                  )
                })}
              </div>
              <div>
                <Field orientation="horizontal">
                  <div className="flex-1">
                    <FieldLabel>Extended Thinking</FieldLabel>
                    <FieldDescription>Enable deeper planning when supported by the provider.</FieldDescription>
                  </div>
                  <Switch checked={thinking} onCheckedChange={handleThinkingChange} />
                </Field>
              </div>
            </div>
          </Field>
        </div>
      </ScrollArea>
      <Separator />
      {error && (
        <div className="px-3 pt-1.5">
          <InlineError error={error} />
        </div>
      )}
      <div className="px-3 py-1.5 flex justify-end gap-2">
        <ToggleGroup
          type="single"
          value={permissionLevel}
          onValueChange={value => {
            if (value === '') return
            handlePermissionLevelSelect(value as PermissionLevel)
          }}
          variant="outline"
          aria-label="Permission Level">
          <ToggleGroupItem value="safe">Safe</ToggleGroupItem>
          <ToggleGroupItem value="balanced">Balanced</ToggleGroupItem>
          <ToggleGroupItem value="auto">Auto</ToggleGroupItem>
          <ToggleGroupItem value="full_auto">Full Auto</ToggleGroupItem>
        </ToggleGroup>
        <Button type="button" onClick={handleRun} disabled={loadingModels || !model || models.length === 0}>
          Run
        </Button>
      </div>
    </div>
  )
}
