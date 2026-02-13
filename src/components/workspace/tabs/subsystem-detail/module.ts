import { defineWorkspaceTabModule } from '../contracts'
import { SubsystemDetailTabContent } from './content'
import { createSubsystemDetailTab } from './factory'
import { parseSubsystemDetailTabParams } from './schema'

export const subsystemDetailTabModule = defineWorkspaceTabModule({
  type: 'subsystem-detail',
  component: SubsystemDetailTabContent,
  parseParams: parseSubsystemDetailTabParams,
  createTab: createSubsystemDetailTab
})
