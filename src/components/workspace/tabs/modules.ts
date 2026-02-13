import { agentSessionConfigTabModule } from './agent-session-config'
import { disciplineDetailTabModule } from './discipline-detail'
import { subsystemDetailTabModule } from './subsystem-detail'
import { taskDetailTabModule } from './task-detail'
import { terminalTabModule } from './terminal'

export const workspaceTabModules = [
  terminalTabModule,
  agentSessionConfigTabModule,
  taskDetailTabModule,
  subsystemDetailTabModule,
  disciplineDetailTabModule
] as const
