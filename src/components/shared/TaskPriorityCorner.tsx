import { cn } from '@/lib/utils'
import { PriorityIcon } from './PriorityIcon'
import { PriorityRadial } from './PriorityRadial'

export function TaskPriorityCorner({
  priority,
  size = 'sm',
  className
}: {
  priority?: string | null
  size?: 'sm' | 'md' | 'lg'
  className?: string
}) {
  if (!priority) return null

  return (
    <>
      <PriorityRadial priority={priority} />
      <div className={cn('absolute top-3 right-3 z-20 pointer-events-none', className)}>
        <PriorityIcon priority={priority} size={size} />
      </div>
    </>
  )
}
