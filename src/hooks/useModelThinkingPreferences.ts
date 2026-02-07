import { useCallback, useState } from 'react'

const STORAGE_KEY_MODEL = 'ralph.preferences.model'
const STORAGE_KEY_THINKING = 'ralph.preferences.thinking'

const VALID_MODELS = ['haiku', 'sonnet', 'opus'] as const
export type Model = (typeof VALID_MODELS)[number]

function isValidModel(value: string | null): value is Model {
  return VALID_MODELS.includes(value as Model)
}

export function useModelThinkingPreferences() {
  const [model, setModelState] = useState<Model>(() => {
    const saved = localStorage.getItem(STORAGE_KEY_MODEL)
    return isValidModel(saved) ? saved : 'sonnet'
  })

  const [thinking, setThinkingState] = useState(() => {
    const saved = localStorage.getItem(STORAGE_KEY_THINKING)
    return saved === 'true'
  })

  const setModel = useCallback((value: Model) => {
    setModelState(value)
    if (isValidModel(value)) {
      localStorage.setItem(STORAGE_KEY_MODEL, value)
    }
  }, [])

  const setThinking = useCallback((value: boolean) => {
    setThinkingState(value)
    localStorage.setItem(STORAGE_KEY_THINKING, String(value))
  }, [])

  return {
    model,
    setModel,
    thinking,
    setThinking
  }
}
