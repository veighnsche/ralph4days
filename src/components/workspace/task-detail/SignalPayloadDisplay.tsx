interface SignalPayloadDisplayProps {
  verb: string
  payload: Record<string, unknown>
}

export function SignalPayloadDisplay({ verb, payload }: SignalPayloadDisplayProps) {
  const summary = typeof payload.summary === 'string' ? payload.summary : null
  const remaining = typeof payload.remaining === 'string' ? payload.remaining : null
  const reason = typeof payload.reason === 'string' ? payload.reason : null
  const question = typeof payload.question === 'string' ? payload.question : null
  const blocking = typeof payload.blocking === 'boolean' ? payload.blocking : null
  const what = typeof payload.what === 'string' ? payload.what : null
  const severity = typeof payload.severity === 'string' ? payload.severity : null
  const category = typeof payload.category === 'string' ? payload.category : null
  const text = typeof payload.text === 'string' ? payload.text : null
  const kind = typeof payload.kind === 'string' ? payload.kind : null
  const rationale = typeof payload.rationale === 'string' ? payload.rationale : null
  const scope = typeof payload.scope === 'string' ? payload.scope : null
  const why = typeof payload.why === 'string' ? payload.why : null
  const on = typeof payload.on === 'string' ? payload.on : null

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
              <span className="text-xs font-bold uppercase tracking-wider text-red-600 dark:text-red-500">
                ⚠ Blocking
              </span>
            )}
          </div>
          <p className="text-sm leading-relaxed text-foreground">{question}</p>
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
