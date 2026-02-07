import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useEffect, useRef } from 'react'

export function useTauriEvent<T>(eventName: string, handler: (payload: T) => void) {
  const handlerRef = useRef(handler)
  handlerRef.current = handler

  useEffect(() => {
    if (typeof window === 'undefined' || !('__TAURI__' in window)) {
      return
    }

    let unlisten: UnlistenFn | null = null

    listen<T>(eventName, event => {
      handlerRef.current(event.payload)
    })
      .then(fn => {
        unlisten = fn
      })
      .catch(console.error)

    return () => {
      if (unlisten) {
        unlisten()
      }
    }
  }, [eventName])
}
