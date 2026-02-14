import { Circle } from 'lucide-react'
import { cn } from '@/lib/utils'
import { PriorityIcon } from './PriorityIcon'
import { PriorityRadial } from './PriorityRadial'

export function TaskPriorityCorner({
  priority,
  size = 'sm',
  className,
  showUnset = false,
  onClick,
  disabled = false
}: {
  priority?: string | null
  size?: 'sm' | 'md' | 'lg'
  className?: string
  showUnset?: boolean
  onClick?: () => void
  disabled?: boolean
}) {
  const shouldRender = !!priority || showUnset
  if (!shouldRender) return null

  const isClickable = !!onClick && !disabled

  const renderIcon = () => {
    if (!priority) {
      const sizeClass = size === 'sm' ? 'h-3 w-3' : size === 'md' ? 'h-4 w-4' : 'h-5 w-5'
      return <Circle className={cn(sizeClass, 'text-muted-foreground')} />
    }
    return <PriorityIcon priority={priority} size={size} />
  }

  return (
    <>
      <PriorityRadial priority={priority} />
      <div
        className={cn(
          'absolute top-3 right-3 z-20',
          onClick ? 'pointer-events-auto' : 'pointer-events-none',
          className
        )}>
        {onClick ? (
          <button
            type="button"
            aria-label="Cycle priority"
            disabled={disabled}
            className={cn(
              'inline-flex items-center justify-center rounded-sm',
              'h-7 w-7',
              'transition-opacity',
              disabled ? 'opacity-50' : 'hover:opacity-80'
            )}
            onClick={() => {
              if (!isClickable) return
              onClick()
            }}>
            {renderIcon()}
          </button>
        ) : (
          renderIcon()
        )}
      </div>
    </>
  )
}
