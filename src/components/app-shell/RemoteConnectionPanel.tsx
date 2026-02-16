import { useEffect, useState } from 'react'
import { InlineError } from '@/components/shared'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Field, FieldDescription, FieldGroup, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { tauriInvoke } from '@/lib/tauri/invoke'
import type { RemoteConnectResult, RemoteStatus } from '@/types/generated'

const LAST_REMOTE_WS_URL_STORAGE_KEY = 'ralph.remote.lastWsUrl'
const DEFAULT_REMOTE_WS_URL = 'ws://127.0.0.1:9944'

function readStoredWsUrl(): string | null {
  if (typeof window === 'undefined') return null
  const value = window.localStorage.getItem(LAST_REMOTE_WS_URL_STORAGE_KEY)
  return value?.trim() ? value.trim() : null
}

function writeStoredWsUrl(value: string) {
  if (typeof window === 'undefined') return
  window.localStorage.setItem(LAST_REMOTE_WS_URL_STORAGE_KEY, value)
}

function resolveInitialWsUrl(status: RemoteStatus | undefined): string {
  return readStoredWsUrl() ?? status?.wsUrl ?? DEFAULT_REMOTE_WS_URL
}

interface RemoteConnectionPanelProps {
  status: RemoteStatus | undefined
  statusError: Error | null
  onConnected: () => void
}

export function RemoteConnectionPanel({ status, statusError, onConnected }: RemoteConnectionPanelProps) {
  const [wsUrl, setWsUrl] = useState<string>(() => resolveInitialWsUrl(status))
  const [connectError, setConnectError] = useState<Error | string | null>(null)
  const [isConnecting, setIsConnecting] = useState(false)

  useEffect(() => {
    if (!status?.wsUrl) return
    if (wsUrl.trim().length > 0) return
    setWsUrl(status.wsUrl)
  }, [status?.wsUrl, wsUrl])

  useEffect(() => {
    if (!statusError) return
    setConnectError(statusError)
  }, [statusError])

  const handleConnect = async () => {
    const normalizedWsUrl = wsUrl.trim()
    if (!normalizedWsUrl) {
      setConnectError('Remote WebSocket URL is required')
      return
    }

    setConnectError(null)
    setIsConnecting(true)
    try {
      await tauriInvoke<RemoteConnectResult>('remote_connect', { wsUrl: normalizedWsUrl })
      writeStoredWsUrl(normalizedWsUrl)
      onConnected()
    } catch (error) {
      setConnectError(error instanceof Error ? error : String(error))
    } finally {
      setIsConnecting(false)
    }
  }

  return (
    <div className="flex min-h-svh items-center justify-center px-4 py-8">
      <Card className="w-full max-w-xl">
        <CardHeader>
          <CardTitle>Connect To Ralphd</CardTitle>
          <CardDescription>Mobile mode is remote-only. Connect before opening a project.</CardDescription>
        </CardHeader>

        <CardContent className="space-y-4">
          <FieldGroup>
            <Field>
              <FieldLabel>Remote WebSocket URL</FieldLabel>
              <Input
                value={wsUrl}
                onChange={event => setWsUrl(event.target.value)}
                placeholder={DEFAULT_REMOTE_WS_URL}
                autoCapitalize="none"
                autoCorrect="off"
                spellCheck={false}
              />
              <FieldDescription>Use `wss://` when TLS is configured.</FieldDescription>
            </Field>
          </FieldGroup>

          <InlineError error={connectError} onDismiss={() => setConnectError(null)} />

          {status?.wsUrl && !status.connected ? (
            <p className="text-xs text-muted-foreground">Last attempted endpoint: {status.wsUrl}</p>
          ) : null}

          <Button onClick={handleConnect} disabled={isConnecting || wsUrl.trim().length === 0} className="w-full">
            {isConnecting ? 'Connecting...' : 'Connect'}
          </Button>
        </CardContent>
      </Card>
    </div>
  )
}
