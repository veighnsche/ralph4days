import { invoke } from '@tauri-apps/api/core'
import { useEffect, useRef, useState } from 'react'
import { toast } from 'sonner'
import { SECTION_REGISTRY } from '@/lib/recipe-registry'
import type { RecipeConfigData, RecipeConfigInput } from '@/types/generated'
import { useInvoke } from './useInvoke'
import { useInvokeMutation } from './useInvokeMutation'
import type { SectionBlock } from './useSectionConfiguration'

const RECIPE_LIST_KEY = [['list_recipe_configs']]

export function useRecipeManagement(
  open: boolean,
  sections: SectionBlock[],
  loadRecipeSections: (promptType: string) => Promise<boolean>,
  loadCustomSections: (configs: { name: string; enabled: boolean; instructionOverride?: string | null }[]) => void
) {
  const [baseRecipe, setBaseRecipe] = useState('braindump')
  const [recipeName, setRecipeName] = useState<string | null>(null)
  const [saveDialogOpen, setSaveDialogOpen] = useState(false)
  const [saveNameInput, setSaveNameInput] = useState('')
  const recipeChangeGenRef = useRef(0)

  const { data: customRecipeNames = [] } = useInvoke<string[]>('list_recipe_configs', undefined, { enabled: open })

  const initializedRef = useRef(false)
  // biome-ignore lint/correctness/useExhaustiveDependencies: baseRecipe and loadRecipeSections intentionally excluded — recipe switching is handled by handleRecipeChange, not this init effect
  useEffect(() => {
    if (!open) {
      initializedRef.current = false
      return
    }
    if (!initializedRef.current) {
      initializedRef.current = true
      loadRecipeSections(baseRecipe)
    }
  }, [open])

  const saveMutation = useInvokeMutation<{ config: RecipeConfigInput }>('save_recipe_config', {
    invalidateKeys: RECIPE_LIST_KEY,
    onSuccess: (_data, variables) => {
      setRecipeName(variables.config.name)
      setSaveDialogOpen(false)
      toast.success(`Recipe "${variables.config.name}" saved`)
    },
    onError: err => toast.error(`Failed to save: ${err.message}`)
  })

  const deleteMutation = useInvokeMutation<{ name: string }>('delete_recipe_config', {
    invalidateKeys: RECIPE_LIST_KEY,
    onSuccess: (_data, variables) => {
      toast.success(`Recipe "${variables.name}" deleted`)
      setRecipeName(null)
      loadRecipeSections(baseRecipe)
    },
    onError: err => toast.error(`Failed to delete: ${err.message}`)
  })

  const loadCustomRecipe = async (name: string, gen: number) => {
    const data = await invoke<RecipeConfigData>('get_recipe_config', { name })
    if (gen !== recipeChangeGenRef.current) return
    if (!data) {
      toast.error(`Recipe "${name}" not found`)
      return
    }
    setBaseRecipe(data.baseRecipe)
    setRecipeName(data.name)
    const configs = data.sectionOrder.map(sectionName => {
      const settings = data.sections[sectionName]
      const meta = SECTION_REGISTRY.find(m => m.name === sectionName)
      return {
        name: sectionName,
        enabled: settings?.enabled ?? false,
        instructionOverride: settings?.instructionOverride ?? null,
        displayName: meta?.displayName ?? sectionName,
        description: meta?.description ?? '',
        category: meta?.category ?? 'unknown',
        isInstruction: meta?.isInstruction ?? false
      }
    })
    loadCustomSections(configs)
  }

  const handleRecipeChange = async (value: string) => {
    const gen = ++recipeChangeGenRef.current
    if (customRecipeNames.includes(value)) {
      try {
        await loadCustomRecipe(value, gen)
      } catch (err) {
        if (gen !== recipeChangeGenRef.current) return
        toast.error(`Failed to load recipe: ${err}`)
      }
    } else {
      setBaseRecipe(value)
      await loadRecipeSections(value)
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
    const config: RecipeConfigInput = {
      name,
      baseRecipe: baseRecipe,
      sectionOrder: sections.map(s => s.name),
      sections: Object.fromEntries(
        sections.map(s => [
          s.name,
          {
            enabled: s.enabled,
            instructionOverride: s.instructionOverride ?? undefined
          }
        ])
      )
    }
    saveMutation.mutate({ config })
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
