import { AlertCircle } from 'lucide-react'
import type { TaskSignal } from '@/types/generated'

interface SignalDisplayProps {
  signal: TaskSignal
}

export function SignalDisplay({ signal }: SignalDisplayProps) {
  const verb = signal.signal_verb
  if (!verb) return null

  const {
    summary,
    remaining,
    reason,
    question,
    blocking,
    what,
    severity,
    category,
    text,
    kind,
    rationale,
    scope,
    why,
    on,
    options,
    preferred
  } = signal

  return (
    <>
      {verb === 'done' && summary && (
        <div className="space-y-1">
          <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide">Summary</div>
          <p className="text-sm leading-relaxed text-foreground">{summary}</p>
        </div>
      )}

      {verb === 'partial' && (
        <div className="space-y-2">
          {summary && (
            <div className="space-y-1">
              <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide">Progress Made</div>
              <p className="text-sm leading-relaxed text-foreground">{summary}</p>
            </div>
          )}
          {remaining && (
            <div className="space-y-1">
              <div className="text-xs font-semibold text-amber-600 dark:text-amber-500 uppercase tracking-wide">
                Still Remaining
              </div>
              <p className="text-sm leading-relaxed text-foreground">{remaining}</p>
            </div>
          )}
        </div>
      )}

      {verb === 'stuck' && reason && (
        <div className="space-y-1">
          <div className="text-xs font-semibold text-destructive uppercase tracking-wide">Blocked Because</div>
          <p className="text-sm leading-relaxed text-foreground">{reason}</p>
        </div>
      )}

      {verb === 'ask' && question && (
        <div className="space-y-2">
          <div className="flex items-center gap-2">
            <div className="text-xs font-semibold text-blue-600 dark:text-blue-500 uppercase tracking-wide">
              Question
            </div>
            {blocking && (
              <div className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md bg-destructive/10 text-destructive text-xs font-medium">
                <AlertCircle className="h-3 w-3" />
                Blocking
              </div>
            )}
          </div>
          <p className="text-sm leading-relaxed text-foreground">{question}</p>
          {options && options.length > 0 && (
            <div className="space-y-1">
              <span className="text-xs font-medium text-muted-foreground uppercase tracking-wider">Options</span>
              <ul className="list-disc list-inside space-y-0.5">
                {options.map((opt, i) => (
                  <li key={i} className="text-sm text-foreground/90">
                    {opt}
                    {preferred === opt && <span className="ml-1.5 text-xs text-muted-foreground">(recommended)</span>}
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}

      {verb === 'flag' && (
        <div className="space-y-2">
          {(severity || category) && (
            <div className="flex items-center gap-2 text-xs">
              {severity && (
                <span
                  className={`font-bold uppercase tracking-wider ${severity === 'blocking' ? 'text-red-600 dark:text-red-500' : 'text-orange-600 dark:text-orange-500'}`}>
                  {severity}
                </span>
              )}
              {severity && category && <span className="text-muted-foreground">·</span>}
              {category && <span className="text-muted-foreground">{category}</span>}
            </div>
          )}
          {what && (
            <div className="space-y-1">
              <div className="text-xs font-semibold text-orange-600 dark:text-orange-500 uppercase tracking-wide">
                Issue Flagged
              </div>
              <p className="text-sm leading-relaxed text-foreground">{what}</p>
            </div>
          )}
        </div>
      )}

      {verb === 'learned' && (
        <div className="space-y-2">
          {text && (
            <div className="space-y-1">
              <div className="flex items-center gap-2 text-xs">
                <div className="font-semibold text-cyan-600 dark:text-cyan-500 uppercase tracking-wide">Knowledge</div>
                {kind && (
                  <>
                    <span className="text-muted-foreground">·</span>
                    <span className="text-muted-foreground">{kind}</span>
                  </>
                )}
              </div>
              <p className="text-sm leading-relaxed text-foreground font-medium">{text}</p>
            </div>
          )}
          {rationale && (
            <div className="space-y-1">
              <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide">
                Why This Matters
              </div>
              <p className="text-sm leading-relaxed text-muted-foreground">{rationale}</p>
            </div>
          )}
          {scope && (
            <div className="flex items-center gap-2 text-xs">
              <span className="font-semibold text-muted-foreground uppercase tracking-wide">Scope:</span>
              <span className="text-muted-foreground">{scope}</span>
            </div>
          )}
        </div>
      )}

      {verb === 'suggest' && (
        <div className="space-y-2">
          {what && (
            <div className="space-y-1">
              <div className="flex items-center gap-2 text-xs">
                <div className="font-semibold text-purple-600 dark:text-purple-500 uppercase tracking-wide">
                  Suggestion
                </div>
                {kind && (
                  <>
                    <span className="text-muted-foreground">·</span>
                    <span className="text-muted-foreground">{kind}</span>
                  </>
                )}
              </div>
              <p className="text-sm leading-relaxed text-foreground">{what}</p>
            </div>
          )}
          {why && (
            <div className="space-y-1">
              <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide">Rationale</div>
              <p className="text-sm leading-relaxed text-muted-foreground">{why}</p>
            </div>
          )}
        </div>
      )}

      {verb === 'blocked' && (
        <div className="space-y-1">
          <div className="flex items-center gap-2 text-xs">
            <div className="font-semibold text-red-600 dark:text-red-500 uppercase tracking-wide">Blocked By</div>
            {kind && (
              <>
                <span className="text-muted-foreground">·</span>
                <span className="text-muted-foreground">{kind}</span>
              </>
            )}
          </div>
          {on && <p className="text-sm leading-relaxed text-foreground">{on}</p>}
        </div>
      )}
    </>
  )
}
