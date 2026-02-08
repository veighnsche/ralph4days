import { AlertCircle, Check, Copy, X } from 'lucide-react'
import { useState } from 'react'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'

function parseErrorCode(message: string): { code: string | null; text: string } {
  const match = message.match(/^\[R-(\d{4})\] (.*)$/s)
  if (match) return { code: `R-${match[1]}`, text: match[2] }
  return { code: null, text: message }
}

function formatError(message: string): string {
  const { code, text } = parseErrorCode(message)
  const prefix = code ? `[Ralph Error ${code}]` : '[Ralph Error]'
  return `${prefix} ${new Date().toISOString()}\n${text}`
}

export function InlineError({
  error,
  onDismiss,
  className
}: {
  error: Error | string | null
  onDismiss?: () => void
  className?: string
}) {
  const [copied, setCopied] = useState(false)

  if (!error) return null
  const message = error instanceof Error ? error.message : error
  const { code, text } = parseErrorCode(message)

  function handleCopy() {
    navigator.clipboard.writeText(formatError(message))
    setCopied(true)
    setTimeout(() => setCopied(false), 1500)
  }

  return (
    <Alert variant="destructive" className={className}>
      <AlertCircle className="h-4 w-4" />
      <AlertDescription className="flex items-center gap-2">
        {code && <span className="font-mono text-[10px] opacity-60 flex-shrink-0">{code}</span>}
        <span className="flex-1 text-xs">{text}</span>
        <Button variant="ghost" size="sm" className="h-5 w-5 p-0 flex-shrink-0" onClick={handleCopy}>
          {copied ? <Check className="h-3 w-3" /> : <Copy className="h-3 w-3" />}
        </Button>
        {onDismiss && (
          <Button variant="ghost" size="sm" className="h-5 w-5 p-0 flex-shrink-0" onClick={onDismiss}>
            <X className="h-3 w-3" />
          </Button>
        )}
      </AlertDescription>
    </Alert>
  )
}
