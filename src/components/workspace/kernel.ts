import type { WorkspaceTab } from '@/stores/useWorkspaceStore'

export type WorkspaceLifecycleHook = 'onMount' | 'onUnmount' | 'onActivate' | 'onDeactivate'

export interface WorkspaceKernelLifecycleEvent {
  hook: WorkspaceLifecycleHook
  tab: WorkspaceTab
}

export interface WorkspaceKernelSnapshot {
  tabs: WorkspaceTab[]
  tabsById: Map<string, WorkspaceTab>
  activeTabId: string
  mountedTabIds: Set<string>
}

export interface WorkspaceKernelMountPlan {
  activeTab: WorkspaceTab | null
  inactiveKeepAliveTabs: WorkspaceTab[]
}

export function buildWorkspaceKernelSnapshot(
  tabs: WorkspaceTab[],
  activeTabId: string,
  keepAliveOnDeactivate: (tabType: string) => boolean
): WorkspaceKernelSnapshot {
  const tabsById = new Map(tabs.map(tab => [tab.id, tab]))
  const mountedTabIds = new Set<string>()

  for (const tab of tabs) {
    if (tab.id === activeTabId || keepAliveOnDeactivate(tab.type)) {
      mountedTabIds.add(tab.id)
    }
  }

  return {
    tabs,
    tabsById,
    activeTabId,
    mountedTabIds
  }
}

export function buildWorkspaceMountPlan(
  tabs: WorkspaceTab[],
  activeTabId: string,
  keepAliveOnDeactivate: (tabType: string) => boolean
): WorkspaceKernelMountPlan {
  const activeTab = tabs.find(tab => tab.id === activeTabId) ?? null
  const inactiveKeepAliveTabs = tabs.filter(tab => tab.id !== activeTabId && keepAliveOnDeactivate(tab.type))

  return { activeTab, inactiveKeepAliveTabs }
}

export function computeWorkspaceLifecycleEvents(
  previous: WorkspaceKernelSnapshot,
  next: WorkspaceKernelSnapshot
): WorkspaceKernelLifecycleEvent[] {
  const events: WorkspaceKernelLifecycleEvent[] = []
  const activeChanged = previous.activeTabId !== next.activeTabId

  if (activeChanged) {
    const previousActive = previous.tabsById.get(previous.activeTabId)
    if (previousActive) {
      events.push({ hook: 'onDeactivate', tab: previousActive })
    }
  }

  for (const tab of previous.tabs) {
    if (previous.mountedTabIds.has(tab.id) && !next.mountedTabIds.has(tab.id)) {
      events.push({ hook: 'onUnmount', tab })
    }
  }

  for (const tab of next.tabs) {
    if (!previous.mountedTabIds.has(tab.id) && next.mountedTabIds.has(tab.id)) {
      events.push({ hook: 'onMount', tab })
    }
  }

  if (activeChanged) {
    const nextActive = next.tabsById.get(next.activeTabId)
    if (nextActive) {
      events.push({ hook: 'onActivate', tab: nextActive })
    }
  }

  return events
}
