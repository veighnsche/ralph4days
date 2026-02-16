import { describe, expect, it } from 'vitest'
import { canQueryProjectLock, requiresRemoteConnection } from './mobileGate'

describe('mobileGate', () => {
  it('requires remote connection only for mobile backend mode', () => {
    expect(requiresRemoteConnection(true)).toBe(true)
    expect(requiresRemoteConnection(false)).toBe(false)
    expect(requiresRemoteConnection(undefined)).toBe(false)
  })

  it('allows project-lock query on desktop mode without remote status', () => {
    expect(canQueryProjectLock(false, undefined)).toBe(true)
    expect(canQueryProjectLock(undefined, undefined)).toBe(true)
  })

  it('blocks project-lock query on mobile until remote status is connected', () => {
    expect(canQueryProjectLock(true, undefined)).toBe(false)
    expect(canQueryProjectLock(true, { connected: false })).toBe(false)
    expect(canQueryProjectLock(true, { connected: true })).toBe(true)
  })
})
