import { describe, expect, it } from 'vitest'
import { TERMINAL_BRIDGE_EVENTS } from '@/lib/terminal/terminalBridgeContract'
import { BACKEND_DIAGNOSTIC_EVENT } from './eventsContract'
import { FRONTEND_LISTENED_EVENT_NAMES } from './listenedEventsContract'

describe('listenedEventsContract', () => {
  it('keeps the set of frontend-listened IPC event names stable', () => {
    expect([...FRONTEND_LISTENED_EVENT_NAMES]).toEqual([
      BACKEND_DIAGNOSTIC_EVENT,
      TERMINAL_BRIDGE_EVENTS.output,
      TERMINAL_BRIDGE_EVENTS.closed
    ])
  })

  it('has no duplicates', () => {
    const unique = new Set(FRONTEND_LISTENED_EVENT_NAMES)
    expect(unique.size).toBe(FRONTEND_LISTENED_EVENT_NAMES.length)
  })
})
