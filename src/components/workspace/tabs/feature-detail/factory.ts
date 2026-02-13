import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import type { FeatureDetailTabParams } from './schema'

export function createFeatureDetailTab(featureId: number): Omit<WorkspaceTab, 'id'> {
  return {
    type: 'feature-detail',
    title: `Feature #${featureId.toString().padStart(3, '0')}`,
    key: String(featureId),
    closeable: true,
    params: {
      entityId: featureId
    } satisfies FeatureDetailTabParams
  }
}
