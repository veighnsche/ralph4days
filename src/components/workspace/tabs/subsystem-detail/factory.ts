import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import type { SubsystemDetailTabParams } from './schema'

export function createSubsystemDetailTab(subsystemId: number): Omit<WorkspaceTab, 'id'> {
  return {
    type: 'subsystem-detail',
    title: `Subsystem #${subsystemId.toString().padStart(3, '0')}`,
    key: String(subsystemId),
    closeable: true,
    params: {
      entityId: subsystemId
    } satisfies SubsystemDetailTabParams
  }
}
