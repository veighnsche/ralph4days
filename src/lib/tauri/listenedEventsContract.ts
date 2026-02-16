import { TERMINAL_BRIDGE_EVENTS } from '@/lib/terminal/terminalBridgeContract'
import { BACKEND_DIAGNOSTIC_EVENT } from './eventsContract'

// Canonical list of IPC event names that the frontend listens to.
// Rust is the canonical owner of the string values (see `crates/core-contracts`).
export const FRONTEND_LISTENED_EVENT_NAMES = [
  BACKEND_DIAGNOSTIC_EVENT,
  TERMINAL_BRIDGE_EVENTS.output,
  TERMINAL_BRIDGE_EVENTS.closed
] as const

export type FrontendEventName = (typeof FRONTEND_LISTENED_EVENT_NAMES)[number]
