import { ChevronDown } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { type SignalVerb, VERB_CONFIG } from '@/constants/signals'
import { formatDate } from '@/lib/formatDate'
import type { TaskSignal } from '@/types/generated'
import { SignalCard } from './SignalCard'

interface SessionGroupProps {
  sessionId: string
  sessionNumber: number
  signals: TaskSignal[]
  defaultOpen?: boolean
  onAnswerAsk?: (signalId: number, answer: string) => void
  isAnswering?: boolean
}

function getClosingVerb(signals: TaskSignal[]): SignalVerb | null {
  for (let i = signals.length - 1; i >= 0; i--) {
    const verb = signals[i].verb
    if (verb === 'done' || verb === 'partial' || verb === 'stuck') {
      return verb as SignalVerb
    }
  }
  return null
}

function buildSessionSummary(signals: TaskSignal[]): string {
  const counts: Record<string, number> = {}
  for (const s of signals) {
    if (s.verb !== 'done' && s.verb !== 'partial' && s.verb !== 'stuck') {
      counts[s.verb] = (counts[s.verb] ?? 0) + 1
    }
  }
  return Object.entries(counts)
    .map(([verb, count]) => `${count} ${verb}`)
    .join(' · ')
}

export function SessionGroup({
  sessionNumber,
  signals,
  defaultOpen = false,
  onAnswerAsk,
  isAnswering
}: SessionGroupProps) {
  const closingVerb = getClosingVerb(signals)
  const closingConfig = closingVerb ? VERB_CONFIG[closingVerb] : null
  const summary = buildSessionSummary(signals)
  const lastSignal = signals[signals.length - 1]
  const timestamp = lastSignal ? formatDate(lastSignal.createdAt) : ''

  return (
    <Collapsible defaultOpen={defaultOpen}>
      <CollapsibleTrigger className="w-full group">
        <div className="flex items-center gap-2 py-1.5 px-1 hover:opacity-70 transition-opacity text-left">
          <span className="text-xs font-medium text-muted-foreground">Session {sessionNumber}</span>
          {closingConfig && (
            <Badge
              variant="outline"
              className="text-xs px-1.5 py-0 h-4"
              style={{ borderColor: closingConfig.color, color: closingConfig.color }}>
              {closingConfig.label}
            </Badge>
          )}
          {summary && <span className="text-xs text-muted-foreground">{summary}</span>}
          <span className="text-xs text-muted-foreground ml-auto">{timestamp}</span>
          <ChevronDown className="h-3.5 w-3.5 text-muted-foreground transition-transform group-data-[state=closed]:-rotate-90" />
        </div>
      </CollapsibleTrigger>
      <CollapsibleContent>
        <div className="space-y-1.5 pb-2">
          {signals.map(signal => (
            <SignalCard key={signal.id} signal={signal} onAnswerAsk={onAnswerAsk} isAnswering={isAnswering} />
          ))}
        </div>
      </CollapsibleContent>
    </Collapsible>
  )
}
