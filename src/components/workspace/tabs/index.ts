export {
  type AgentSessionConfigTabParams,
  createAgentSessionConfigTab
} from './agent-session-config'
export { WorkspaceTabContentHost } from './content-host'
export {
  useWorkspaceTabContext,
  useWorkspaceTabData,
  useWorkspaceTabIsActive,
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
  type DisciplineDetailTabParams
} from './discipline-detail'
export { getTabComponent, getTabKeepAliveOnDeactivate, getTabLifecycle } from './registry'
export { createSubsystemDetailTab, type SubsystemDetailTabParams } from './subsystem-detail'
export { createTaskDetailTab, type TaskDetailTabParams } from './task-detail'
export {
  createDefaultTerminalTab,
  createTerminalTab,
  createTerminalTabFromLaunch,
  createTerminalTabFromTask,
  type TerminalTabParams
} from './terminal'
