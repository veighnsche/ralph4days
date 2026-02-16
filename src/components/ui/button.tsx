import { cva, type VariantProps } from 'class-variance-authority'
import { Slot } from 'radix-ui'
import type * as React from 'react'

import { cn } from '@/lib/utils'

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-[color,box-shadow,transform,opacity] duration-150 disabled:pointer-events-none disabled:opacity-50 active:scale-[0.99] motion-reduce:transition-none motion-reduce:active:scale-100 [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4 shrink-0 [&_svg]:shrink-0 outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive",
  {
    variants: {
      variant: {
        default: 'bg-primary text-primary-foreground hover:bg-primary/90',
        destructive:
          'bg-destructive text-white hover:bg-destructive/90 focus-visible:ring-destructive/20 dark:focus-visible:ring-destructive/40 dark:bg-destructive/60',
        outline:
          'border bg-background shadow-xs hover:bg-accent hover:text-accent-foreground dark:bg-input/30 dark:border-input dark:hover:bg-input/50',
        secondary: 'bg-secondary text-secondary-foreground hover:bg-secondary/80',
        ghost: 'hover:bg-accent hover:text-accent-foreground dark:hover:bg-accent/50',
        link: 'text-primary underline-offset-4 hover:underline'
      },
      size: {
        default:
          'h-[var(--touch-min)] rounded-[var(--mobile-control-radius)] px-[var(--mobile-control-padding-inline)] py-[var(--mobile-control-padding-block)] text-sm has-[>svg]:px-3 sm:h-8 sm:rounded-md sm:px-3 sm:py-1.5 sm:has-[>svg]:px-2.5',
        xs: "h-9 gap-1 rounded-[var(--mobile-control-radius)] px-2.5 text-sm has-[>svg]:px-2 [&_svg:not([class*='size-'])]:size-3.5 sm:h-6 sm:rounded-md sm:px-2 sm:text-xs sm:has-[>svg]:px-1.5 sm:[&_svg:not([class*='size-'])]:size-3",
        sm: 'h-10 rounded-[var(--mobile-control-radius)] gap-1.5 px-3 text-sm has-[>svg]:px-2 sm:h-6 sm:rounded-md sm:gap-1 sm:px-2 sm:text-xs sm:has-[>svg]:px-1.5',
        lg: 'h-12 rounded-lg px-5 text-base has-[>svg]:px-4 sm:h-10 sm:rounded-md sm:px-4 sm:text-sm sm:has-[>svg]:px-3',
        icon: 'size-[var(--touch-min)] rounded-[var(--mobile-control-radius)] sm:size-8 sm:rounded-md',
        'icon-xs':
          "size-9 rounded-[var(--mobile-control-radius)] [&_svg:not([class*='size-'])]:size-3.5 sm:size-6 sm:rounded-md sm:[&_svg:not([class*='size-'])]:size-3",
        'icon-sm': 'size-10 rounded-[var(--mobile-control-radius)] sm:size-6 sm:rounded-md',
        'icon-lg': 'size-12 rounded-lg sm:size-10 sm:rounded-md'
      }
    },
    defaultVariants: {
      variant: 'default',
      size: 'default'
    }
  }
)

function Button({
  className,
  variant = 'default',
  size = 'default',
  asChild = false,
  ...props
}: React.ComponentProps<'button'> &
  VariantProps<typeof buttonVariants> & {
    asChild?: boolean
  }) {
  const Comp = asChild ? Slot.Root : 'button'

  return (
    <Comp
      data-slot="button"
      data-variant={variant}
      data-size={size}
      className={cn(buttonVariants({ variant, size, className }))}
      {...props}
    />
  )
}

export { Button, buttonVariants }
