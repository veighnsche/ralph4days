import type { ReactNode } from 'react'
import { cn } from '@/lib/utils'

type LaunchOptionsReadoutRow = {
  label: string
  value: ReactNode
  valueTitle?: string
  right?: ReactNode
  monospace?: boolean
  muted?: boolean
}

function ValuePill({
  children,
  title,
  monospace,
  muted
}: {
  children: ReactNode
  title?: string
  monospace?: boolean
  muted?: boolean
}) {
  return (
    <span
      title={title}
      className={cn(
        'inline-flex min-w-0 items-center rounded-md border border-border/60 bg-muted/20 px-2 py-1 text-xs leading-none',
        monospace && 'font-mono',
        muted && 'text-muted-foreground'
      )}>
      <span className="min-w-0 truncate">{children}</span>
    </span>
  )
}

export function LaunchOptionsReadout({
  rows,
  variant = 'panel',
  className
}: {
  rows: LaunchOptionsReadoutRow[]
  variant?: 'panel' | 'tooltip'
  className?: string
}) {
  return (
    <div
      className={cn(
        'grid gap-1.5',
        variant === 'panel' && 'rounded-md border border-border/60 bg-muted/10 p-2',
        className
      )}>
      {rows.map(row => (
        <div key={row.label} className="flex min-w-0 items-center gap-2">
          <span className="w-16 shrink-0 text-[11px] font-medium uppercase tracking-wide text-muted-foreground">
            {row.label}
          </span>
          <div className="min-w-0 flex-1">
            <ValuePill title={row.valueTitle} monospace={row.monospace} muted={row.muted}>
              {row.value}
            </ValuePill>
          </div>
          {row.right ? <div className="shrink-0">{row.right}</div> : null}
        </div>
      ))}
    </div>
  )
}
