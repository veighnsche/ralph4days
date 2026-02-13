import { createContext, type ReactNode, useContext, useRef } from 'react'
import { useStore } from 'zustand'
import { createStore, type StoreApi } from 'zustand/vanilla'
import type { AgentSessionLaunchConfig } from '@/lib/agent-session-launch-config'

export type AgentSessionConfigStoreState = AgentSessionLaunchConfig & {
  setAgent: (agent: AgentSessionLaunchConfig['agent']) => void
  setModel: (model: string) => void
  setEffort: (effort: AgentSessionLaunchConfig['effort']) => void
  setThinking: (thinking: boolean) => void
  setPermissionLevel: (permissionLevel: AgentSessionLaunchConfig['permissionLevel']) => void
}

export function createAgentSessionConfigStore(initial: AgentSessionLaunchConfig) {
  return createStore<AgentSessionConfigStoreState>()(set => ({
    ...initial,
    setAgent: agent =>
      set(state => {
        if (state.agent === agent) return state
        return { agent }
      }),
    setModel: model =>
      set(state => {
        if (state.model === model) return state
        return { model }
      }),
    setEffort: effort =>
      set(state => {
        if (state.effort === effort) return state
        return { effort }
      }),
    setThinking: thinking =>
      set(state => {
        if (state.thinking === thinking) return state
        return { thinking }
      }),
    setPermissionLevel: permissionLevel =>
      set(state => {
        if (state.permissionLevel === permissionLevel) return state
        return { permissionLevel }
      })
  }))
}

const AgentSessionConfigStoreContext = createContext<StoreApi<AgentSessionConfigStoreState> | null>(null)

export function AgentSessionConfigStoreProvider({
  initialConfig,
  children
}: {
  initialConfig: AgentSessionLaunchConfig
  children: ReactNode
}) {
  const storeRef = useRef<StoreApi<AgentSessionConfigStoreState> | null>(null)
  if (!storeRef.current) {
    storeRef.current = createAgentSessionConfigStore(initialConfig)
  }
  return (
    <AgentSessionConfigStoreContext.Provider value={storeRef.current}>
      {children}
    </AgentSessionConfigStoreContext.Provider>
  )
}

export function useAgentSessionConfigStore<T>(selector: (state: AgentSessionConfigStoreState) => T): T {
  const store = useContext(AgentSessionConfigStoreContext)
  if (!store) {
    throw new Error('useAgentSessionConfigStore must be used within AgentSessionConfigStoreProvider')
  }
  return useStore(store, selector)
}
