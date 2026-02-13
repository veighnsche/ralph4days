import { type ComponentType, createElement } from 'react'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import type { WorkspaceTabLifecycle } from './contracts'
import { workspaceTabModules } from './modules'

const NOOP_TAB_LIFECYCLE: WorkspaceTabLifecycle = {
  onMount: () => {},
  onUnmount: () => {},
  onActivate: () => {},
  onDeactivate: () => {}
}

const TAB_COMPONENTS = Object.fromEntries(
  workspaceTabModules.map(module => [
    module.type,
    ({ tab }: { tab: WorkspaceTab }) => {
      const params = module.parseParams(tab.params)
      const component = module.component as ComponentType<{ tab: WorkspaceTab; params: unknown }>
      return createElement(component, { tab, params })
    }
  ])
) as Record<string, ComponentType<{ tab: WorkspaceTab }>>

const TAB_LIFECYCLES = Object.fromEntries(
  workspaceTabModules.filter(module => module.lifecycle !== undefined).map(module => [module.type, module.lifecycle])
) as Partial<Record<string, WorkspaceTabLifecycle>>

const TAB_KEEP_ALIVE_ON_DEACTIVATE = Object.fromEntries(
  workspaceTabModules.map(module => [module.type, module.keepAliveOnDeactivate === true])
) as Record<string, boolean>

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

export function getTabKeepAliveOnDeactivate(type: string): boolean {
  return TAB_KEEP_ALIVE_ON_DEACTIVATE[type] ?? false
}
