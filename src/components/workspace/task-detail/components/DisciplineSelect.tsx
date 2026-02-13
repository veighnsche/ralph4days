import { Bot, Check, Pencil } from 'lucide-react'
import type { ComponentType, CSSProperties } from 'react'
import { useMemo, useState } from 'react'
import { CroppedImage } from '@/components/ui/cropped-image'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { useDisciplines } from '@/hooks/disciplines'
import { cn } from '@/lib/utils'

function DisciplineAvatar({
  name,
  color,
  faceCrop,
  Icon,
  className
}: {
  name: string
  color: string
  faceCrop?: { x: number; y: number; w: number; h: number }
  Icon?: ComponentType<{ className?: string; style?: CSSProperties }>
  className?: string
}) {
  if (faceCrop) {
    return (
      <span
        className={cn('relative overflow-hidden rounded-sm border', className)}
        style={{ borderColor: `${color}66`, backgroundColor: `color-mix(in oklch, ${color} 18%, transparent)` }}>
        <CroppedImage disciplineName={name} label="discipline-select-face" crop={faceCrop} className="h-full w-full" />
      </span>
    )
  }

  return (
    <span
      className={cn('inline-flex items-center justify-center rounded-sm border', className)}
      style={{ borderColor: `${color}66`, backgroundColor: `color-mix(in oklch, ${color} 18%, transparent)` }}>
      {Icon ? <Icon className="h-3.5 w-3.5" style={{ color }} /> : <Bot className="h-3.5 w-3.5" style={{ color }} />}
    </span>
  )
}

export function DisciplineSelect({
  value,
  onSelect,
  disabled
}: {
  value: string
  onSelect: (disciplineName: string) => void
  disabled?: boolean
}) {
  const { disciplines } = useDisciplines()
  const [open, setOpen] = useState(false)

  const selected = useMemo(() => disciplines.find(d => d.name === value), [disciplines, value])
  const SelectedIcon = selected?.icon

  if (!selected) {
    return (
      <span className="text-sm text-muted-foreground" title={value}>
        {value}
      </span>
    )
  }

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <button
          type="button"
          disabled={disabled}
          className={cn(
            'group inline-flex items-center gap-1.5 rounded-sm px-1 py-0.5 -mx-1 transition-colors',
            'hover:bg-muted/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring',
            'disabled:opacity-50 disabled:pointer-events-none'
          )}
          aria-label={`Select discipline. Current: ${selected.displayName}`}>
          <span className="inline-flex items-center justify-center size-4.5">
            {SelectedIcon ? (
              <SelectedIcon className="h-3.5 w-3.5" style={{ color: selected.color }} />
            ) : (
              <Bot className="h-3.5 w-3.5" style={{ color: selected.color }} />
            )}
          </span>
          <span className="text-sm" style={{ color: selected.color }}>
            {selected.displayName}
          </span>
          <Pencil className="h-3 w-3 text-muted-foreground opacity-0 group-hover:opacity-55 group-data-[state=open]:opacity-55 ml-0.5" />
        </button>
      </PopoverTrigger>

      <PopoverContent align="start" className="w-[360px] p-1 overflow-hidden">
        <div
          className="absolute -top-8 -right-6 h-20 w-20 rounded-full blur-xl pointer-events-none"
          style={{ background: `radial-gradient(circle, ${selected.color}33 0%, transparent 70%)` }}
        />
        <div className="px-2 py-1.5 text-[11px] uppercase tracking-wider text-muted-foreground">Switch Discipline</div>
        <div className="grid grid-cols-2 gap-1">
          {disciplines.map(discipline => {
            const Icon = discipline.icon
            const isCurrent = discipline.name === value

            return (
              <button
                key={discipline.name}
                type="button"
                onClick={() => {
                  setOpen(false)
                  onSelect(discipline.name)
                }}
                className={cn(
                  'w-full relative overflow-hidden flex items-center gap-2 rounded-md px-2 py-1.5 text-left min-w-0',
                  'hover:bg-muted/70 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring'
                )}
                title={discipline.displayName}>
                <span
                  className="absolute -top-10 -right-10 h-28 w-28 rounded-full blur-lg pointer-events-none"
                  style={{ background: `radial-gradient(circle, ${discipline.color}88 0%, transparent 74%)` }}
                />
                <DisciplineAvatar
                  name={discipline.name}
                  color={discipline.color}
                  faceCrop={discipline.crops?.face}
                  Icon={Icon}
                  className="size-6"
                />
                <div className="min-w-0 flex-1">
                  <div className="text-sm truncate" style={{ color: discipline.color }}>
                    {discipline.displayName}
                  </div>
                  <div className="text-[11px] text-muted-foreground truncate">{discipline.acronym}</div>
                </div>
                {isCurrent && <Check className="h-3.5 w-3.5" style={{ color: discipline.color }} />}
              </button>
            )
          })}
        </div>
      </PopoverContent>
    </Popover>
  )
}
