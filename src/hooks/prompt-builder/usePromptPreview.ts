import { useEffect, useRef, useState } from 'react'
import { useDebouncedCallback } from 'use-debounce'
import { tauriInvoke } from '@/lib/tauri/invoke'
import type { PromptPreview, SectionConfig } from '@/types/generated'
import type { SectionBlock } from './useSectionConfiguration'

export type { PromptPreview }

export function usePromptPreview(open: boolean, sections: SectionBlock[]) {
  const [preview, setPreview] = useState<PromptPreview | null>(null)
  const [previewError, setPreviewError] = useState<string | null>(null)
  const userInputRef = useRef('')
  const latestPreviewRequestRef = useRef(0)

  const fetchPreview = useDebouncedCallback(async (requestId: number, currentSections: SectionBlock[]) => {
    try {
      const wireSections: SectionConfig[] = currentSections.map(s => ({
        name: s.name,
        enabled: s.enabled,
        instructionOverride: s.instructionOverride ?? undefined
      }))
      const userInput = userInputRef.current.trim() ? userInputRef.current : undefined
      const result = await tauriInvoke<PromptPreview>('prompt_builder_preview', {
        sections: wireSections,
        userInput
      })
      if (requestId !== latestPreviewRequestRef.current) {
        return
      }
      setPreviewError(null)
      setPreview(result)
    } catch (err) {
      if (requestId !== latestPreviewRequestRef.current) {
        return
      }
      const message = err instanceof Error ? err.message : String(err)
      setPreviewError(`Failed to preview: ${message}`)
    }
  }, 500)

  useEffect(() => {
    if (!open) {
      fetchPreview.cancel()
      latestPreviewRequestRef.current += 1
      userInputRef.current = ''
      setPreview(null)
      return
    }
  }, [fetchPreview, open])

  useEffect(() => {
    if (!open || sections.length === 0) return
    const requestId = latestPreviewRequestRef.current + 1
    latestPreviewRequestRef.current = requestId
    fetchPreview(requestId, sections)
    return () => {
      fetchPreview.cancel()
    }
  }, [fetchPreview, open, sections])

  const handleUserInputChange = (value: string) => {
    userInputRef.current = value
    if (open && sections.length > 0) {
      const requestId = latestPreviewRequestRef.current + 1
      latestPreviewRequestRef.current = requestId
      fetchPreview(requestId, sections)
    }
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
