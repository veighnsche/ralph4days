import { Send } from 'lucide-react'
import { useState } from 'react'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'

export function SignalEditor({
  value,
  onChange,
  onSubmit,
  onCancel,
  submitLabel,
  placeholder,
  disabled,
  autoFocus
}: {
  value: string
  onChange: (value: string) => void
  onSubmit: () => void
  onCancel?: () => void
  submitLabel: string
  placeholder?: string
  disabled?: boolean
  autoFocus?: boolean
}) {
  const [focused, setFocused] = useState(false)
  const canSubmit = value.trim().length > 0 && !disabled
  const collapsed = !(focused || value.trim())
  const hints = [onCancel && 'Esc to cancel', 'Ctrl+Enter to submit'].filter(Boolean).join(' · ')

  return (
    <div
      className={`rounded-md border bg-muted/30 overflow-hidden transition-opacity ${collapsed ? 'opacity-30' : ''}`}>
      <Textarea
        value={value}
        onChange={e => onChange(e.target.value)}
        onFocus={() => setFocused(true)}
        onBlur={() => setFocused(false)}
        onKeyDown={e => {
          if (e.key === 'Enter' && (e.ctrlKey || e.metaKey) && canSubmit) {
            e.preventDefault()
            onSubmit()
          }
          if (e.key === 'Escape' && onCancel) onCancel()
        }}
        placeholder={placeholder}
        className={`text-sm resize-none border-0 shadow-none bg-transparent rounded-none focus-visible:ring-0 ${collapsed ? 'min-h-0 h-8 py-1.5' : 'min-h-[48px]'}`}
        autoFocus={autoFocus}
      />
      {!collapsed && (
        <div className="flex items-center justify-between border-t bg-muted/40 px-2.5 py-1.5">
          <span className="text-xs text-muted-foreground">{value.trim() ? hints : ''}</span>
          <div className="flex gap-1.5">
            {onCancel && (
              <Button variant="ghost" size="sm" className="h-6 px-2 text-xs" onClick={onCancel}>
                Cancel
              </Button>
            )}
            <Button size="sm" className="h-6 px-2 text-xs gap-1.5" onClick={onSubmit} disabled={!canSubmit}>
              <Send className="h-3 w-3" />
              {submitLabel}
            </Button>
          </div>
        </div>
      )}
    </div>
  )
}
