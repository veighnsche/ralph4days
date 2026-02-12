import { cva, type VariantProps } from 'class-variance-authority'
import type * as React from 'react'
import { cn } from '@/lib/utils'

const selectableCardVariants = cva(
  'relative border text-left transition-colors duration-100 cursor-pointer disabled:cursor-not-allowed disabled:opacity-50',
  {
    variants: {
      selected: {
        true: 'border-primary bg-primary/5 shadow-[inset_0_0_0_1px_hsl(var(--primary)/0.2)]',
        false: 'hover:bg-accent'
      },
      radius: {
        default: 'rounded-md',
        lg: 'rounded-lg'
      },
      variant: {
        default: '',
        icon: 'inline-flex items-center justify-center h-9 w-9 p-0'
      }
    },
    defaultVariants: {
      selected: false,
      radius: 'default',
      variant: 'default'
    }
  }
)

type SelectableCardProps = React.ComponentProps<'button'> &
  VariantProps<typeof selectableCardVariants> & {
    selected?: boolean
  }

export function SelectableCard({
  className,
  selected,
  radius,
  variant,
  type = 'button',
  ...props
}: SelectableCardProps) {
  return (
    <button
      type={type}
      data-selected={selected ? 'true' : 'false'}
      className={cn(selectableCardVariants({ selected, radius, variant }), className)}
      {...props}
    />
  )
}
