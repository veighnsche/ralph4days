import { type Agent, type Model, ModelThinkingTabButton } from '@/components/model-thinking'
import { ErrorBoundary } from '@/components/shared'
import { Button } from '@/components/ui/button'
import { TerminalRunFormTabContent, TerminalTabContent } from '@/components/workspace'
import { useBrowserTabsActions } from '@/hooks/workspace'
import { useWorkspaceStore } from '@/stores/useWorkspaceStore'
import type { BrowserTab } from './BrowserTabs'
import { BrowserTabs } from './BrowserTabs'

export function WorkspacePanel() {
  const tabs = useWorkspaceStore(s => s.tabs)
  const activeTabId = useWorkspaceStore(s => s.activeTabId)
  const openTab = useWorkspaceStore(s => s.openTab)
  const tabActions = useBrowserTabsActions()

  const handleNewTab = (agent: Agent, model: Model, thinking: boolean) => {
    const agentLabel = agent === 'codex' ? 'Codex' : 'Claude'
    openTab({
      type: 'terminal',
      component: TerminalTabContent,
      title: `${agentLabel} (${model})`,
      closeable: true,
      data: {
        agent,
        model,
        thinking
      }
    })
  }

  const handleOpenRunForm = (agent: Agent, model: Model, thinking: boolean) => {
    openTab({
      type: 'terminal-run-form',
      component: TerminalRunFormTabContent,
      title: 'Run Agent',
      closeable: true,
      data: { agent, model, thinking }
    })
  }

  const browserTabs: BrowserTab[] = tabs.map(t => ({
    id: t.id,
    title: t.title,
    icon: t.icon,
    closeable: t.closeable
  }))

  const newTabButton = <ModelThinkingTabButton onNewTab={handleNewTab} onOpenRunForm={handleOpenRunForm} />

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
                <tab.component tab={tab} />
              </ErrorBoundary>
            </div>
          ))
        )}
      </div>
    </div>
  )
}

function EmptyWorkspace() {
  const openTab = useWorkspaceStore(s => s.openTab)

  const handleCreateTerminal = () => {
    openTab({
      type: 'terminal',
      component: TerminalTabContent,
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
