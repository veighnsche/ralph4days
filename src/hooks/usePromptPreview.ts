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
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const userInputRef = useRef('')

  const schedulePreview = (currentSections: SectionBlock[]) => {
    if (!open || currentSections.length === 0) return
    if (debounceRef.current) clearTimeout(debounceRef.current)
    debounceRef.current = setTimeout(async () => {
      try {
        const wireSections: SectionConfigWire[] = currentSections.map(s => ({
          name: s.name,
          enabled: s.enabled,
          instructionOverride: s.instructionOverride
        }))
        const result = await invoke<PromptPreview>('preview_custom_recipe', {
          sections: wireSections,
          userInput: userInputRef.current || null
        })
        setPreview(result)
      } catch (err) {
        console.error('Failed to preview:', err)
      }
    }, 500)
  }

  useEffect(() => {
    schedulePreview(sections)
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current)
    }
  }, [sections])

  const handleUserInputChange = (value: string) => {
    userInputRef.current = value
    schedulePreview(sections)
  }

  const handleCopy = () => {
    if (preview?.fullPrompt) {
      navigator.clipboard.writeText(preview.fullPrompt)
      toast.success('Copied to clipboard')
    }
  }

  return { preview, handleUserInputChange, handleCopy }
}
