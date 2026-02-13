import { Field } from '@/components/ui/field'
import { SelectableCard } from '@/components/ui/selectable-card'
import type { Agent } from '@/hooks/preferences'
import { AGENT_OPTIONS, AGENT_PROVIDER_META } from '../constants'
import { useAgentSessionConfigActions, useAgentSessionConfigLaunchState } from '../hooks/useAgentSessionConfigTabState'
import { PickerSectionHeader } from './PickerSectionHeader'

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

export function AgentProviderPicker() {
  const { agent } = useAgentSessionConfigLaunchState()
  const { setAgent } = useAgentSessionConfigActions()

  return (
    <Field className="gap-2">
      <PickerSectionHeader title="Agent Provider" />
      <div className="flex items-center justify-start gap-3" role="radiogroup" aria-label="Agent provider">
        {AGENT_OPTIONS.map(agentOption => (
          <AgentProviderCard
            key={agentOption}
            agentOption={agentOption}
            selected={agent === agentOption}
            onSelect={() => setAgent(agentOption)}
          />
        ))}
      </div>
    </Field>
  )
}
