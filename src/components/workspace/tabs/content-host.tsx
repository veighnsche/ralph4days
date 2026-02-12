import { ErrorBoundary } from '@/components/shared'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import { WorkspaceTabProvider } from './context'
import { getTabComponent } from './registry'

export function WorkspaceTabContentHost({ tab }: { tab: WorkspaceTab }) {
  const TabComponent = getTabComponent(tab.type)

  return (
    <ErrorBoundary>
      <WorkspaceTabProvider tab={tab}>
        <TabComponent tab={tab} />
      </WorkspaceTabProvider>
    </ErrorBoundary>
  )
}
