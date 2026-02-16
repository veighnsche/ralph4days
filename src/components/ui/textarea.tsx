import type * as React from 'react'

import { cn } from '@/lib/utils'

function Textarea({ className, ...props }: React.ComponentProps<'textarea'>) {
  return (
    <textarea
      data-slot="textarea"
      className={cn(
        'border-input placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive dark:bg-input/30 flex field-sizing-content min-h-24 w-full rounded-[var(--mobile-control-radius)] border bg-transparent px-[var(--mobile-control-padding-inline)] py-[var(--mobile-control-padding-block)] text-base shadow-xs transition-[color,box-shadow] outline-none focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50 sm:min-h-16 sm:rounded-md sm:px-3 sm:py-2 sm:text-sm',
        className
      )}
      {...props}
    />
  )
}

export { Textarea }
