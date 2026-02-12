import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import type { DisciplineDetailTabParams } from './schema'

export type DisciplineDetailTabInput = {
  name: string
  displayName: string
}

export function createDisciplineDetailTab(discipline: DisciplineDetailTabInput): Omit<WorkspaceTab, 'id'> {
  return {
    type: 'discipline-detail',
    title: discipline.displayName,
    key: discipline.name,
    closeable: true,
    params: {
      entityId: discipline.name
    } satisfies DisciplineDetailTabParams
  }
}
