import { Bot } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { InlineError } from '@/components/shared'
import { Button } from '@/components/ui/button'
import { Field, FieldDescription, FieldLabel } from '@/components/ui/field'
import { FormDescription, FormHeader, FormTitle } from '@/components/ui/form-header'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { Switch } from '@/components/ui/switch'
import { type Agent, useModelThinkingPreferences } from '@/hooks/preferences'
import { useTabMeta } from '@/hooks/workspace'
import { terminalBridgeListModels } from '@/lib/terminal/terminalBridgeClient'
import { cn } from '@/lib/utils'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'
import { TerminalTabContent } from './TerminalTabContent'

export function TerminalRunFormTabContent({ tab }: { tab: WorkspaceTab }) {
  useTabMeta(tab.id, 'Run Agent', Bot)
  const closeTab = useWorkspaceStore(s => s.closeTab)
  const openTab = useWorkspaceStore(s => s.openTab)
  const {
    setAgent: saveAgent,
    setModel: saveModel,
    setThinking: saveThinking,
    getDefaultModel
  } = useModelThinkingPreferences()

  const [agent, setAgent] = useState<Agent>((tab.data?.agent as Agent | undefined) ?? 'claude')
  const [model, setModel] = useState(tab.data?.model ?? getDefaultModel(agent))
  const [thinking, setThinking] = useState(tab.data?.thinking ?? false)
  const [models, setModels] = useState<string[]>([])
  const [loadingModels, setLoadingModels] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const fallbackModels = useMemo(
    () => ({
      claude: ['claude-sonnet-4', 'claude-opus-4', 'claude-haiku-3.5'],
      codex: ['gpt-5-codex', 'gpt-5', 'o4-mini']
    }),
    []
  )

  useEffect(() => {
    let cancelled = false
    setLoadingModels(true)
    terminalBridgeListModels(agent)
      .then(res => {
        if (cancelled) return
        const nextModels = res.models.length > 0 ? res.models : fallbackModels[agent]
        setModels(nextModels)
        if (!nextModels.includes(model)) {
          setModel(nextModels[0] ?? getDefaultModel(agent))
        }
        setError(null)
      })
      .catch(err => {
        if (cancelled) return
        const nextModels = fallbackModels[agent]
        setModels(nextModels)
        if (!nextModels.includes(model)) {
          setModel(nextModels[0] ?? getDefaultModel(agent))
        }
        setError(`Failed to load model list: ${String(err)}`)
      })
      .finally(() => {
        if (!cancelled) setLoadingModels(false)
      })

    return () => {
      cancelled = true
    }
  }, [agent, fallbackModels, getDefaultModel, model])

  const handleRun = () => {
    saveAgent(agent)
    saveModel(model)
    saveThinking(thinking)
    const agentLabel = agent === 'codex' ? 'Codex' : 'Claude'
    openTab({
      type: 'terminal',
      component: TerminalTabContent,
      title: `${agentLabel} (${model})`,
      closeable: true,
      data: { agent, model, thinking }
    })
    closeTab(tab.id)
  }

  return (
    <div className="h-full flex flex-col">
      <div className="px-4">
        <FormHeader>
          <FormTitle>Run Agent</FormTitle>
          <FormDescription>Choose provider-specific settings, then start a terminal session.</FormDescription>
        </FormHeader>
      </div>
      <Separator />
      <ScrollArea className="flex-1 min-h-0">
        <div className="px-4 py-4 space-y-4">
          <Field>
            <FieldLabel>Agent Provider</FieldLabel>
            <div className="flex flex-col gap-1" role="radiogroup" aria-label="Agent provider">
              {(['claude', 'codex'] as const).map(agentOption => (
                <button
                  key={agentOption}
                  type="button"
                  role="radio"
                  aria-checked={agent === agentOption}
                  onClick={() => setAgent(agentOption)}
                  className={cn(
                    'flex items-center rounded-md border px-3 py-2 text-left text-sm font-medium transition-colors duration-100 cursor-pointer',
                    agent === agentOption ? 'border-primary bg-primary/5' : 'hover:bg-accent'
                  )}>
                  {agentOption === 'claude' ? 'Claude' : 'Codex'}
                </button>
              ))}
            </div>
            <FieldDescription>Model options update based on provider.</FieldDescription>
          </Field>

          <Field>
            <FieldLabel>Model</FieldLabel>
            <div className="flex flex-col gap-1" role="radiogroup" aria-label="Model">
              {models.map(modelName => (
                <button
                  key={modelName}
                  type="button"
                  role="radio"
                  aria-checked={model === modelName}
                  onClick={() => setModel(modelName)}
                  disabled={loadingModels}
                  className={cn(
                    'flex items-center rounded-md border px-3 py-2 text-left text-sm font-medium transition-colors duration-100 cursor-pointer disabled:cursor-not-allowed disabled:opacity-50',
                    model === modelName ? 'border-primary bg-primary/5' : 'hover:bg-accent'
                  )}>
                  {modelName}
                </button>
              ))}
            </div>
          </Field>

          <Field orientation="horizontal">
            <div className="flex-1">
              <FieldLabel>Extended Thinking</FieldLabel>
              <FieldDescription>Enable deeper planning when supported by the provider.</FieldDescription>
            </div>
            <Switch checked={thinking} onCheckedChange={setThinking} />
          </Field>
        </div>
      </ScrollArea>
      <Separator />
      {error && (
        <div className="px-3 pt-1.5">
          <InlineError error={error} onDismiss={() => setError(null)} />
        </div>
      )}
      <div className="px-3 py-1.5 flex justify-end gap-2">
        <Button type="button" variant="outline" onClick={() => closeTab(tab.id)}>
          Cancel
        </Button>
        <Button type="button" onClick={handleRun} disabled={loadingModels || !model}>
          Run
        </Button>
      </div>
    </div>
  )
}
