import { Brain } from 'lucide-react'
import { Fragment } from 'react'
import { SelectableCard } from '@/components/ui/selectable-card'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import type { Effort } from '@/hooks/preferences'
import type { TerminalBridgeModelOption } from '@/types/generated'
import { asEffort } from '../state'
import { useAgentSessionConfigStore } from '../store'

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

export function ModelPicker() {
  const models = useAgentSessionConfigStore(state => state.models)
  const model = useAgentSessionConfigStore(state => state.model)
  const effort = useAgentSessionConfigStore(state => state.effort)
  const thinking = useAgentSessionConfigStore(state => state.thinking)
  const loadingModels = useAgentSessionConfigStore(state => state.loadingModels)
  const setModel = useAgentSessionConfigStore(state => state.setModel)
  const setEffort = useAgentSessionConfigStore(state => state.setEffort)
  const setThinking = useAgentSessionConfigStore(state => state.setThinking)

  return (
    <div className="flex flex-col gap-1" role="radiogroup" aria-label="Model">
      <ModelSectionHeader thinking={thinking} onThinkingChange={setThinking} />
      {models.map(modelOption => {
        const selected = model === modelOption.name
        return (
          <Fragment key={modelOption.name}>
            <ModelCard
              modelOption={modelOption}
              selected={selected}
              loading={loadingModels}
              onSelect={() => setModel(modelOption.name)}
            />
            <ModelEffortRow
              modelOption={modelOption}
              selected={selected}
              effort={effort}
              onModelSelect={setModel}
              onEffortSelect={setEffort}
            />
          </Fragment>
        )
      })}
    </div>
  )
}
