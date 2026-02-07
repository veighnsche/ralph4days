import { invoke } from '@tauri-apps/api/core'
import { useEffect, useState } from 'react'
import { toast } from 'sonner'
import type { SectionBlock, SectionConfigWire } from './useSectionConfiguration'

interface CustomRecipeWire {
  name: string
  baseRecipe: string | null
  sections: SectionConfigWire[]
}

export function useRecipeManagement(
  open: boolean,
  sectionMeta: { length: number },
  sections: SectionBlock[],
  loadRecipeSections: (promptType: string) => Promise<boolean>,
  loadCustomSections: (configs: SectionConfigWire[]) => void
) {
  const [baseRecipe, setBaseRecipe] = useState('braindump')
  const [recipeName, setRecipeName] = useState<string | null>(null)
  const [customRecipeNames, setCustomRecipeNames] = useState<string[]>([])
  const [saveDialogOpen, setSaveDialogOpen] = useState(false)
  const [saveNameInput, setSaveNameInput] = useState('')

  useEffect(() => {
    if (!open) return
    invoke<string[]>('list_saved_recipes').then(setCustomRecipeNames).catch(console.error)
  }, [open])

  useEffect(() => {
    if (open && sectionMeta.length > 0) {
      loadRecipeSections(baseRecipe)
      setRecipeName(null)
    }
  }, [open, sectionMeta.length, baseRecipe])

  const handleRecipeChange = async (value: string) => {
    if (customRecipeNames.includes(value)) {
      try {
        const custom = await invoke<CustomRecipeWire>('load_saved_recipe', { name: value })
        setBaseRecipe(custom.baseRecipe ?? 'braindump')
        setRecipeName(custom.name)
        loadCustomSections(custom.sections)
      } catch (err) {
        toast.error(`Failed to load recipe: ${err}`)
      }
    } else {
      setBaseRecipe(value)
      await loadRecipeSections(value)
      setRecipeName(null)
    }
  }

  const handleSave = async () => {
    if (!recipeName) {
      setSaveDialogOpen(true)
      return
    }
    await doSave(recipeName)
  }

  const doSave = async (name: string) => {
    try {
      const wireSections: SectionConfigWire[] = sections.map(s => ({
        name: s.name,
        enabled: s.enabled,
        instructionOverride: s.instructionOverride
      }))
      await invoke('save_recipe', {
        recipe: { name, baseRecipe, sections: wireSections }
      })
      setRecipeName(name)
      setSaveDialogOpen(false)
      const names = await invoke<string[]>('list_saved_recipes')
      setCustomRecipeNames(names)
      toast.success(`Recipe "${name}" saved`)
    } catch (err) {
      toast.error(`Failed to save: ${err}`)
    }
  }

  const handleDelete = async () => {
    if (!recipeName) return
    try {
      await invoke('delete_recipe', { name: recipeName })
      const names = await invoke<string[]>('list_saved_recipes')
      setCustomRecipeNames(names)
      toast.success(`Recipe "${recipeName}" deleted`)
      setRecipeName(null)
      loadRecipeSections(baseRecipe)
    } catch (err) {
      toast.error(`Failed to delete: ${err}`)
    }
  }

  const currentPickerValue = recipeName ?? baseRecipe

  return {
    baseRecipe,
    recipeName,
    customRecipeNames,
    saveDialogOpen,
    setSaveDialogOpen,
    saveNameInput,
    setSaveNameInput,
    currentPickerValue,
    handleRecipeChange,
    handleSave,
    doSave,
    handleDelete
  }
}
