import type { DragEndEvent } from '@dnd-kit/core'
import { invoke } from '@tauri-apps/api/core'
import { useEffect, useState } from 'react'

export interface SectionMeta {
  name: string
  display_name: string
  description: string
  category: string
  is_instruction: boolean
}

export interface SectionBlock {
  name: string
  displayName: string
  description: string
  category: string
  isInstruction: boolean
  enabled: boolean
  instructionOverride: string | null
}

export interface SectionConfigWire {
  name: string
  enabled: boolean
  instructionOverride: string | null
}

function configsToBlocks(configs: SectionConfigWire[], sectionMeta: SectionMeta[]): SectionBlock[] {
  return configs.map(cfg => {
    const meta = sectionMeta.find(m => m.name === cfg.name)
    return {
      name: cfg.name,
      displayName: meta?.display_name ?? cfg.name,
      description: meta?.description ?? '',
      category: meta?.category ?? 'unknown',
      isInstruction: meta?.is_instruction ?? false,
      enabled: cfg.enabled,
      instructionOverride: cfg.instructionOverride
    }
  })
}

export function useSectionConfiguration(open: boolean) {
  const [sections, setSections] = useState<SectionBlock[]>([])
  const [sectionMeta, setSectionMeta] = useState<SectionMeta[]>([])

  useEffect(() => {
    if (!open) return
    invoke<SectionMeta[]>('get_section_metadata').then(setSectionMeta).catch(console.error)
  }, [open])

  const loadRecipeSections = async (promptType: string) => {
    try {
      const configs = await invoke<SectionConfigWire[]>('get_recipe_sections', { promptType })
      setSections(configsToBlocks(configs, sectionMeta))
      return true
    } catch (err) {
      console.error('Failed to load recipe sections:', err)
      return false
    }
  }

  const loadCustomSections = (configs: SectionConfigWire[]) => {
    setSections(configsToBlocks(configs, sectionMeta))
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
    sectionMeta,
    enabledCount,
    loadRecipeSections,
    loadCustomSections,
    handleDragEnd,
    toggleSection,
    commitInstructionOverride
  }
}
