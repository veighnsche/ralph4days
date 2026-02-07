import type { DragEndEvent } from '@dnd-kit/core'
import { useState } from 'react'
import { getDefaultRecipeConfig, SECTION_REGISTRY } from '@/lib/recipe-registry'

export interface SectionBlock {
  name: string
  displayName: string
  description: string
  category: string
  isInstruction: boolean
  enabled: boolean
  instructionOverride: string | null | undefined
}

function configsToBlocks(
  configs: { name: string; enabled: boolean; instructionOverride?: string | null }[]
): SectionBlock[] {
  return configs.map(cfg => {
    const meta = SECTION_REGISTRY.find(m => m.name === cfg.name)
    return {
      name: cfg.name,
      displayName: meta?.displayName ?? cfg.name,
      description: meta?.description ?? '',
      category: meta?.category ?? 'unknown',
      isInstruction: meta?.isInstruction ?? false,
      enabled: cfg.enabled,
      instructionOverride: cfg.instructionOverride
    }
  })
}

export function useSectionConfiguration(_open: boolean) {
  const [sections, setSections] = useState<SectionBlock[]>([])
  const [loadError, setLoadError] = useState<string | null>(null)

  const loadRecipeSections = async (promptType: string) => {
    try {
      const config = getDefaultRecipeConfig(promptType)
      const blocks = config.sectionOrder.map(name => {
        const meta = SECTION_REGISTRY.find(m => m.name === name)
        return {
          name,
          displayName: meta?.displayName ?? name,
          description: meta?.description ?? '',
          category: meta?.category ?? 'unknown',
          isInstruction: meta?.isInstruction ?? false,
          enabled: config.sections[name]?.enabled ?? false,
          instructionOverride: undefined as string | null | undefined
        }
      })
      setSections(blocks)
      setLoadError(null)
      return true
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err)
      setLoadError(`Failed to load recipe sections: ${message}`)
      return false
    }
  }

  const loadCustomSections = (configs: { name: string; enabled: boolean; instructionOverride?: string | null }[]) => {
    setSections(configsToBlocks(configs))
  }

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event
    if (!over || active.id === over.id) return

    setSections(prev => {
      const oldIndex = prev.findIndex(s => s.name === active.id)
      const newIndex = prev.findIndex(s => s.name === over.id)
      if (oldIndex === -1 || newIndex === -1) return prev

      const next = [...prev]
      const [moved] = next.splice(oldIndex, 1)
      next.splice(newIndex, 0, moved)
      return next
    })
  }

  const toggleSection = (name: string) => {
    setSections(prev => prev.map(s => (s.name === name ? { ...s, enabled: !s.enabled } : s)))
  }

  const commitInstructionOverride = (name: string, text: string | null) => {
    setSections(prev => prev.map(s => (s.name === name ? { ...s, instructionOverride: text } : s)))
  }

  const enabledCount = sections.filter(s => s.enabled).length

  return {
    sections,
    enabledCount,
    loadRecipeSections,
    loadCustomSections,
    handleDragEnd,
    toggleSection,
    commitInstructionOverride,
    loadError,
    resetLoadError: () => setLoadError(null)
  }
}
