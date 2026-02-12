import { create } from 'zustand'
import { createJSONStorage, persist } from 'zustand/middleware'

const VALID_AGENTS = ['claude', 'codex'] as const
export type Agent = (typeof VALID_AGENTS)[number]
export type Model = string
export type Effort = 'low' | 'medium' | 'high'
export type PermissionLevel = 'safe' | 'balanced' | 'auto' | 'full_auto'
export type AgentSessionLaunchConfig = {
  agent: Agent
  model: Model
  effort: Effort
  thinking: boolean
  permissionLevel: PermissionLevel
}

const DEFAULT_MODELS_BY_AGENT: Record<Agent, string> = {
  claude: 'claude-sonnet-4',
  codex: 'gpt-5-codex'
}

function getDefaultModel(agent: Agent) {
  return DEFAULT_MODELS_BY_AGENT[agent]
}

type AgentSessionLaunchPreferencesStore = AgentSessionLaunchConfig & {
  setAgent: (value: Agent) => void
  setModel: (value: Model) => void
  setEffort: (value: Effort) => void
  setThinking: (value: boolean) => void
  setPermissionLevel: (value: PermissionLevel) => void
  getDefaultModel: (agent: Agent) => Model
}

export const useAgentSessionLaunchPreferences = create<AgentSessionLaunchPreferencesStore>()(
  persist(
    set => ({
      agent: 'claude',
      model: getDefaultModel('claude'),
      effort: 'medium',
      thinking: true,
      permissionLevel: 'balanced',
      setAgent: value => set({ agent: value }),
      setModel: value => {
        if (!value.trim()) return
        set({ model: value })
      },
      setEffort: value => set({ effort: value }),
      setThinking: value => set({ thinking: value }),
      setPermissionLevel: value => set({ permissionLevel: value }),
      getDefaultModel
    }),
    {
      name: 'ralph.preferences.agent-session-launch',
      storage: createJSONStorage(() => localStorage),
      partialize: state => ({
        agent: state.agent,
        model: state.model,
        effort: state.effort,
        thinking: state.thinking,
        permissionLevel: state.permissionLevel
      })
    }
  )
)
