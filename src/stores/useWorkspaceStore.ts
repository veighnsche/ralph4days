import type { LucideIcon } from 'lucide-react'
import { create } from 'zustand'
import { MAX_TABS } from '@/constants/workspace'

export type TabType = string

export interface WorkspaceTab {
  id: string
  type: TabType
  title: string
  icon?: LucideIcon
  closeable: boolean
  key?: string
  params?: unknown
}

interface WorkspaceStore {
  tabs: WorkspaceTab[]
  activeTabId: string
  openTab: (tab: Omit<WorkspaceTab, 'id'> & { id?: string }) => string
  openTabAfter: (afterTabId: string, tab: Omit<WorkspaceTab, 'id'> & { id?: string }) => string
  closeTab: (tabId: string) => void
  switchTab: (tabId: string) => void
  closeAllExcept: (tabId: string) => void
  closeAll: () => void
  closeToRight: (tabId: string) => void
  reorderTabs: (fromIndex: number, toIndex: number) => void
  // WHY: Tab content updates title/icon via this method (declarative pattern, not parent-driven)
  setTabMeta: (tabId: string, meta: { title?: string; icon?: LucideIcon }) => void
}

function hasTabId(tabs: WorkspaceTab[], tabId: string): boolean {
  return tabs.some(tab => tab.id === tabId)
}

function ensureActiveTabId(tabs: WorkspaceTab[], candidate: string): string {
  if (tabs.length === 0) return ''
  if (candidate && hasTabId(tabs, candidate)) return candidate
  return tabs[0]?.id ?? ''
}

function setTabsAndActive(
  set: (updater: (state: WorkspaceStore) => WorkspaceStore | Partial<WorkspaceStore>) => void,
  nextTabs: WorkspaceTab[],
  activeCandidate: string
) {
  set(state => {
    const nextActiveTabId = ensureActiveTabId(nextTabs, activeCandidate)
    const tabsUnchanged =
      state.tabs.length === nextTabs.length && state.tabs.every((tab, index) => tab.id === nextTabs[index]?.id)
    if (tabsUnchanged && state.activeTabId === nextActiveTabId) return state
    return { tabs: nextTabs, activeTabId: nextActiveTabId }
  })
}

function generateTabId(tab: Omit<WorkspaceTab, 'id'> & { id?: string }): string {
  if (tab.id) return tab.id
  if (tab.key) return `${tab.type}-${tab.key}`
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return `${tab.type}-${crypto.randomUUID()}`
  }
  return `${tab.type}-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`
}

export const useWorkspaceStore = create<WorkspaceStore>((set, get) => ({
  tabs: [],
  activeTabId: '',

  openTab: tabInput => {
    const id = generateTabId(tabInput)
    const { tabs } = get()

    const existing = tabs.find(t => t.id === id)
    if (existing) {
      if (get().activeTabId !== id) {
        set({ activeTabId: id })
      }
      return id
    }

    const tab: WorkspaceTab = { ...tabInput, id }

    let nextTabs = [...tabs, tab]
    while (nextTabs.length > MAX_TABS) {
      const oldest = nextTabs.find(t => t.closeable)
      if (!oldest) break
      nextTabs = nextTabs.filter(t => t.id !== oldest.id)
    }

    setTabsAndActive(set, nextTabs, id)
    return id
  },

  openTabAfter: (afterTabId, tabInput) => {
    const id = generateTabId(tabInput)
    const { tabs } = get()

    const existing = tabs.find(t => t.id === id)
    if (existing) {
      if (get().activeTabId !== id) {
        set({ activeTabId: id })
      }
      return id
    }

    const afterIndex = tabs.findIndex(t => t.id === afterTabId)
    if (afterIndex === -1) {
      return get().openTab(tabInput)
    }

    const tab: WorkspaceTab = { ...tabInput, id }
    let nextTabs = [...tabs.slice(0, afterIndex + 1), tab, ...tabs.slice(afterIndex + 1)]

    while (nextTabs.length > MAX_TABS) {
      const oldest = nextTabs.find(t => t.closeable && t.id !== id)
      if (!oldest) break
      nextTabs = nextTabs.filter(t => t.id !== oldest.id)
    }

    setTabsAndActive(set, nextTabs, id)
    return id
  },

  closeTab: tabId => {
    const { tabs, activeTabId } = get()
    const tab = tabs.find(t => t.id === tabId)
    if (!tab?.closeable) return

    const nextTabs = tabs.filter(t => t.id !== tabId)
    let nextActive = activeTabId

    if (activeTabId === tabId) {
      const closedIndex = tabs.findIndex(t => t.id === tabId)
      const prev = tabs[closedIndex - 1] || tabs[closedIndex + 1]
      nextActive = prev?.id ?? ''
    }

    setTabsAndActive(set, nextTabs, nextActive)
  },

  switchTab: tabId => {
    set(state => {
      if (!hasTabId(state.tabs, tabId)) return state
      if (state.activeTabId === tabId) return state
      return { activeTabId: tabId }
    })
  },

  closeAllExcept: tabId => {
    const { tabs } = get()
    const nextTabs = tabs.filter(t => t.id === tabId || !t.closeable)
    setTabsAndActive(set, nextTabs, tabId)
  },

  closeAll: () => {
    const { tabs } = get()
    const nextTabs = tabs.filter(t => !t.closeable)
    setTabsAndActive(set, nextTabs, nextTabs[0]?.id ?? '')
  },

  closeToRight: tabId => {
    const { tabs, activeTabId } = get()
    const targetIndex = tabs.findIndex(t => t.id === tabId)
    if (targetIndex === -1) return

    const nextTabs = tabs.filter((t, i) => i <= targetIndex || !t.closeable)
    const nextActive = hasTabId(nextTabs, activeTabId) ? activeTabId : tabId
    setTabsAndActive(set, nextTabs, nextActive)
  },

  reorderTabs: (fromIndex, toIndex) => {
    const { tabs } = get()
    if (fromIndex === toIndex || fromIndex < 0 || toIndex < 0 || fromIndex >= tabs.length || toIndex >= tabs.length)
      return

    const nextTabs = [...tabs]
    const [movedTab] = nextTabs.splice(fromIndex, 1)
    nextTabs.splice(toIndex, 0, movedTab)
    set(state => {
      const tabsUnchanged =
        state.tabs.length === nextTabs.length && state.tabs.every((tab, index) => tab.id === nextTabs[index]?.id)
      if (tabsUnchanged) return state
      return { tabs: nextTabs }
    })
  },

  setTabMeta: (tabId, meta) => {
    const { tabs } = get()
    const tab = tabs.find(t => t.id === tabId)
    if (!tab) return
    const titleSame = meta.title === undefined || meta.title === tab.title
    const iconSame = meta.icon === undefined || meta.icon === tab.icon
    if (titleSame && iconSame) return
    set({
      tabs: tabs.map(t =>
        t.id === tabId
          ? {
              ...t,
              ...(meta.title !== undefined && { title: meta.title }),
              ...(meta.icon !== undefined && { icon: meta.icon })
            }
          : t
      )
    })
  }
}))
