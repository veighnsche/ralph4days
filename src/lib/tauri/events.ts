import { listen } from '@tauri-apps/api/event'

export type TauriUnlistenFn = () => void

// Single boundary for frontend event subscriptions.
// Later: remote mode can swap this implementation to WS without touching call sites.
export function tauriListen<T>(event: string, handler: (event: { payload: T }) => void): Promise<TauriUnlistenFn> {
  return listen<T>(event, handler)
}
