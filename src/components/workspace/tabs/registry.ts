import type { ComponentType } from 'react'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import { AgentSessionConfigTabContent } from './AgentSessionConfigTabContent'
import { DisciplineDetailTabContent } from './DisciplineDetailTabContent'
import { FeatureDetailTabContent } from './FeatureDetailTabContent'
import { TaskDetailTabContent } from './TaskDetailTabContent'
import { TerminalTabContent } from './TerminalTabContent'

export interface WorkspaceTabLifecycle {
  onMount?: (tab: WorkspaceTab) => void
  onUnmount?: (tab: WorkspaceTab) => void
  onActivate?: (tab: WorkspaceTab) => void
  onDeactivate?: (tab: WorkspaceTab) => void
}

const NOOP_TAB_LIFECYCLE: WorkspaceTabLifecycle = {
  onMount: () => {},
  onUnmount: () => {},
  onActivate: () => {},
  onDeactivate: () => {}
}

const TAB_COMPONENTS: Record<string, ComponentType<{ tab: WorkspaceTab }>> = {
  terminal: TerminalTabContent,
  'agent-session-config': AgentSessionConfigTabContent,
  'task-detail': TaskDetailTabContent,
  'feature-detail': FeatureDetailTabContent,
  'discipline-detail': DisciplineDetailTabContent
}

const TAB_LIFECYCLES: Partial<Record<string, WorkspaceTabLifecycle>> = {}

export function getTabComponent(type: string): ComponentType<{ tab: WorkspaceTab }> {
  const component = TAB_COMPONENTS[type]
  if (!component) {
    throw new Error(`Unknown tab type: ${type}`)
  }
  return component
}

export function getTabLifecycle(type: string): WorkspaceTabLifecycle {
  return TAB_LIFECYCLES[type] ?? NOOP_TAB_LIFECYCLE
}
