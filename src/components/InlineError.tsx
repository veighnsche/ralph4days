import { AlertCircle, X } from 'lucide-react'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'

export function InlineError({
  error,
  onDismiss,
  className
}: {
  error: Error | string | null
  onDismiss?: () => void
  className?: string
}) {
  if (!error) return null
  const message = error instanceof Error ? error.message : error

  return (
    <Alert variant="destructive" className={className}>
      <AlertCircle className="h-4 w-4" />
      <AlertDescription className="flex items-center gap-2">
        <span className="flex-1 text-xs">{message}</span>
        {onDismiss && (
          <Button variant="ghost" size="sm" className="h-5 w-5 p-0 flex-shrink-0" onClick={onDismiss}>
            <X className="h-3 w-3" />
          </Button>
        )}
      </AlertDescription>
    </Alert>
  )
}
