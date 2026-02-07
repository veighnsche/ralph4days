import { invoke } from '@tauri-apps/api/core'
import { useEffect, useRef, useState } from 'react'
import { toast } from 'sonner'
import type { SectionBlock, SectionConfigWire } from './useSectionConfiguration'

interface PromptPreviewSection {
  name: string
  content: string
}

export interface PromptPreview {
  sections: PromptPreviewSection[]
  fullPrompt: string
}

export function usePromptPreview(open: boolean, sections: SectionBlock[]) {
  const [preview, setPreview] = useState<PromptPreview | null>(null)
  const [previewTrigger, setPreviewTrigger] = useState(0)
  const userInputRef = useRef('')

  // biome-ignore lint/correctness/useExhaustiveDependencies: previewTrigger is an intentional re-fire signal for userInputRef changes without making every keystroke a state update
  useEffect(() => {
    if (!open || sections.length === 0) return
    let ignore = false
    const timer = setTimeout(async () => {
      try {
        const wireSections: SectionConfigWire[] = sections.map(s => ({
          name: s.name,
          enabled: s.enabled,
          instructionOverride: s.instructionOverride
        }))
        const result = await invoke<PromptPreview>('preview_custom_recipe', {
          sections: wireSections,
          userInput: userInputRef.current || null
        })
        if (!ignore) setPreview(result)
      } catch (err) {
        if (!ignore) console.error('Failed to preview:', err)
      }
    }, 500)
    return () => {
      ignore = true
      clearTimeout(timer)
    }
  }, [open, sections, previewTrigger])

  const handleUserInputChange = (value: string) => {
    userInputRef.current = value
    setPreviewTrigger(n => n + 1)
  }

  const handleCopy = () => {
    if (preview?.fullPrompt) {
      navigator.clipboard.writeText(preview.fullPrompt)
      toast.success('Copied to clipboard')
    }
  }

  return { preview, handleUserInputChange, handleCopy }
}
