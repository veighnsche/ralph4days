import { defineWorkspaceTabModule } from '../contracts'
import { DisciplineDetailTabContent } from './content'
import { createDisciplineDetailTab } from './factory'
import { parseDisciplineDetailTabParams } from './schema'

export const disciplineDetailTabModule = defineWorkspaceTabModule({
  type: 'discipline-detail',
  component: DisciplineDetailTabContent,
  parseParams: parseDisciplineDetailTabParams,
  createTab: createDisciplineDetailTab
})
