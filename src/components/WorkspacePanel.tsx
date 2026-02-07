import type { BrowserTab } from '@/components/BrowserTabs'
import { BrowserTabs } from '@/components/BrowserTabs'
import { ErrorBoundary } from '@/components/ErrorBoundary'
import { type Model, ModelThinkingTabButton } from '@/components/ModelThinkingTabButton'
import { Button } from '@/components/ui/button'
import {
  BraindumpFormTabContent,
  DisciplineFormTabContent,
  FeatureFormTabContent,
  TaskDetailTabContent,
  TaskFormTabContent,
  TerminalTabContent
} from '@/components/workspace'
import { useWorkspaceStore, type WorkspaceTab } from '@/stores/useWorkspaceStore'

export function WorkspacePanel() {
  const tabs = useWorkspaceStore(s => s.tabs)
  const activeTabId = useWorkspaceStore(s => s.activeTabId)
  const switchTab = useWorkspaceStore(s => s.switchTab)
  const closeTab = useWorkspaceStore(s => s.closeTab)
  const openTab = useWorkspaceStore(s => s.openTab)

  const handleNewTab = (model: Model, thinking: boolean) => {
    openTab({
      type: 'terminal',
      title: `Claude (${model})`,
      closeable: true,
      data: {
        model,
        thinking
      }
    })
  }

  const browserTabs: BrowserTab[] = tabs.map(t => ({
    id: t.id,
    title: t.title,
    icon: t.icon,
    closeable: t.closeable
  }))

  const newTabButton = <ModelThinkingTabButton onNewTab={handleNewTab} />

  return (
    <div className="flex h-full flex-col">
      {/* BrowserTabs ALWAYS rendered at root - never bound to tab content */}
      <BrowserTabs
        tabs={browserTabs}
        activeTabId={activeTabId}
        onTabChange={switchTab}
        onTabClose={closeTab}
        newTabButton={newTabButton}
      />

      {/* Tab content area - shows empty state if no tabs */}
      <div className="flex-1 min-h-0 relative">
        {tabs.length === 0 ? (
          <EmptyWorkspace />
        ) : (
          tabs.map(tab => (
            <div key={tab.id} className={tab.id === activeTabId ? 'absolute inset-0' : 'absolute inset-0 hidden'}>
              <ErrorBoundary>
                <TabContent tab={tab} />
              </ErrorBoundary>
            </div>
          ))
        )}
      </div>
    </div>
  )
}

function TabContent({ tab }: { tab: WorkspaceTab }) {
  switch (tab.type) {
    case 'terminal':
      return <TerminalTabContent tab={tab} />
    case 'braindump-form':
      return <BraindumpFormTabContent tab={tab} />
    case 'task-form':
      return <TaskFormTabContent tab={tab} />
    case 'feature-form':
      return <FeatureFormTabContent tab={tab} />
    case 'discipline-form':
      return <DisciplineFormTabContent tab={tab} />
    case 'task-detail':
      return <TaskDetailTabContent tab={tab} />
    case 'feature-detail':
      return <PlaceholderTab title={tab.title} />
    case 'discipline-detail':
      return <PlaceholderTab title={tab.title} />
  }
}

function PlaceholderTab({ title }: { title: string }) {
  return (
    <div className="h-full flex items-center justify-center text-muted-foreground">
      <span>{title} — coming soon</span>
    </div>
  )
}

function EmptyWorkspace() {
  const openTab = useWorkspaceStore(s => s.openTab)

  const handleCreateTerminal = () => {
    openTab({
      type: 'terminal',
      title: 'Terminal 1',
      closeable: true
    })
  }

  return (
    <div className="h-full flex items-center justify-center">
      <div className="text-center space-y-3 text-muted-foreground">
        <p className="text-sm">No workspace tabs open</p>
        <div className="text-xs opacity-70 space-y-1">
          <p>Click the + button above to create a new terminal</p>
          <p>or select an item from the left to open a tab</p>
        </div>
        <Button variant="link" size="sm" onClick={handleCreateTerminal} className="text-xs">
          Create terminal tab
        </Button>
      </div>
    </div>
  )
}
