import { invoke } from '@tauri-apps/api/core'
import { useEffect, useRef, useState } from 'react'
import { QUERY_KEYS } from '@/constants/cache'
import { useInvoke, useInvokeMutation } from '@/hooks/api'
import { SECTION_REGISTRY } from '@/lib/recipe-registry'
import type { RecipeConfigData, RecipeConfigInput } from '@/types/generated'
import type { SectionBlock } from './useSectionConfiguration'

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
  const [loadError, setLoadError] = useState<string | null>(null)
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
    invalidateKeys: QUERY_KEYS.RECIPE_LIST,
    onSuccess: (_data, variables) => {
      setRecipeName(variables.config.name)
      setSaveDialogOpen(false)
    }
  })

  const deleteMutation = useInvokeMutation<{ name: string }>('delete_recipe_config', {
    invalidateKeys: QUERY_KEYS.RECIPE_LIST,
    onSuccess: () => {
      setRecipeName(null)
      loadRecipeSections(baseRecipe)
    }
  })

  const loadCustomRecipe = async (name: string, gen: number) => {
    const data = await invoke<RecipeConfigData>('get_recipe_config', { name })
    if (gen !== recipeChangeGenRef.current) return
    if (!data) {
      setLoadError(`Recipe "${name}" not found`)
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
        setLoadError(`Failed to load recipe: ${err}`)
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
  const error = saveMutation.error ?? deleteMutation.error ?? loadError
  const resetError = () => {
    saveMutation.reset()
    deleteMutation.reset()
    setLoadError(null)
  }

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
    handleDelete,
    error,
    resetError
  }
}
