import { describe, expect, it } from 'vitest'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import { buildWorkspaceKernelSnapshot, buildWorkspaceMountPlan, computeWorkspaceLifecycleEvents } from './kernel'

const terminalTab: WorkspaceTab = {
  id: 'terminal-1',
  type: 'terminal',
  title: 'Terminal',
  closeable: true
}

const taskTab: WorkspaceTab = {
  id: 'task-detail-101',
  type: 'task-detail',
  title: 'Task #101',
  closeable: true
}

function keepAliveOnDeactivate(tabType: string): boolean {
  return tabType === 'terminal'
}

describe('workspace kernel mount plan', () => {
  it('keeps inactive keep-alive tabs mounted', () => {
    const mountPlan = buildWorkspaceMountPlan([terminalTab, taskTab], taskTab.id, keepAliveOnDeactivate)
    expect(mountPlan.activeTab?.id).toBe(taskTab.id)
    expect(mountPlan.inactiveKeepAliveTabs.map(tab => tab.id)).toEqual([terminalTab.id])
  })
})

describe('workspace kernel lifecycle events', () => {
  it('fires deactivate/unmount/mount/activate in deterministic order for non-keepalive switches', () => {
    const previous = buildWorkspaceKernelSnapshot([taskTab], taskTab.id, keepAliveOnDeactivate)
    const next = buildWorkspaceKernelSnapshot(
      [taskTab, { ...taskTab, id: 'task-detail-102' }],
      'task-detail-102',
      keepAliveOnDeactivate
    )
    const events = computeWorkspaceLifecycleEvents(previous, next)
    expect(events.map(event => event.hook)).toEqual(['onDeactivate', 'onUnmount', 'onMount', 'onActivate'])
    expect(events.map(event => event.tab.id)).toEqual([taskTab.id, taskTab.id, 'task-detail-102', 'task-detail-102'])
  })

  it('deactivates then unmounts active tab when active tab is closed', () => {
    const previous = buildWorkspaceKernelSnapshot([taskTab, terminalTab], taskTab.id, keepAliveOnDeactivate)
    const next = buildWorkspaceKernelSnapshot([terminalTab], terminalTab.id, keepAliveOnDeactivate)
    const events = computeWorkspaceLifecycleEvents(previous, next)
    expect(events.map(event => event.hook)).toEqual(['onDeactivate', 'onUnmount', 'onActivate'])
    expect(events.map(event => event.tab.id)).toEqual([taskTab.id, taskTab.id, terminalTab.id])
  })
})
