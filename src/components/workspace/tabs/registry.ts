import type { ComponentType } from 'react'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import type { WorkspaceTabLifecycle } from './contracts'
import { workspaceTabModules } from './modules'

const NOOP_TAB_LIFECYCLE: WorkspaceTabLifecycle = {
  onMount: () => {},
  onUnmount: () => {},
  onActivate: () => {},
  onDeactivate: () => {}
}

const TAB_COMPONENTS = Object.fromEntries(workspaceTabModules.map(module => [module.type, module.component])) as Record<
  string,
  ComponentType<{ tab: WorkspaceTab }>
>

const TAB_LIFECYCLES = Object.fromEntries(
  workspaceTabModules.filter(module => module.lifecycle !== undefined).map(module => [module.type, module.lifecycle])
) as Partial<Record<string, WorkspaceTabLifecycle>>

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
