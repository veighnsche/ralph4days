import { useRef } from 'react'
import { createDefaultTerminalTab } from '@/components/workspace/tabs'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'

export interface WorkspaceTabsActions {
  switchTab: (tabId: string) => void
  closeTab: (tabId: string) => void
  closeAll: () => void
  closeOthers: (tabId: string) => void
  closeToRight: (tabId: string) => void
  newTabToRight: (afterTabId: string) => void
  reorderTabs: (fromIndex: number, toIndex: number) => void
}

export function useWorkspaceTabsActions(): WorkspaceTabsActions {
  const switchTab = useWorkspaceStore(s => s.switchTab)
  const closeTab = useWorkspaceStore(s => s.closeTab)
  const closeAll = useWorkspaceStore(s => s.closeAll)
  const closeAllExcept = useWorkspaceStore(s => s.closeAllExcept)
  const closeToRight = useWorkspaceStore(s => s.closeToRight)
  const openTabAfter = useWorkspaceStore(s => s.openTabAfter)
  const reorderTabs = useWorkspaceStore(s => s.reorderTabs)

  const actionsRef = useRef<WorkspaceTabsActions | null>(null)
  if (!actionsRef.current) {
    actionsRef.current = {
      switchTab,
      closeTab,
      closeAll,
      closeOthers: closeAllExcept,
      closeToRight,
      newTabToRight: (afterTabId: string) => {
        openTabAfter(afterTabId, createDefaultTerminalTab())
      },
      reorderTabs
    }
  }
  return actionsRef.current
}
