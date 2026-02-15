import { listen } from '@tauri-apps/api/event'
import { FRONTEND_LISTENED_EVENT_NAMES, type FrontendEventName } from './listenedEventsContract'

export type TauriUnlistenFn = () => void

const FRONTEND_LISTENED_EVENT_NAME_SET = new Set<string>(FRONTEND_LISTENED_EVENT_NAMES)

// Single boundary for frontend event subscriptions.
// Later: remote mode can swap this implementation to WS without touching call sites.
export function tauriListen<T>(
  event: FrontendEventName,
  handler: (event: { payload: T }) => void
): Promise<TauriUnlistenFn> {
  if (!FRONTEND_LISTENED_EVENT_NAME_SET.has(event)) {
    throw new Error(`Unknown frontend event name: ${event}`)
  }
  return listen<T>(event, handler)
}
