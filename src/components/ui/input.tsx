import type * as React from 'react'

import { cn } from '@/lib/utils'

function Input({ className, type, ...props }: React.ComponentProps<'input'>) {
  return (
    <input
      type={type}
      data-slot="input"
      className={cn(
        'file:text-foreground placeholder:text-muted-foreground selection:bg-primary selection:text-primary-foreground dark:bg-input/30 border-input h-[var(--touch-min)] w-full min-w-0 rounded-[var(--mobile-control-radius)] border bg-transparent px-[var(--mobile-control-padding-inline)] py-[var(--mobile-control-padding-block)] text-base shadow-xs transition-[color,box-shadow,transform] duration-150 outline-none file:inline-flex file:h-9 file:border-0 file:bg-transparent file:text-base file:font-medium disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 sm:h-8 sm:rounded-md sm:px-3 sm:py-1 sm:text-sm sm:file:h-7 sm:file:text-sm',
        'focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px]',
        'aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive',
        className
      )}
      {...props}
    />
  )
}

export { Input }
