import { useState } from 'react'
import { STORAGE_KEYS } from '@/constants/storage'

const VALID_AGENTS = ['claude', 'codex'] as const
export type Agent = (typeof VALID_AGENTS)[number]
export type Model = string

const DEFAULT_MODELS_BY_AGENT: Record<Agent, string> = {
  claude: 'claude-sonnet-4',
  codex: 'gpt-5-codex'
}

function isValidAgent(value: string | null): value is Agent {
  return VALID_AGENTS.includes(value as Agent)
}

function getDefaultModel(agent: Agent) {
  return DEFAULT_MODELS_BY_AGENT[agent]
}

export function useModelThinkingPreferences() {
  const [agent, setAgentState] = useState<Agent>(() => {
    const saved = localStorage.getItem(STORAGE_KEYS.AGENT)
    return isValidAgent(saved) ? saved : 'claude'
  })

  const [model, setModelState] = useState<Model>(() => {
    const saved = localStorage.getItem(STORAGE_KEYS.MODEL)
    if (saved?.trim()) return saved
    const savedAgent = localStorage.getItem(STORAGE_KEYS.AGENT)
    const defaultAgent = isValidAgent(savedAgent) ? savedAgent : 'claude'
    return getDefaultModel(defaultAgent)
  })

  const [thinking, setThinkingState] = useState(() => {
    const saved = localStorage.getItem(STORAGE_KEYS.THINKING)
    return saved === 'true'
  })

  const setAgent = (value: Agent) => {
    setAgentState(value)
    localStorage.setItem(STORAGE_KEYS.AGENT, value)
  }

  const setModel = (value: Model) => {
    setModelState(value)
    if (value.trim()) {
      localStorage.setItem(STORAGE_KEYS.MODEL, value)
    }
  }

  const setThinking = (value: boolean) => {
    setThinkingState(value)
    localStorage.setItem(STORAGE_KEYS.THINKING, String(value))
  }

  return {
    agent,
    setAgent,
    model,
    setModel,
    thinking,
    setThinking,
    getDefaultModel
  }
}
