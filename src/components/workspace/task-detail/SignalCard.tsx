import { useState } from 'react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import { Textarea } from '@/components/ui/textarea'
import { FLAG_SEVERITY_CONFIG, type FlagSeverity, type SignalVerb, VERB_CONFIG } from '@/constants/signals'
import { formatDate } from '@/lib/formatDate'
import type { TaskSignal } from '@/types/generated'

interface SignalCardProps {
  signal: TaskSignal
  onAnswerAsk?: (signalId: number, answer: string) => void
  isAnswering?: boolean
}

interface ParsedPayload {
  [key: string]: unknown
}

const VERB_BORDER_COLORS: Record<SignalVerb, string> = {
  done: 'var(--status-done)',
  partial: 'var(--priority-medium)',
  stuck: 'var(--status-blocked)',
  ask: 'var(--secondary)',
  flag: 'var(--priority-medium)',
  learned: 'var(--primary)',
  suggest: 'var(--primary)',
  blocked: 'var(--status-blocked)'
}

function parsePayload(payload: string): ParsedPayload {
  try {
    return JSON.parse(payload) as ParsedPayload
  } catch {
    return { text: payload }
  }
}

function str(val: unknown): string {
  return typeof val === 'string' ? val : ''
}

function DoneContent({ data }: { data: ParsedPayload }) {
  return <p className="text-sm text-foreground/90">{str(data.summary)}</p>
}

function PartialContent({ data }: { data: ParsedPayload }) {
  return (
    <div className="space-y-1.5">
      <p className="text-sm text-foreground/90">{str(data.summary)}</p>
      {typeof data.remaining === 'string' && (
        <div className="bg-muted/50 rounded px-2 py-1.5">
          <span className="text-xs font-medium text-muted-foreground">Remaining:</span>
          <p className="text-sm text-foreground/80">{data.remaining}</p>
        </div>
      )}
    </div>
  )
}

function StuckContent({ data }: { data: ParsedPayload }) {
  return <p className="text-sm text-foreground/90">{str(data.reason)}</p>
}

function AskContent({
  signal,
  data,
  onAnswer,
  isAnswering
}: {
  signal: TaskSignal
  data: ParsedPayload
  onAnswer?: (signalId: number, answer: string) => void
  isAnswering?: boolean
}) {
  const [selectedOption, setSelectedOption] = useState('')
  const [customAnswer, setCustomAnswer] = useState('')
  const options = Array.isArray(data.options) ? (data.options as string[]) : []
  const preferred = str(data.preferred)
  const isBlocking = data.blocking === true
  const isUnanswered = !signal.answered

  const handleSubmit = () => {
    const answer = customAnswer.trim() || selectedOption
    if (answer && onAnswer) {
      onAnswer(signal.id, answer)
    }
  }

  return (
    <div className="space-y-2">
      <p className="text-sm text-foreground/90">{str(data.question)}</p>

      {options.length > 0 && isUnanswered && isBlocking && onAnswer && (
        <RadioGroup value={selectedOption} onValueChange={setSelectedOption} className="gap-1.5">
          {options.map(opt => (
            <div key={opt} className="flex items-center gap-2">
              <RadioGroupItem value={opt} id={`opt-${signal.id}-${opt}`} />
              <Label htmlFor={`opt-${signal.id}-${opt}`} className="text-sm cursor-pointer">
                {opt}
                {opt === preferred && (
                  <Badge variant="secondary" className="ml-1.5 text-xs px-1.5 py-0 h-4">
                    recommended
                  </Badge>
                )}
              </Label>
            </div>
          ))}
        </RadioGroup>
      )}

      {!isUnanswered && options.length > 0 && (
        <div className="flex flex-wrap gap-1">
          {options.map(opt => (
            <Badge key={opt} variant={opt === preferred ? 'secondary' : 'outline'} className="text-xs">
              {opt}
            </Badge>
          ))}
        </div>
      )}

      {isUnanswered && isBlocking && onAnswer && (
        <div className="space-y-2">
          <Textarea
            value={customAnswer}
            onChange={e => setCustomAnswer(e.target.value)}
            placeholder="Or type a custom answer..."
            className="min-h-[60px] text-sm"
          />
          <Button
            size="sm"
            className="h-7"
            onClick={handleSubmit}
            disabled={!(selectedOption || customAnswer.trim()) || isAnswering}>
            {isAnswering ? 'Submitting...' : 'Answer'}
          </Button>
        </div>
      )}

      {signal.answered && (
        <div className="bg-green-500/10 border border-green-500/20 rounded px-2 py-1.5">
          <span className="text-xs font-medium text-green-600 dark:text-green-400">Answer:</span>
          <p className="text-sm text-foreground/90">{signal.answered}</p>
        </div>
      )}
    </div>
  )
}

function FlagContent({ data }: { data: ParsedPayload }) {
  const severity = str(data.severity) as FlagSeverity
  const category = str(data.category)
  const severityConfig = FLAG_SEVERITY_CONFIG[severity]

  return (
    <div className="space-y-1">
      <p className="text-sm text-foreground/90">{str(data.what)}</p>
      <div className="flex gap-1.5">
        {severityConfig && (
          <Badge
            variant="outline"
            className="text-xs"
            style={{ borderColor: severityConfig.color, color: severityConfig.color }}>
            {severityConfig.label}
          </Badge>
        )}
        {category && (
          <Badge variant="outline" className="text-xs">
            {category}
          </Badge>
        )}
      </div>
    </div>
  )
}

function LearnedContent({ data }: { data: ParsedPayload }) {
  const kind = str(data.kind)
  const rationale = str(data.rationale)

  return (
    <div className="space-y-1">
      <p className="text-sm text-foreground/90">{str(data.text)}</p>
      {kind && (
        <div className="flex items-center gap-1.5">
          <Badge variant="outline" className="text-xs">
            {kind}
          </Badge>
        </div>
      )}
      {rationale && <p className="text-xs italic text-muted-foreground">{rationale}</p>}
    </div>
  )
}

function SuggestContent({ data }: { data: ParsedPayload }) {
  const taskId = typeof data.created_task_id === 'number' ? data.created_task_id : undefined
  const kind = str(data.kind)
  const why = str(data.why)

  return (
    <div className="space-y-1">
      <p className="text-sm text-foreground/90">{str(data.what)}</p>
      <div className="flex items-center gap-1.5">
        {kind && (
          <Badge variant="outline" className="text-xs">
            {kind}
          </Badge>
        )}
        {taskId != null && (
          <Badge variant="secondary" className="text-xs font-mono">
            #{taskId.toString().padStart(3, '0')}
          </Badge>
        )}
      </div>
      {why && <p className="text-xs text-muted-foreground">{why}</p>}
    </div>
  )
}

function BlockedContent({ data }: { data: ParsedPayload }) {
  const kind = str(data.kind)
  const detail = str(data.detail)

  return (
    <div className="space-y-1">
      <p className="text-sm text-foreground/90">{str(data.on)}</p>
      {kind && (
        <div className="flex items-center gap-1.5">
          <Badge variant="outline" className="text-xs">
            {kind}
          </Badge>
        </div>
      )}
      {detail && <p className="text-xs text-muted-foreground">{detail}</p>}
    </div>
  )
}

function getVerbBorderColor(verb: string, data: ParsedPayload): string {
  if (verb === 'flag') {
    const sev = str(data.severity) as FlagSeverity
    return FLAG_SEVERITY_CONFIG[sev]?.color ?? VERB_BORDER_COLORS.flag
  }
  return VERB_BORDER_COLORS[verb as SignalVerb] ?? 'var(--border)'
}

export function SignalCard({ signal, onAnswerAsk, isAnswering }: SignalCardProps) {
  const verb = signal.verb as SignalVerb
  const config = VERB_CONFIG[verb]
  if (!config) return null

  const VerbIcon = config.icon
  const data = parsePayload(signal.payload)
  const borderColor = getVerbBorderColor(signal.verb, data)

  return (
    <div className="border-l-2 rounded-r-md py-2 px-3 bg-card/50" style={{ borderLeftColor: borderColor }}>
      <div className="flex items-center gap-1.5 mb-1">
        <VerbIcon className="h-3.5 w-3.5" style={{ color: config.color }} />
        <span className="text-xs font-medium" style={{ color: config.color }}>
          {config.label}
        </span>
        <span className="text-xs text-muted-foreground">· {formatDate(signal.createdAt)}</span>
      </div>

      {verb === 'done' && <DoneContent data={data} />}
      {verb === 'partial' && <PartialContent data={data} />}
      {verb === 'stuck' && <StuckContent data={data} />}
      {verb === 'ask' && <AskContent signal={signal} data={data} onAnswer={onAnswerAsk} isAnswering={isAnswering} />}
      {verb === 'flag' && <FlagContent data={data} />}
      {verb === 'learned' && <LearnedContent data={data} />}
      {verb === 'suggest' && <SuggestContent data={data} />}
      {verb === 'blocked' && <BlockedContent data={data} />}
    </div>
  )
}
