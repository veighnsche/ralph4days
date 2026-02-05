import { useEffect } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

export function useTauriEvent<T>(
  eventName: string,
  handler: (payload: T) => void
) {
  useEffect(() => {
    // Skip if not in Tauri context
    if (typeof window === 'undefined' || !('__TAURI__' in window)) {
      return;
    }

    let unlisten: UnlistenFn | null = null;

    listen<T>(eventName, (event) => {
      handler(event.payload);
    }).then((fn) => {
      unlisten = fn;
    }).catch(console.error);

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [eventName, handler]);
}
