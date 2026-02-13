import { invoke } from '@tauri-apps/api/core'
import { useEffect, useRef, useState } from 'react'
import { useDebouncedCallback } from 'use-debounce'
import type { PromptPreview, SectionConfig } from '@/types/generated'
import type { SectionBlock } from './useSectionConfiguration'

export type { PromptPreview }

function rebuildPreviewWithUserInput(base: PromptPreview, userInput: string, sections: SectionBlock[]): PromptPreview {
  if (!userInput.trim()) {
    return base
  }

  const userInputSection = {
    name: 'user_input',
    content: `## User's Input\n\n${userInput}`
  }

  const userInputIndex = sections.findIndex(s => s.name === 'user_input' && s.enabled)

  if (userInputIndex === -1) {
    return base
  }

  const newSections = [...base.sections]
  newSections.splice(userInputIndex, 0, userInputSection)

  const fullPrompt = newSections.map(s => s.content).join('\n\n')

  return {
    sections: newSections,
    fullPrompt
  }
}

export function usePromptPreview(open: boolean, sections: SectionBlock[]) {
  const [basePreview, setBasePreview] = useState<PromptPreview | null>(null)
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
      const result = await invoke<PromptPreview>('preview_custom_prompt_builder', {
        sections: wireSections,
        userInput: null
      })
      if (requestId !== latestPreviewRequestRef.current) {
        return
      }
      setBasePreview(result)
      setPreviewError(null)
      setPreview(rebuildPreviewWithUserInput(result, userInputRef.current, currentSections))
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
      setBasePreview(null)
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
    if (basePreview) {
      setPreview(rebuildPreviewWithUserInput(basePreview, value, sections))
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
