import { AlertCircle } from 'lucide-react'
import type { ReactNode } from 'react'
import type { TaskSignal } from '@/types/generated'

interface SignalDisplayProps {
  signal: TaskSignal
}

type SignalRenderer = (signal: TaskSignal) => ReactNode | null

function renderDone(signal: TaskSignal): ReactNode | null {
  if (!signal.summary) return null

  return (
    <div className="space-y-1">
      <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide">Summary</div>
      <p className="text-sm leading-relaxed text-foreground">{signal.summary}</p>
    </div>
  )
}

function renderPartial(signal: TaskSignal): ReactNode | null {
  if (!(signal.summary || signal.remaining)) return null

  return (
    <div className="space-y-2">
      {signal.summary && (
        <div className="space-y-1">
          <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide">Progress Made</div>
          <p className="text-sm leading-relaxed text-foreground">{signal.summary}</p>
        </div>
      )}
      {signal.remaining && (
        <div className="space-y-1">
          <div className="text-xs font-semibold text-amber-600 dark:text-amber-500 uppercase tracking-wide">
            Still Remaining
          </div>
          <p className="text-sm leading-relaxed text-foreground">{signal.remaining}</p>
        </div>
      )}
    </div>
  )
}

function renderStuck(signal: TaskSignal): ReactNode | null {
  if (!signal.reason) return null

  return (
    <div className="space-y-1">
      <div className="text-xs font-semibold text-destructive uppercase tracking-wide">Blocked Because</div>
      <p className="text-sm leading-relaxed text-foreground">{signal.reason}</p>
    </div>
  )
}

function renderAsk(signal: TaskSignal): ReactNode | null {
  if (!(signal.question || (signal.options && signal.options.length > 0))) return null

  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2">
        <div className="text-xs font-semibold text-blue-600 dark:text-blue-500 uppercase tracking-wider">Question</div>
        {signal.blocking && (
          <div className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md bg-destructive/10 text-destructive text-xs font-medium">
            <AlertCircle className="h-3 w-3" />
            Blocking
          </div>
        )}
      </div>
      {signal.question && <p className="text-sm leading-relaxed text-foreground">{signal.question}</p>}
      {signal.options && signal.options.length > 0 && (
        <div className="space-y-1">
          <span className="text-xs font-medium text-muted-foreground uppercase tracking-wider">Options</span>
          <ul className="list-disc list-inside space-y-0.5">
            {signal.options.map(opt => (
              <li key={`option-${opt}`} className="text-sm text-foreground/90">
                {opt}
                {signal.preferred === opt && (
                  <span className="ml-1.5 text-xs text-muted-foreground">(recommended)</span>
                )}
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  )
}

function renderFlag(signal: TaskSignal): ReactNode | null {
  if (!(signal.what || signal.severity || signal.category)) return null

  return (
    <div className="space-y-2">
      {(signal.severity || signal.category) && (
        <div className="flex items-center gap-2 text-xs">
          {signal.severity && (
            <span
              className={`font-bold uppercase tracking-wider ${signal.severity === 'blocking' ? 'text-red-600 dark:text-red-500' : 'text-orange-600 dark:text-orange-500'}`}>
              {signal.severity}
            </span>
          )}
          {signal.severity && signal.category && <span className="text-muted-foreground">·</span>}
          {signal.category && <span className="text-muted-foreground">{signal.category}</span>}
        </div>
      )}
      {signal.what && (
        <div className="space-y-1">
          <div className="text-xs font-semibold text-orange-600 dark:text-orange-500 uppercase tracking-wide">
            Issue Flagged
          </div>
          <p className="text-sm leading-relaxed text-foreground">{signal.what}</p>
        </div>
      )}
    </div>
  )
}

function renderLearned(signal: TaskSignal): ReactNode | null {
  if (!(signal.text || signal.rationale || signal.scope)) return null

  return (
    <div className="space-y-2">
      {signal.text && (
        <div className="space-y-1">
          <div className="flex items-center gap-2 text-xs">
            <div className="font-semibold text-cyan-600 dark:text-cyan-500 uppercase tracking-wide">Knowledge</div>
            {signal.kind && (
              <>
                <span className="text-muted-foreground">·</span>
                <span className="text-muted-foreground">{signal.kind}</span>
              </>
            )}
          </div>
          <p className="text-sm leading-relaxed text-foreground font-medium">{signal.text}</p>
        </div>
      )}
      {signal.rationale && (
        <div className="space-y-1">
          <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide">Why This Matters</div>
          <p className="text-sm leading-relaxed text-muted-foreground">{signal.rationale}</p>
        </div>
      )}
      {signal.scope && (
        <div className="flex items-center gap-2 text-xs">
          <span className="font-semibold text-muted-foreground uppercase tracking-wide">Scope:</span>
          <span className="text-muted-foreground">{signal.scope}</span>
        </div>
      )}
    </div>
  )
}

function renderSuggest(signal: TaskSignal): ReactNode | null {
  if (!(signal.what || signal.why)) return null

  return (
    <div className="space-y-2">
      {signal.what && (
        <div className="space-y-1">
          <div className="flex items-center gap-2 text-xs">
            <div className="font-semibold text-purple-600 dark:text-purple-500 uppercase tracking-wide">Suggestion</div>
            {signal.kind && (
              <>
                <span className="text-muted-foreground">·</span>
                <span className="text-muted-foreground">{signal.kind}</span>
              </>
            )}
          </div>
          <p className="text-sm leading-relaxed text-foreground">{signal.what}</p>
        </div>
      )}
      {signal.why && (
        <div className="space-y-1">
          <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide">Rationale</div>
          <p className="text-sm leading-relaxed text-muted-foreground">{signal.why}</p>
        </div>
      )}
    </div>
  )
}

function renderBlocked(signal: TaskSignal): ReactNode | null {
  if (!signal.on) return null

  return (
    <div className="space-y-1">
      <div className="flex items-center gap-2 text-xs">
        <div className="font-semibold text-red-600 dark:text-red-500 uppercase tracking-wide">Blocked By</div>
        {signal.kind && (
          <>
            <span className="text-muted-foreground">·</span>
            <span className="text-muted-foreground">{signal.kind}</span>
          </>
        )}
      </div>
      <p className="text-sm leading-relaxed text-foreground">{signal.on}</p>
    </div>
  )
}

const SIGNAL_RENDERERS: Partial<Record<NonNullable<TaskSignal['signal_verb']>, SignalRenderer>> = {
  done: renderDone,
  partial: renderPartial,
  stuck: renderStuck,
  ask: renderAsk,
  flag: renderFlag,
  learned: renderLearned,
  suggest: renderSuggest,
  blocked: renderBlocked
}

export function SignalDisplay({ signal }: SignalDisplayProps) {
  const verb = signal.signal_verb
  if (!verb) return null

  const renderer = SIGNAL_RENDERERS[verb]
  if (!renderer) return null

  const rendered = renderer(signal)
  if (!rendered) return null

  return <>{rendered}</>
}
