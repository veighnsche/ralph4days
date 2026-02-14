import { describe, expect, it } from 'vitest'
import { BACKEND_DIAGNOSTIC_EVENT } from './eventsContract'

describe('eventsContract', () => {
  it('keeps backend diagnostic event name stable', () => {
    expect(BACKEND_DIAGNOSTIC_EVENT).toBe('backend-diagnostic')
  })
})
