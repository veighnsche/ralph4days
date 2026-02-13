import { createContext, type ReactNode, useContext } from 'react'
import type { TabType, WorkspaceTab } from '@/stores/useWorkspaceStore'

interface WorkspaceTabContextValue {
  tab: WorkspaceTab
  isActive: boolean
}

const WorkspaceTabContext = createContext<WorkspaceTabContextValue | null>(null)

export function WorkspaceTabProvider({
  tab,
  isActive,
  children
}: {
  tab: WorkspaceTab
  isActive: boolean
  children: ReactNode
}) {
  return <WorkspaceTabContext.Provider value={{ tab, isActive }}>{children}</WorkspaceTabContext.Provider>
}

export function useWorkspaceTabContext() {
  const context = useContext(WorkspaceTabContext)
  if (!context) {
    throw new Error('useWorkspaceTabContext must be used inside WorkspaceTabProvider')
  }
  return context.tab
}

export function useWorkspaceTabIsActive() {
  const context = useContext(WorkspaceTabContext)
  if (!context) {
    throw new Error('useWorkspaceTabIsActive must be used inside WorkspaceTabProvider')
  }
  return context.isActive
}

export function useWorkspaceTabOfType<T extends TabType>(type: T): WorkspaceTab {
  const tab = useWorkspaceTabContext()
  if (tab.type !== type) {
    throw new Error(`Workspace tab type mismatch: expected '${type}', got '${tab.type}'`)
  }
  return tab
}

export function useWorkspaceTabData<T>(selector: (tab: WorkspaceTab) => T): T {
  return selector(useWorkspaceTabContext())
}
