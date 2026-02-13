import { describe, expect, it, vi } from 'vitest'
import type { AgentSessionLaunchConfig } from '@/lib/agent-session-launch-config'
import { resolveLaunchConfigAgainstCatalog } from './resolveLaunchConfig'

const { terminalBridgeListModelFormTreeMock } = vi.hoisted(() => ({
  terminalBridgeListModelFormTreeMock: vi.fn()
}))

vi.mock('@/lib/terminal/terminalBridgeClient', () => ({
  terminalBridgeListModelFormTree: () => terminalBridgeListModelFormTreeMock()
}))

const BASE_CONFIG: AgentSessionLaunchConfig = {
  agent: 'claude',
  model: 'claude-sonnet-4',
  effort: 'medium',
  thinking: true,
  permissionLevel: 'balanced'
}

describe('resolveLaunchConfigAgainstCatalog', () => {
  it('keeps config when model exists and effort is valid', async () => {
    terminalBridgeListModelFormTreeMock.mockResolvedValue({
      providers: [
        {
          agent: 'claude',
          models: [{ name: 'claude-sonnet-4', effortOptions: ['low', 'medium', 'high'] }]
        }
      ]
    })

    const next = await resolveLaunchConfigAgainstCatalog(BASE_CONFIG)
    expect(next).toEqual(BASE_CONFIG)
  })

  it('replaces stale model with first valid model for agent', async () => {
    terminalBridgeListModelFormTreeMock.mockResolvedValue({
      providers: [
        {
          agent: 'claude',
          models: [{ name: 'claude-opus-4', effortOptions: ['low', 'medium', 'high'] }]
        }
      ]
    })

    const next = await resolveLaunchConfigAgainstCatalog({
      ...BASE_CONFIG,
      model: 'stale-model'
    })

    expect(next.model).toBe('claude-opus-4')
    expect(next.effort).toBe('medium')
  })

  it('normalizes effort when selected model does not support saved effort', async () => {
    terminalBridgeListModelFormTreeMock.mockResolvedValue({
      providers: [
        {
          agent: 'claude',
          models: [{ name: 'claude-opus-4', effortOptions: ['low', 'high'] }]
        }
      ]
    })

    const next = await resolveLaunchConfigAgainstCatalog({
      ...BASE_CONFIG,
      model: 'claude-opus-4',
      effort: 'medium'
    })

    expect(next.effort).toBe('low')
  })
})
