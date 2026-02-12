export {
  type AgentSessionConfigTabParams,
  createAgentSessionConfigTab
} from './agent-session-config'
export {
  useWorkspaceTabContext,
  useWorkspaceTabData,
  useWorkspaceTabOfType,
  WorkspaceTabProvider
} from './context'
export {
  defineWorkspaceTabModule,
  type WorkspaceTabLifecycle,
  type WorkspaceTabModule
} from './contracts'
export {
  createDisciplineDetailTab,
  type DisciplineDetailTabInput,
  type DisciplineDetailTabParams
} from './discipline-detail'
export { createFeatureDetailTab, type FeatureDetailTabParams } from './feature-detail'
export { getTabComponent, getTabLifecycle } from './registry'
export { createTaskDetailTab, type TaskDetailTabParams } from './task-detail'
export { createTerminalTab, type TerminalTabParams } from './terminal'
