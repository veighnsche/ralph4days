import { TerminalTabContent } from '@/components/workspace'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'

export interface BrowserTabsActions {
  switchTab: (tabId: string) => void
  closeTab: (tabId: string) => void
  closeAll: () => void
  closeOthers: (tabId: string) => void
  closeToRight: (tabId: string) => void
  newTabToRight: (afterTabId: string) => void
  reorderTabs: (fromIndex: number, toIndex: number) => void
}

export function useBrowserTabsActions(): BrowserTabsActions {
  const switchTab = useWorkspaceStore(s => s.switchTab)
  const closeTab = useWorkspaceStore(s => s.closeTab)
  const closeAll = useWorkspaceStore(s => s.closeAll)
  const closeAllExcept = useWorkspaceStore(s => s.closeAllExcept)
  const closeToRight = useWorkspaceStore(s => s.closeToRight)
  const openTabAfter = useWorkspaceStore(s => s.openTabAfter)
  const reorderTabs = useWorkspaceStore(s => s.reorderTabs)

  const newTabToRight = (afterTabId: string) => {
    openTabAfter(afterTabId, {
      type: 'terminal',
      component: TerminalTabContent,
      title: 'New Terminal',
      closeable: true
    })
  }

  return {
    switchTab,
    closeTab,
    closeAll,
    closeOthers: closeAllExcept,
    closeToRight,
    newTabToRight,
    reorderTabs
  }
}
