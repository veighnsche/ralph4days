import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import type { DisciplineDetailTabParams } from './schema'

export function createDisciplineDetailTab(disciplineId: number): Omit<WorkspaceTab, 'id'> {
  return {
    type: 'discipline-detail',
    title: `Discipline #${disciplineId.toString().padStart(3, '0')}`,
    key: String(disciplineId),
    closeable: true,
    params: {
      entityId: disciplineId
    } satisfies DisciplineDetailTabParams
  }
}
