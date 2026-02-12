import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import type { FeatureData } from '@/types/generated'
import type { FeatureDetailTabParams } from './schema'

export function createFeatureDetailTab(feature: FeatureData): Omit<WorkspaceTab, 'id'> {
  return {
    type: 'feature-detail',
    title: feature.displayName,
    key: feature.name,
    closeable: true,
    params: {
      entityId: feature.name
    } satisfies FeatureDetailTabParams
  }
}
