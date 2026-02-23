import type * as React from 'react'
import { cn } from '@/lib/utils'

interface MobileScrollPageProps extends React.ComponentProps<'div'> {
  includeBounceSentinel?: boolean
}

function MobileScrollPage({
  className,
  children,
  includeBounceSentinel = false,
  ...props
}: MobileScrollPageProps) {
  return (
    <div
      className={cn('h-dvh overflow-y-scroll bg-background', className)}
      style={{ WebkitOverflowScrolling: 'touch' }}
      {...props}>
      {children}
      {includeBounceSentinel ? <div aria-hidden className="h-px w-full" /> : null}
    </div>
  )
}

export { MobileScrollPage }
