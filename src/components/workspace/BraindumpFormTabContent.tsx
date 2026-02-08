// TODO: Wire up MCP server generation & database sync
// TODO: Add context prompt wrapper with tool instructions
// TODO: Listen for database changes & invalidate cache
// TODO: Handle terminal close gracefully with background option
// TODO: Show progress indicator & creation summary

import { invoke } from '@tauri-apps/api/core'
import { Brain } from 'lucide-react'
import { useState } from 'react'
import { toast } from 'sonner'
import { type Model, ModelThinkingPicker } from '@/components/model-thinking'
import { InlineError } from '@/components/shared'
import { Button } from '@/components/ui/button'
import { FormDescription, FormHeader, FormTitle } from '@/components/ui/form-header'
import { Label } from '@/components/ui/label'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { Textarea } from '@/components/ui/textarea'
import { useTabMeta, useWorkspaceActions } from '@/hooks/workspace'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'

const DEFAULT_QUESTIONS = `What is this project about?

What problem does it solve?

Who will use this?

What are the main features you're thinking about?

What's the tech stack (if you know)?

Any constraints or special requirements?

What's your timeline or priorities?`

interface BraindumpFormTabContentProps {
  tab: WorkspaceTab
}

export function BraindumpFormTabContent({ tab }: BraindumpFormTabContentProps) {
  useTabMeta(tab.id, 'Braindump', Brain)
  const closeTab = useWorkspaceStore(s => s.closeTab)
  const { openTerminalTab } = useWorkspaceActions()
  const [braindump, setBraindump] = useState(DEFAULT_QUESTIONS)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const sendToTerminal = async (terminalId: string, text: string) => {
    await new Promise(resolve => setTimeout(resolve, 1000))

    const terminalExists = useWorkspaceStore.getState().tabs.some(t => t.id === terminalId)
    if (!terminalExists) throw new Error('Terminal tab was closed before sending')

    const bytes = Array.from(new TextEncoder().encode(`${text}\n`))
    await invoke('send_terminal_input', { sessionId: terminalId, data: bytes })

    closeTab(tab.id)
    toast.success('Braindump sent to Claude')
  }

  const handleSubmit = async (model: Model, thinking: boolean) => {
    setError(null)
    const trimmedBraindump = braindump.trim()
    if (!trimmedBraindump) {
      setError('Please enter some text before sending')
      return
    }
    if (isSubmitting) return

    setIsSubmitting(true)

    try {
      const terminalId = openTerminalTab(model, thinking)
      await sendToTerminal(terminalId, trimmedBraindump)
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error'
      setError(`Failed to send braindump: ${errorMessage}`)
      setIsSubmitting(false)
    }
  }

  const handleCancel = () => {
    closeTab(tab.id)
  }

  return (
    <div className="flex h-full flex-col">
      <ScrollArea className="flex-1">
        <form className="px-4 space-y-4">
          <FormHeader>
            <FormTitle>Braindump Your Project</FormTitle>
            <FormDescription>
              Answer these questions in your own words. Claude will help structure this into features and tasks.
            </FormDescription>
          </FormHeader>

          <Separator />

          <div className="space-y-2">
            <Label htmlFor="braindump">Your thoughts (edit the questions as you like)</Label>
            <Textarea
              id="braindump"
              value={braindump}
              onChange={e => setBraindump(e.target.value)}
              className="min-h-[400px] font-mono text-sm"
              placeholder="Start typing your thoughts..."
            />
            <p className="text-xs text-muted-foreground">
              Tip: Be casual and conversational. Claude understands context and can work with messy notes.
            </p>
          </div>
        </form>
      </ScrollArea>

      <Separator />

      {error && (
        <div className="px-3 pt-1.5">
          <InlineError error={error} onDismiss={() => setError(null)} />
        </div>
      )}

      <div className="px-3 py-1.5 flex items-center justify-end gap-2">
        <Button type="button" variant="outline" size="default" onClick={handleCancel}>
          Cancel
        </Button>

        <ModelThinkingPicker
          onAction={handleSubmit}
          actionLabel={isSubmitting ? 'Sending...' : 'Send to Claude'}
          disabled={isSubmitting || !braindump.trim()}
        />
      </div>
    </div>
  )
}
