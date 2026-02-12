import type { ComponentType } from 'react'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'

export interface WorkspaceTabLifecycle {
  onMount?: (tab: WorkspaceTab) => void
  onUnmount?: (tab: WorkspaceTab) => void
  onActivate?: (tab: WorkspaceTab) => void
  onDeactivate?: (tab: WorkspaceTab) => void
}

export interface WorkspaceTabModule<TType extends string = string, TParams = unknown, TInput = unknown> {
  type: TType
  component: ComponentType<{ tab: WorkspaceTab; params: TParams }>
  parseParams: (params: unknown) => TParams
  createTab: (input: TInput) => Omit<WorkspaceTab, 'id'>
  lifecycle?: WorkspaceTabLifecycle
}

export function defineWorkspaceTabModule<TType extends string, TParams = unknown, TInput = unknown>(
  module: WorkspaceTabModule<TType, TParams, TInput>
) {
  return module
}
