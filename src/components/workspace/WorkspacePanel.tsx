import { useEffect, useRef } from 'react'
import type { AgentSessionLaunchConfig } from '@/components/agent-session-launch'
import { AgentSessionLaunchButton } from '@/components/agent-session-launch'
import { Button } from '@/components/ui/button'
import {
  buildWorkspaceKernelSnapshot,
  buildWorkspaceMountPlan,
  computeWorkspaceLifecycleEvents
} from '@/components/workspace/kernel'
import {
  createAgentSessionConfigTab,
  createDefaultTerminalTab,
  createTerminalTabFromLaunch,
  getTabKeepAliveOnDeactivate,
  getTabLifecycle,
  WorkspaceTabContentHost
} from '@/components/workspace/tabs'
import { useBrowserTabsActions } from '@/hooks/workspace'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'
import type { BrowserTab } from './BrowserTabs'
import { BrowserTabs } from './BrowserTabs'

function runLifecycle(tab: WorkspaceTab, hook: 'onMount' | 'onUnmount' | 'onActivate' | 'onDeactivate') {
  try {
    getTabLifecycle(tab.type)[hook]?.(tab)
  } catch (error) {
    console.error(`[workspace] tab lifecycle ${hook} failed`, error)
  }
}

export function WorkspacePanel() {
  const tabs = useWorkspaceStore(s => s.tabs)
  const activeTabId = useWorkspaceStore(s => s.activeTabId)
  const openTab = useWorkspaceStore(s => s.openTab)
  const tabActions = useBrowserTabsActions()
  const previousSnapshotRef = useRef(
    buildWorkspaceKernelSnapshot([], '', tabType => getTabKeepAliveOnDeactivate(tabType))
  )

  useEffect(() => {
    const previousSnapshot = previousSnapshotRef.current
    const nextSnapshot = buildWorkspaceKernelSnapshot(tabs, activeTabId, tabType =>
      getTabKeepAliveOnDeactivate(tabType)
    )
    const lifecycleEvents = computeWorkspaceLifecycleEvents(previousSnapshot, nextSnapshot)
    for (const event of lifecycleEvents) {
      runLifecycle(event.tab, event.hook)
    }
    previousSnapshotRef.current = nextSnapshot
  }, [activeTabId, tabs])

  const handleNewTab = (config: AgentSessionLaunchConfig) => {
    openTab(createTerminalTabFromLaunch(config))
  }

  const handleOpenRunForm = (config: AgentSessionLaunchConfig) => {
    openTab(createAgentSessionConfigTab(config))
  }

  const browserTabs: BrowserTab[] = tabs.map(t => ({
    id: t.id,
    title: t.title,
    icon: t.icon,
    closeable: t.closeable
  }))

  const newTabButton = <AgentSessionLaunchButton onNewTab={handleNewTab} onOpenRunForm={handleOpenRunForm} />
  const { activeTab, inactiveKeepAliveTabs } = buildWorkspaceMountPlan(tabs, activeTabId, tabType =>
    getTabKeepAliveOnDeactivate(tabType)
  )

  return (
    <div className="flex h-full flex-col">
      <BrowserTabs tabs={browserTabs} activeTabId={activeTabId} actions={tabActions} newTabButton={newTabButton} />

      <div className="flex-1 min-h-0 relative">
        {tabs.length === 0 ? (
          <EmptyWorkspace />
        ) : (
          <>
            {inactiveKeepAliveTabs.map(tab => (
              <div key={tab.id} className="absolute inset-0 hidden">
                <WorkspaceTabContentHost tab={tab} isActive={false} />
              </div>
            ))}
            {activeTab && (
              <div className="absolute inset-0">
                <WorkspaceTabContentHost tab={activeTab} isActive />
              </div>
            )}
          </>
        )}
      </div>
    </div>
  )
}

function EmptyWorkspace() {
  const openTab = useWorkspaceStore(s => s.openTab)

  return (
    <div className="h-full flex items-center justify-center">
      <div className="text-center space-y-3 text-muted-foreground">
        <p className="text-sm">No workspace tabs open</p>
        <div className="text-xs opacity-70 space-y-1">
          <p>Click the + button above to create a new terminal</p>
          <p>or select an item from the left to open a tab</p>
        </div>
        <Button
          variant="link"
          size="sm"
          onClick={() => openTab(createDefaultTerminalTab('Terminal 1'))}
          className="text-xs">
          Create terminal tab
        </Button>
      </div>
    </div>
  )
}
