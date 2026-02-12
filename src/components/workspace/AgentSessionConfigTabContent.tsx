import { Bot } from 'lucide-react'
import { useEffect, useState } from 'react'
import { InlineError } from '@/components/shared'
import { Button } from '@/components/ui/button'
import { Field, FieldDescription, FieldLabel } from '@/components/ui/field'
import { FormDescription, FormHeader, FormTitle } from '@/components/ui/form-header'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { Switch } from '@/components/ui/switch'
import { type Agent, type Effort, useAgentSessionLaunchPreferences } from '@/hooks/preferences'
import { useTabMeta } from '@/hooks/workspace'
import { cn } from '@/lib/utils'
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
    getDefaultModel,
    effort: preferredEffort
  } = useAgentSessionLaunchPreferences()

  const [agent, setAgent] = useState<Agent>((tab.data?.agent as Agent | undefined) ?? 'claude')
  const [model, setModel] = useState(tab.data?.model ?? getDefaultModel(agent))
  const [effort, setEffort] = useState<Effort>((tab.data?.effort as Effort | undefined) ?? preferredEffort)
  const [thinking, setThinking] = useState(tab.data?.thinking ?? false)
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

  const selectedModelOption = models.find(m => m.name === model)
  const selectedEffortOptions = selectedModelOption?.effortOptions ?? []
  const showEffortSelector = selectedEffortOptions.length > 0

  const handleRun = () => {
    const agentLabel = agent === 'codex' ? 'Codex' : 'Claude'
    openTab({
      type: 'terminal',
      component: TerminalTabContent,
      title: `${agentLabel} (${model})`,
      closeable: true,
      data: { agent, model, effort, thinking }
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
                <button
                  key={agentOption}
                  type="button"
                  role="radio"
                  aria-checked={agent === agentOption}
                  onClick={() => handleAgentSelect(agentOption)}
                  className={cn(
                    'relative aspect-[5/7] w-40 shrink-0 overflow-hidden rounded-lg border px-3 py-2 text-left text-sm font-medium transition-colors duration-100 cursor-pointer',
                    agent === agentOption
                      ? 'border-primary bg-primary/5 shadow-[inset_0_0_0_1px_hsl(var(--primary)/0.2)]'
                      : 'hover:bg-accent'
                  )}>
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
                </button>
              ))}
            </div>
            <FieldDescription>Model options update based on provider.</FieldDescription>
          </Field>

          <Field>
            <FieldLabel>Model</FieldLabel>
            <div className="flex flex-col gap-1" role="radiogroup" aria-label="Model">
              {models.map(modelOption => (
                <button
                  key={modelOption.name}
                  type="button"
                  role="radio"
                  aria-checked={model === modelOption.name}
                  onClick={() => handleModelSelect(modelOption.name)}
                  disabled={loadingModels}
                  className={cn(
                    'flex flex-col items-start gap-0.5 rounded-md border px-3 py-2 text-left text-sm font-medium transition-colors duration-100 cursor-pointer disabled:cursor-not-allowed disabled:opacity-50',
                    model === modelOption.name ? 'border-primary bg-primary/5' : 'hover:bg-accent'
                  )}>
                  <span>{modelOption.name}</span>
                  <span className="text-xs font-normal text-muted-foreground">{modelOption.description}</span>
                  {modelOption.sessionModel && modelOption.sessionModel !== modelOption.name && (
                    <span className="text-[11px] font-normal text-muted-foreground/90">
                      Runs as: {modelOption.sessionModel}
                    </span>
                  )}
                </button>
              ))}
            </div>
          </Field>

          <Field orientation="horizontal">
            <div className="flex-1">
              <FieldLabel>Extended Thinking</FieldLabel>
              <FieldDescription>Enable deeper planning when supported by the provider.</FieldDescription>
            </div>
            <Switch checked={thinking} onCheckedChange={handleThinkingChange} />
          </Field>

          {showEffortSelector && (
            <Field>
              <FieldLabel>Effort</FieldLabel>
              <div className="flex flex-col gap-1" role="radiogroup" aria-label="Effort">
                {selectedEffortOptions.map(effortOption => (
                  <button
                    key={effortOption}
                    type="button"
                    role="radio"
                    aria-checked={effort === effortOption}
                    onClick={() => handleEffortSelect(effortOption as Effort)}
                    className={cn(
                      'flex items-center rounded-md border px-3 py-2 text-left text-sm font-medium transition-colors duration-100 cursor-pointer',
                      effort === effortOption ? 'border-primary bg-primary/5' : 'hover:bg-accent'
                    )}>
                    {effortOption}
                  </button>
                ))}
              </div>
              <FieldDescription>Maps to `claude --effort &lt;level&gt;`.</FieldDescription>
            </Field>
          )}
        </div>
      </ScrollArea>
      <Separator />
      {error && (
        <div className="px-3 pt-1.5">
          <InlineError error={error} />
        </div>
      )}
      <div className="px-3 py-1.5 flex justify-end gap-2">
        <Button type="button" onClick={handleRun} disabled={loadingModels || !model || models.length === 0}>
          Run
        </Button>
      </div>
    </div>
  )
}
