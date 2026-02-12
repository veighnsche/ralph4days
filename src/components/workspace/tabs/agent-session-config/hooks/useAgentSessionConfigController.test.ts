import { renderHook } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import type { WorkspaceTab } from '@/stores/useWorkspaceStore'
import { useAgentSessionConfigController } from './useAgentSessionConfigController'

const { useModelFormTreeByAgentMock, useModelConstraintsMock, useRunSessionActionMock } = vi.hoisted(() => ({
  useModelFormTreeByAgentMock: vi.fn(),
  useModelConstraintsMock: vi.fn(),
  useRunSessionActionMock: vi.fn()
}))

vi.mock('./useSyncLaunchPreferences', () => ({
  useSyncLaunchPreferences: vi.fn()
}))

vi.mock('./useModelFormTreeByAgent', () => ({
  useModelFormTreeByAgent: () => useModelFormTreeByAgentMock()
}))

vi.mock('./useModelConstraints', () => ({
  useModelConstraints: (input: unknown) => useModelConstraintsMock(input)
}))

vi.mock('./useRunSessionAction', () => ({
  useRunSessionAction: (tab: WorkspaceTab, input: { selectedModelDisplay: string | undefined; canRun: boolean }) =>
    useRunSessionActionMock(tab, input)
}))

describe('useAgentSessionConfigController', () => {
  const tab: WorkspaceTab = {
    id: 'tab-1',
    type: 'agent-session-config',
    title: 'Start Agent Session',
    closeable: true
  }

  it('disables run when model is not valid for current constraints', () => {
    const runSession = vi.fn()
    useModelFormTreeByAgentMock.mockReturnValue({
      formTreeByAgent: {},
      formTreeLoading: false,
      formTreeError: null
    })
    useModelConstraintsMock.mockReturnValue({
      models: [],
      loadingModels: false,
      error: null,
      selectedModel: null,
      selectedModelEffortValid: false
    })
    useRunSessionActionMock.mockReturnValue(runSession)

    const { result } = renderHook(() => useAgentSessionConfigController(tab))

    expect(result.current.canRun).toBe(false)
    expect(result.current.runSession).toBe(runSession)
    expect(useRunSessionActionMock).toHaveBeenCalledWith(tab, { selectedModelDisplay: undefined, canRun: false })
  })

  it('enables run only when selected model and effort are valid', () => {
    const runSession = vi.fn()
    useModelFormTreeByAgentMock.mockReturnValue({
      formTreeByAgent: {},
      formTreeLoading: false,
      formTreeError: null
    })
    useModelConstraintsMock.mockReturnValue({
      models: [{ name: 'gpt-5-codex', display: 'GPT-5 Codex' }],
      loadingModels: false,
      error: null,
      selectedModel: { name: 'gpt-5-codex', display: 'GPT-5 Codex' },
      selectedModelEffortValid: true
    })
    useRunSessionActionMock.mockReturnValue(runSession)

    const { result } = renderHook(() => useAgentSessionConfigController(tab))

    expect(result.current.canRun).toBe(true)
    expect(result.current.runSession).toBe(runSession)
    expect(useRunSessionActionMock).toHaveBeenCalledWith(tab, { selectedModelDisplay: 'GPT-5 Codex', canRun: true })
  })
})
