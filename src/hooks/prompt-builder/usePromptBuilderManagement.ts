import { useEffect, useRef, useState } from 'react'
import { QUERY_KEYS } from '@/constants/cache'
import { useInvoke, useInvokeMutation } from '@/hooks/api'
import { SECTION_REGISTRY } from '@/lib/prompt-builder-registry'
import { tauriInvoke } from '@/lib/tauri/invoke'
import type { PromptBuilderConfigData, PromptBuilderConfigInput } from '@/types/generated'
import type { SectionBlock } from './useSectionConfiguration'

const EMPTY_CUSTOM_PROMPT_BUILDER_NAMES: string[] = []

export function usePromptBuilderManagement(
  open: boolean,
  sections: SectionBlock[],
  loadPromptBuilderSections: (promptType: string) => Promise<boolean>,
  loadCustomSections: (configs: { name: string; enabled: boolean; instructionOverride?: string | null }[]) => void
) {
  const [basePrompt, setBasePrompt] = useState('braindump')
  const [promptBuilderName, setPromptBuilderName] = useState<string | null>(null)
  const [saveDialogOpen, setSaveDialogOpen] = useState(false)
  const [saveNameInput, setSaveNameInput] = useState('')
  const [loadError, setLoadError] = useState<string | null>(null)
  const promptBuilderChangeGenRef = useRef(0)

  const { data: customPromptBuilderNamesData } = useInvoke<string[]>('prompt_builder_config_list', undefined, {
    enabled: open
  })
  const customPromptBuilderNames = customPromptBuilderNamesData ?? EMPTY_CUSTOM_PROMPT_BUILDER_NAMES

  const initializedRef = useRef(false)
  // biome-ignore lint/correctness/useExhaustiveDependencies: basePrompt and loadPromptBuilderSections intentionally excluded — picker switching is handled by handlePromptBuilderChange, not this init effect
  useEffect(() => {
    if (!open) {
      initializedRef.current = false
      return
    }
    if (!initializedRef.current) {
      initializedRef.current = true
      loadPromptBuilderSections(basePrompt)
    }
  }, [open])

  const saveMutation = useInvokeMutation<{ config: PromptBuilderConfigInput }>('prompt_builder_config_save', {
    invalidateKeys: QUERY_KEYS.PROMPT_BUILDER_LIST,
    onSuccess: (_data, variables) => {
      setPromptBuilderName(variables.config.name)
      setSaveDialogOpen(false)
    }
  })

  const deleteMutation = useInvokeMutation<{ name: string }>('prompt_builder_config_delete', {
    invalidateKeys: QUERY_KEYS.PROMPT_BUILDER_LIST,
    onSuccess: () => {
      setPromptBuilderName(null)
      loadPromptBuilderSections(basePrompt)
    }
  })

  const loadCustomPromptBuilder = async (name: string, gen: number) => {
    const data = await tauriInvoke<PromptBuilderConfigData | null>('prompt_builder_config_get', { name })
    if (gen !== promptBuilderChangeGenRef.current) return
    if (!data) {
      setLoadError(`Prompt builder config "${name}" not found`)
      return
    }
    setBasePrompt(data.basePrompt)
    setPromptBuilderName(data.name)
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

  const handlePromptBuilderChange = async (value: string) => {
    const gen = ++promptBuilderChangeGenRef.current
    if (customPromptBuilderNames.includes(value)) {
      try {
        await loadCustomPromptBuilder(value, gen)
      } catch (err) {
        if (gen !== promptBuilderChangeGenRef.current) return
        setLoadError(`Failed to load prompt builder config: ${err}`)
      }
    } else {
      setBasePrompt(value)
      await loadPromptBuilderSections(value)
      setPromptBuilderName(null)
    }
  }

  const handleSave = () => {
    if (!promptBuilderName) {
      setSaveDialogOpen(true)
      return
    }
    doSave(promptBuilderName)
  }

  const doSave = (name: string) => {
    const config: PromptBuilderConfigInput = {
      name,
      basePrompt: basePrompt,
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
    if (!promptBuilderName) return
    deleteMutation.mutate({ name: promptBuilderName })
  }

  const currentPickerValue = promptBuilderName ?? basePrompt
  const error = saveMutation.error ?? deleteMutation.error ?? loadError
  const resetError = () => {
    saveMutation.reset()
    deleteMutation.reset()
    setLoadError(null)
  }

  return {
    basePrompt,
    promptBuilderName,
    customPromptBuilderNames,
    saveDialogOpen,
    setSaveDialogOpen,
    saveNameInput,
    setSaveNameInput,
    currentPickerValue,
    handlePromptBuilderChange,
    handleSave,
    doSave,
    handleDelete,
    error,
    resetError
  }
}
