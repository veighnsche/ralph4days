import { ErrorBoundary } from '@/components/shared'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import { WorkspaceTabProvider } from './context'
import { getTabComponent } from './registry'

export function WorkspaceTabContentHost({ tab, isActive }: { tab: WorkspaceTab; isActive: boolean }) {
  const TabComponent = getTabComponent(tab.type)

  return (
    <ErrorBoundary>
      <WorkspaceTabProvider tab={tab} isActive={isActive}>
        <TabComponent tab={tab} />
      </WorkspaceTabProvider>
    </ErrorBoundary>
  )
}
