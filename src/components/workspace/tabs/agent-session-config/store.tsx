import { createContext, type ReactNode, useContext, useRef } from 'react'
import { useStore } from 'zustand'
import { createStore, type StoreApi } from 'zustand/vanilla'
import type { AgentSessionLaunchConfig } from '@/hooks/preferences'
import type { TerminalBridgeModelOption } from '@/types/generated'

type AgentSessionConfigStoreState = AgentSessionLaunchConfig & {
  models: TerminalBridgeModelOption[]
  loadingModels: boolean
  error: string | null
  setAgent: (agent: AgentSessionLaunchConfig['agent']) => void
  setModel: (model: string) => void
  setEffort: (effort: AgentSessionLaunchConfig['effort']) => void
  setThinking: (thinking: boolean) => void
  setPermissionLevel: (permissionLevel: AgentSessionLaunchConfig['permissionLevel']) => void
  setModels: (models: TerminalBridgeModelOption[]) => void
  setLoadingModels: (loading: boolean) => void
  setError: (error: string | null) => void
}

function createAgentSessionConfigStore(initial: AgentSessionLaunchConfig) {
  return createStore<AgentSessionConfigStoreState>()(set => ({
    ...initial,
    models: [],
    loadingModels: true,
    error: null,
    setAgent: agent => set({ agent }),
    setModel: model => set({ model }),
    setEffort: effort => set({ effort }),
    setThinking: thinking => set({ thinking }),
    setPermissionLevel: permissionLevel => set({ permissionLevel }),
    setModels: models => set({ models }),
    setLoadingModels: loadingModels => set({ loadingModels }),
    setError: error => set({ error })
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
