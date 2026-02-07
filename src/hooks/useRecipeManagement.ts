import { invoke } from '@tauri-apps/api/core'
import { useEffect, useRef, useState } from 'react'
import { toast } from 'sonner'
import { useInvoke } from './useInvoke'
import { useInvokeMutation } from './useInvokeMutation'
import type { SectionBlock, SectionConfigWire, SectionMeta } from './useSectionConfiguration'

interface CustomRecipeWire {
  name: string
  baseRecipe: string | null
  sections: SectionConfigWire[]
}

const RECIPE_LIST_KEY = [['list_saved_recipes']]

export function useRecipeManagement(
  open: boolean,
  sectionMeta: SectionMeta[],
  sections: SectionBlock[],
  loadRecipeSections: (promptType: string, meta: SectionMeta[]) => Promise<boolean>,
  loadCustomSections: (configs: SectionConfigWire[], meta: SectionMeta[]) => void
) {
  const [baseRecipe, setBaseRecipe] = useState('braindump')
  const [recipeName, setRecipeName] = useState<string | null>(null)
  const [saveDialogOpen, setSaveDialogOpen] = useState(false)
  const [saveNameInput, setSaveNameInput] = useState('')

  const { data: customRecipeNames = [] } = useInvoke<string[]>('list_saved_recipes', undefined, { enabled: open })

  const initializedRef = useRef(false)
  // biome-ignore lint/correctness/useExhaustiveDependencies: baseRecipe and loadRecipeSections intentionally excluded — recipe switching is handled by handleRecipeChange, not this init effect
  useEffect(() => {
    if (!open) {
      initializedRef.current = false
      return
    }
    if (sectionMeta.length > 0 && !initializedRef.current) {
      initializedRef.current = true
      loadRecipeSections(baseRecipe, sectionMeta)
    }
  }, [open, sectionMeta])

  const saveMutation = useInvokeMutation<{ recipe: CustomRecipeWire }>('save_recipe', {
    invalidateKeys: RECIPE_LIST_KEY,
    onSuccess: (_data, variables) => {
      setRecipeName(variables.recipe.name)
      setSaveDialogOpen(false)
      toast.success(`Recipe "${variables.recipe.name}" saved`)
    },
    onError: err => toast.error(`Failed to save: ${err.message}`)
  })

  const deleteMutation = useInvokeMutation<{ name: string }>('delete_recipe', {
    invalidateKeys: RECIPE_LIST_KEY,
    onSuccess: (_data, variables) => {
      toast.success(`Recipe "${variables.name}" deleted`)
      setRecipeName(null)
      loadRecipeSections(baseRecipe, sectionMeta)
    },
    onError: err => toast.error(`Failed to delete: ${err.message}`)
  })

  const handleRecipeChange = async (value: string) => {
    if (customRecipeNames.includes(value)) {
      try {
        const custom = await invoke<CustomRecipeWire>('load_saved_recipe', { name: value })
        setBaseRecipe(custom.baseRecipe ?? 'braindump')
        setRecipeName(custom.name)
        loadCustomSections(custom.sections, sectionMeta)
      } catch (err) {
        toast.error(`Failed to load recipe: ${err}`)
      }
    } else {
      setBaseRecipe(value)
      await loadRecipeSections(value, sectionMeta)
      setRecipeName(null)
    }
  }

  const handleSave = () => {
    if (!recipeName) {
      setSaveDialogOpen(true)
      return
    }
    doSave(recipeName)
  }

  const doSave = (name: string) => {
    const wireSections: SectionConfigWire[] = sections.map(s => ({
      name: s.name,
      enabled: s.enabled,
      instructionOverride: s.instructionOverride
    }))
    saveMutation.mutate({ recipe: { name, baseRecipe, sections: wireSections } })
  }

  const handleDelete = () => {
    if (!recipeName) return
    deleteMutation.mutate({ name: recipeName })
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
