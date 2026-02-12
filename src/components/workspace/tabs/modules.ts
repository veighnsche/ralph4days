import { agentSessionConfigTabModule } from './agent-session-config'
import { disciplineDetailTabModule } from './discipline-detail'
import { featureDetailTabModule } from './feature-detail'
import { taskDetailTabModule } from './task-detail'
import { terminalTabModule } from './terminal'

export const workspaceTabModules = [
  terminalTabModule,
  agentSessionConfigTabModule,
  taskDetailTabModule,
  featureDetailTabModule,
  disciplineDetailTabModule
] as const
