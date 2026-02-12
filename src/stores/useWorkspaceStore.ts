import type { LucideIcon } from 'lucide-react'
import type { ComponentType } from 'react'
import { create } from 'zustand'
import { MAX_TABS } from '@/constants/workspace'
import type { DisciplineConfig } from '@/hooks/disciplines'
import type { FeatureData as Feature, Task, TerminalBridgeModelOption } from '@/types/generated'

export type TabType = 'terminal' | 'agent-session-config' | 'task-detail' | 'feature-detail' | 'discipline-detail'

export interface WorkspaceTabLifecycle {
  onMount?: (tab: WorkspaceTab) => void
  onUnmount?: (tab: WorkspaceTab) => void
  onActivate?: (tab: WorkspaceTab) => void
  onDeactivate?: (tab: WorkspaceTab) => void
}

export interface WorkspaceTab {
  id: string
  type: TabType
  component: ComponentType<{ tab: WorkspaceTab }>
  title: string
  icon?: LucideIcon
  closeable: boolean
  lifecycle: WorkspaceTabLifecycle
  data?: {
    mode?: 'create' | 'edit'
    entityId?: number | string
    entity?: Task | Feature | DisciplineConfig
    sessionId?: string // For output tabs
    agent?: string // For terminal tabs (claude, codex)
    model?: string // For terminal tabs (haiku, sonnet, opus)
    effort?: 'low' | 'medium' | 'high' // Model capability: optional effort level
    thinking?: boolean // For terminal tabs (extended thinking)
    permissionLevel?: 'safe' | 'balanced' | 'auto' | 'full_auto' // Canonical launch permission level
    taskId?: number // For task execution terminals
    initPrompt?: string // Optional prompt captured at session start
    formTreeByAgent?: Record<string, TerminalBridgeModelOption[]>
    formTreeLoading?: boolean
    formTreeError?: string | null
  }
}

interface WorkspaceStore {
  tabs: WorkspaceTab[]
  activeTabId: string
  openTab: (tab: Omit<WorkspaceTab, 'id' | 'lifecycle'> & { id?: string; lifecycle?: WorkspaceTabLifecycle }) => string
  openTabAfter: (
    afterTabId: string,
    tab: Omit<WorkspaceTab, 'id' | 'lifecycle'> & { id?: string; lifecycle?: WorkspaceTabLifecycle }
  ) => string
  setTabData: (tabId: string, data: Partial<NonNullable<WorkspaceTab['data']>>) => void
  closeTab: (tabId: string) => void
  switchTab: (tabId: string) => void
  closeAllExcept: (tabId: string) => void
  closeAll: () => void
  closeToRight: (tabId: string) => void
  reorderTabs: (fromIndex: number, toIndex: number) => void
  // WHY: Tab content updates title/icon via this method (browser pattern, not parent-driven)
  setTabMeta: (tabId: string, meta: { title?: string; icon?: LucideIcon }) => void
}

export const NOOP_TAB_LIFECYCLE: WorkspaceTabLifecycle = {
  onMount: () => {},
  onUnmount: () => {},
  onActivate: () => {},
  onDeactivate: () => {}
}

function generateTabId(
  tab: Omit<WorkspaceTab, 'id' | 'lifecycle'> & { id?: string; lifecycle?: WorkspaceTabLifecycle }
): string {
  if (tab.id) return tab.id
  const entityId = tab.data?.entityId
  const mode = tab.data?.mode
  if (mode) return `${tab.type}-${mode}${entityId != null ? `-${entityId}` : ''}`
  if (entityId != null) return `${tab.type}-${entityId}`
  return `${tab.type}-${Date.now()}`
}

export const useWorkspaceStore = create<WorkspaceStore>((set, get) => ({
  tabs: [],
  activeTabId: '',

  openTab: tabInput => {
    const id = generateTabId(tabInput)
    const { tabs } = get()

    const existing = tabs.find(t => t.id === id)
    if (existing) {
      set({ activeTabId: id })
      return id
    }

    const tab: WorkspaceTab = { ...tabInput, id, lifecycle: tabInput.lifecycle ?? NOOP_TAB_LIFECYCLE }

    let nextTabs = [...tabs, tab]
    while (nextTabs.length > MAX_TABS) {
      const oldest = nextTabs.find(t => t.closeable)
      if (!oldest) break
      nextTabs = nextTabs.filter(t => t.id !== oldest.id)
    }

    set({ tabs: nextTabs, activeTabId: id })
    return id
  },

  openTabAfter: (afterTabId, tabInput) => {
    const id = generateTabId(tabInput)
    const { tabs } = get()

    const existing = tabs.find(t => t.id === id)
    if (existing) {
      set({ activeTabId: id })
      return id
    }

    const afterIndex = tabs.findIndex(t => t.id === afterTabId)
    if (afterIndex === -1) {
      return get().openTab(tabInput)
    }

    const tab: WorkspaceTab = { ...tabInput, id, lifecycle: tabInput.lifecycle ?? NOOP_TAB_LIFECYCLE }
    let nextTabs = [...tabs.slice(0, afterIndex + 1), tab, ...tabs.slice(afterIndex + 1)]

    while (nextTabs.length > MAX_TABS) {
      const oldest = nextTabs.find(t => t.closeable && t.id !== id)
      if (!oldest) break
      nextTabs = nextTabs.filter(t => t.id !== oldest.id)
    }

    set({ tabs: nextTabs, activeTabId: id })
    return id
  },

  setTabData: (tabId, data) => {
    const { tabs } = get()
    const tab = tabs.find(t => t.id === tabId)
    if (!tab) return
    set({
      tabs: tabs.map(t => {
        if (t.id !== tabId) return t
        return {
          ...t,
          data: {
            ...(t.data ?? {}),
            ...data
          }
        }
      })
    })
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

    set({ tabs: nextTabs, activeTabId: nextActive })
  },

  switchTab: tabId => {
    set({ activeTabId: tabId })
  },

  closeAllExcept: tabId => {
    const { tabs } = get()
    set({
      tabs: tabs.filter(t => t.id === tabId || !t.closeable),
      activeTabId: tabId
    })
  },

  closeAll: () => {
    const { tabs } = get()
    const nextTabs = tabs.filter(t => !t.closeable)
    set({
      tabs: nextTabs,
      activeTabId: nextTabs[0]?.id ?? ''
    })
  },

  closeToRight: tabId => {
    const { tabs } = get()
    const targetIndex = tabs.findIndex(t => t.id === tabId)
    if (targetIndex === -1) return

    const nextTabs = tabs.filter((t, i) => i <= targetIndex || !t.closeable)
    set({ tabs: nextTabs })
  },

  reorderTabs: (fromIndex, toIndex) => {
    const { tabs } = get()
    if (fromIndex === toIndex || fromIndex < 0 || toIndex < 0 || fromIndex >= tabs.length || toIndex >= tabs.length)
      return

    const nextTabs = [...tabs]
    const [movedTab] = nextTabs.splice(fromIndex, 1)
    nextTabs.splice(toIndex, 0, movedTab)
    set({ tabs: nextTabs })
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
