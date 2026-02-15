import { describe, expect, it, vi } from 'vitest'
import { RalphIpcError, tauriInvoke } from './invoke'

const mockInvoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args)
}))

describe('tauriInvoke', () => {
  it('invokes commands without args directly', async () => {
    mockInvoke.mockResolvedValueOnce({ ok: true })
    await expect(tauriInvoke<{ ok: boolean }>('protocol_version_get')).resolves.toEqual({ ok: true })
    expect(mockInvoke).toHaveBeenCalledWith('protocol_version_get')
  })

  it('wraps args under the canonical top-level args key', async () => {
    mockInvoke.mockResolvedValueOnce({ ok: true })
    await expect(tauriInvoke<{ ok: boolean }>('project_lock_set', { path: '/tmp/x' })).resolves.toEqual({ ok: true })
    expect(mockInvoke).toHaveBeenCalledWith('project_lock_set', { args: { path: '/tmp/x' } })
  })

  it('surfaces coded string errors as RalphIpcError', async () => {
    mockInvoke.mockRejectedValueOnce('[R-2000] Failed to open database')

    const p = tauriInvoke('project_lock_get')
    await expect(p).rejects.toBeInstanceOf(RalphIpcError)
    await expect(p).rejects.toMatchObject({
      code: 2000,
      message: '[R-2000] Failed to open database',
      ralph: { code: 2000, message: 'Failed to open database' }
    })
  })

  it('surfaces structured error payloads as RalphIpcError', async () => {
    mockInvoke.mockRejectedValueOnce({ code: 7000, message: 'Unknown model' })

    const p = tauriInvoke('terminal_start_session')
    await expect(p).rejects.toBeInstanceOf(RalphIpcError)
    await expect(p).rejects.toMatchObject({
      code: 7000,
      message: '[R-7000] Unknown model',
      ralph: { code: 7000, message: 'Unknown model' }
    })
  })

  it('hard-fails uncoded errors as INTERNAL RalphIpcError', async () => {
    mockInvoke.mockRejectedValueOnce('uncoded')
    await expect(tauriInvoke('tasks_get', { id: 1 })).rejects.toMatchObject({
      code: 8100,
      message: expect.stringContaining("uncoded IPC error for 'tasks_get': uncoded")
    })
  })
})
