import type * as React from 'react'
import { cn } from '@/lib/utils'

function DottedList({ className, ...props }: React.ComponentProps<'ul'>) {
  return <ul data-slot="dotted-list" className={cn('space-y-1 text-xs', className)} {...props} />
}

function DottedListItem({ className, ...props }: React.ComponentProps<'li'>) {
  return (
    <li
      data-slot="dotted-list-item"
      className={cn('ml-4 list-disc text-muted-foreground marker:text-muted-foreground', className)}
      {...props}
    />
  )
}

export { DottedList, DottedListItem }
