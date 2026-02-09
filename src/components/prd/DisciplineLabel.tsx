import { cn } from '@/lib/utils'

interface DisciplineLabelProps {
  acronym: string
  color: string
  className?: string
}

export function DisciplineLabel({ acronym, color, className }: DisciplineLabelProps) {
  if (import.meta.env.DEV && acronym.length !== 4) {
    throw new Error(`DisciplineLabel: acronym must be exactly 4 characters, got "${acronym}" (${acronym.length})`)
  }

  return (
    <span className={cn('font-mono text-xs font-medium', className)} style={{ color }}>
      {acronym}
    </span>
  )
}
