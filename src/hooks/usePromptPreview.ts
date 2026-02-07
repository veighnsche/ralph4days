import { invoke } from '@tauri-apps/api/core'
import { useEffect, useRef, useState } from 'react'
import type { PromptPreview, SectionConfig } from '@/types/generated'
import type { SectionBlock } from './useSectionConfiguration'

export type { PromptPreview }

export function usePromptPreview(open: boolean, sections: SectionBlock[]) {
  const [preview, setPreview] = useState<PromptPreview | null>(null)
  const [previewTrigger, setPreviewTrigger] = useState(0)
  const [previewError, setPreviewError] = useState<string | null>(null)
  const userInputRef = useRef('')

  useEffect(() => {
    if (!open) {
      userInputRef.current = ''
      return
    }
  }, [open])

  // biome-ignore lint/correctness/useExhaustiveDependencies: previewTrigger is an intentional re-fire signal for userInputRef changes without making every keystroke a state update
  useEffect(() => {
    if (!open || sections.length === 0) return
    let ignore = false
    const timer = setTimeout(async () => {
      try {
        const wireSections: SectionConfig[] = sections.map(s => ({
          name: s.name,
          enabled: s.enabled,
          instructionOverride: s.instructionOverride ?? undefined
        }))
        const result = await invoke<PromptPreview>('preview_custom_recipe', {
          sections: wireSections,
          userInput: userInputRef.current || null
        })
        if (!ignore) {
          setPreview(result)
          setPreviewError(null)
        }
      } catch (err) {
        if (!ignore) {
          const message = err instanceof Error ? err.message : String(err)
          setPreviewError(`Failed to preview: ${message}`)
        }
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

  const handleCopy = async () => {
    if (!preview?.fullPrompt) return
    try {
      await navigator.clipboard.writeText(preview.fullPrompt)
      setPreviewError(null)
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err)
      setPreviewError(`Failed to copy: ${message}`)
    }
  }

  return { preview, handleUserInputChange, handleCopy, previewError, resetPreviewError: () => setPreviewError(null) }
}
