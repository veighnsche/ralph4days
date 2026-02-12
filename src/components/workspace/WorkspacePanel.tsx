import { useEffect, useRef } from 'react'
import { AgentSessionLaunchButton } from '@/components/agent-session-launch'
import { ErrorBoundary } from '@/components/shared'
import { Button } from '@/components/ui/button'
import { WorkspaceTabProvider } from '@/components/workspace/tabs'
import { useBrowserTabsActions, useWorkspaceActions } from '@/hooks/workspace'
import { useWorkspaceStore, type WorkspaceTab } from '@/stores/useWorkspaceStore'
import type { BrowserTab } from './BrowserTabs'
import { BrowserTabs } from './BrowserTabs'

function runLifecycle(tab: WorkspaceTab, hook: 'onMount' | 'onUnmount' | 'onActivate' | 'onDeactivate') {
  try {
    tab.lifecycle?.[hook]?.(tab)
  } catch (error) {
    console.error(`[workspace] tab lifecycle ${hook} failed`, error)
  }
}

export function WorkspacePanel() {
  const tabs = useWorkspaceStore(s => s.tabs)
  const activeTabId = useWorkspaceStore(s => s.activeTabId)
  const tabActions = useBrowserTabsActions()
  const { openAgentSessionConfigTab, openTerminalFromLaunchConfig } = useWorkspaceActions()
  const previousTabsRef = useRef<Map<string, WorkspaceTab>>(new Map())
  const previousActiveTabIdRef = useRef<string>('')

  useEffect(() => {
    const previousTabs = previousTabsRef.current
    const nextTabs = new Map(tabs.map(tab => [tab.id, tab]))

    for (const tab of tabs) {
      if (!previousTabs.has(tab.id)) {
        runLifecycle(tab, 'onMount')
      }
    }
    for (const [tabId, tab] of previousTabs) {
      if (!nextTabs.has(tabId)) {
        runLifecycle(tab, 'onUnmount')
      }
    }

    previousTabsRef.current = nextTabs
  }, [tabs])

  useEffect(() => {
    const previousActiveTabId = previousActiveTabIdRef.current
    if (previousActiveTabId === activeTabId) return

    const previousActiveTab = tabs.find(tab => tab.id === previousActiveTabId)
    const nextActiveTab = tabs.find(tab => tab.id === activeTabId)

    if (previousActiveTab) runLifecycle(previousActiveTab, 'onDeactivate')
    if (nextActiveTab) runLifecycle(nextActiveTab, 'onActivate')

    previousActiveTabIdRef.current = activeTabId
  }, [activeTabId, tabs])

  const browserTabs: BrowserTab[] = tabs.map(t => ({
    id: t.id,
    title: t.title,
    icon: t.icon,
    closeable: t.closeable
  }))

  const newTabButton = (
    <AgentSessionLaunchButton onNewTab={openTerminalFromLaunchConfig} onOpenRunForm={openAgentSessionConfigTab} />
  )

  return (
    <div className="flex h-full flex-col">
      <BrowserTabs tabs={browserTabs} activeTabId={activeTabId} actions={tabActions} newTabButton={newTabButton} />

      <div className="flex-1 min-h-0 relative">
        {tabs.length === 0 ? (
          <EmptyWorkspace />
        ) : (
          tabs.map(tab => (
            <div key={tab.id} className={tab.id === activeTabId ? 'absolute inset-0' : 'absolute inset-0 hidden'}>
              <ErrorBoundary>
                <WorkspaceTabProvider tab={tab}>
                  <tab.component tab={tab} />
                </WorkspaceTabProvider>
              </ErrorBoundary>
            </div>
          ))
        )}
      </div>
    </div>
  )
}

function EmptyWorkspace() {
  const { openDefaultTerminalTab } = useWorkspaceActions()

  return (
    <div className="h-full flex items-center justify-center">
      <div className="text-center space-y-3 text-muted-foreground">
        <p className="text-sm">No workspace tabs open</p>
        <div className="text-xs opacity-70 space-y-1">
          <p>Click the + button above to create a new terminal</p>
          <p>or select an item from the left to open a tab</p>
        </div>
        <Button variant="link" size="sm" onClick={openDefaultTerminalTab} className="text-xs">
          Create terminal tab
        </Button>
      </div>
    </div>
  )
}
