import { cn } from '@/lib/utils'

interface FormHeaderProps extends React.ComponentProps<'div'> {}

/**
 * Form header wrapper - provides consistent spacing and structure for form titles
 */
function FormHeader({ className, ...props }: FormHeaderProps) {
  return <div className={cn('flex flex-col gap-2 pt-3', className)} {...props} />
}

interface FormTitleProps extends React.ComponentProps<'h2'> {}

/**
 * Form title - consistent styling for form headings
 */
function FormTitle({ className, ...props }: FormTitleProps) {
  return <h2 className={cn('text-lg font-semibold leading-none', className)} {...props} />
}

interface FormDescriptionProps extends React.ComponentProps<'p'> {}

/**
 * Form description - consistent styling for form descriptions
 */
function FormDescription({ className, ...props }: FormDescriptionProps) {
  return <p className={cn('text-sm text-muted-foreground', className)} {...props} />
}

export { FormHeader, FormTitle, FormDescription }
