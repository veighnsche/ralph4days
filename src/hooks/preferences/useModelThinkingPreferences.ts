import { useState } from 'react'
import { STORAGE_KEYS } from '@/constants/storage'

const VALID_MODELS = ['haiku', 'sonnet', 'opus'] as const
export type Model = (typeof VALID_MODELS)[number]

function isValidModel(value: string | null): value is Model {
  return VALID_MODELS.includes(value as Model)
}

export function useModelThinkingPreferences() {
  const [model, setModelState] = useState<Model>(() => {
    const saved = localStorage.getItem(STORAGE_KEYS.MODEL)
    return isValidModel(saved) ? saved : 'sonnet'
  })

  const [thinking, setThinkingState] = useState(() => {
    const saved = localStorage.getItem(STORAGE_KEYS.THINKING)
    return saved === 'true'
  })

  const setModel = (value: Model) => {
    setModelState(value)
    if (isValidModel(value)) {
      localStorage.setItem(STORAGE_KEYS.MODEL, value)
    }
  }

  const setThinking = (value: boolean) => {
    setThinkingState(value)
    localStorage.setItem(STORAGE_KEYS.THINKING, String(value))
  }

  return {
    model,
    setModel,
    thinking,
    setThinking
  }
}
