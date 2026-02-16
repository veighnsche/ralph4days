import { useQueryClient } from '@tanstack/react-query'
import { CheckCircle2, Clock3, Plus, ShieldAlert, Trash2, Wifi } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { InlineError } from '@/components/shared'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle
} from '@/components/ui/alert-dialog'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle
} from '@/components/ui/dialog'
import { Field, FieldDescription, FieldGroup, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Separator } from '@/components/ui/separator'
import { Switch } from '@/components/ui/switch'
import { useInvoke } from '@/hooks/api'
import { RalphIpcError, tauriInvoke } from '@/lib/tauri/invoke'
import { cn } from '@/lib/utils'
import type {
  RemoteSshAuthMode,
  RemoteSshConnectResult,
  RemoteSshHostKeyChallenge,
  RemoteSshProfile,
  RemoteSshStatus,
  RemoteStatus
} from '@/types/generated'

const LEGACY_STORAGE_KEY = 'ralph.remote.sshProfile.v1'
const DEFAULT_SSH_PORT = 22
const DEFAULT_REMOTE_PORT = 9944

type ProfileAuthMode = RemoteSshAuthMode

interface ProfileDraft {
  id?: string
  name: string
  host: string
  username: string
  sshPort: string
  remotePort: string
  authMode: ProfileAuthMode
  identityFile: string
  identityRef: string
  knownHostsFile: string
  autoReconnectEnabled: boolean
  password: string
  savePassword: boolean
  keyPassphrase: string
  saveKeyPassphrase: boolean
}

interface ConnectPromptState {
  profile: RemoteSshProfile
  password: string
  keyPassphrase: string
}

interface LegacyStoredProfile {
  host: string
  username: string
  sshPort: number
  remotePort: number
  identityFile?: string
  knownHostsFile?: string
}

interface RemoteConnectionPanelProps {
  status: RemoteStatus | undefined
  sshStatus: RemoteSshStatus | undefined
  statusError: Error | null
  onConnected: () => void
  onDisconnected: () => void
}

function defaultDraft(): ProfileDraft {
  return {
    name: '',
    host: '',
    username: '',
    sshPort: String(DEFAULT_SSH_PORT),
    remotePort: String(DEFAULT_REMOTE_PORT),
    authMode: 'key',
    identityFile: '',
    identityRef: '',
    knownHostsFile: '',
    autoReconnectEnabled: false,
    password: '',
    savePassword: false,
    keyPassphrase: '',
    saveKeyPassphrase: false
  }
}

function draftFromProfile(profile: RemoteSshProfile): ProfileDraft {
  return {
    id: profile.id,
    name: profile.name,
    host: profile.host,
    username: profile.username,
    sshPort: String(profile.sshPort),
    remotePort: String(profile.remotePort),
    authMode: profile.authMode,
    identityFile: profile.identityFile ?? '',
    identityRef: profile.identityRef ?? '',
    knownHostsFile: profile.knownHostsFile ?? '',
    autoReconnectEnabled: profile.autoReconnectEnabled,
    password: '',
    savePassword: false,
    keyPassphrase: '',
    saveKeyPassphrase: false
  }
}

function parsePort(value: string, fieldName: string): number {
  const normalized = value.trim()
  if (!/^\d+$/.test(normalized)) {
    throw new Error(`${fieldName} must be a positive integer`)
  }
  const parsed = Number(normalized)
  if (!Number.isInteger(parsed) || parsed < 1 || parsed > 65535) {
    throw new Error(`${fieldName} must be between 1 and 65535`)
  }
  return parsed
}

function validateDraft(draft: ProfileDraft) {
  const name = draft.name.trim()
  const host = draft.host.trim()
  const username = draft.username.trim()

  if (!name) throw new Error('Profile name is required')
  if (!host) throw new Error('SSH host is required')
  if (!username) throw new Error('SSH username is required')

  const sshPort = parsePort(draft.sshPort, 'SSH port')
  const remotePort = parsePort(draft.remotePort, 'Remote ralphd port')

  if (draft.authMode === 'password' && draft.savePassword && draft.password.trim().length === 0) {
    throw new Error('Password is required when "Save password" is enabled')
  }

  if (draft.authMode === 'key' && draft.saveKeyPassphrase && draft.keyPassphrase.trim().length === 0) {
    throw new Error('Key passphrase is required when "Save key passphrase" is enabled')
  }

  return {
    id: draft.id,
    name,
    host,
    username,
    sshPort,
    remotePort,
    authMode: draft.authMode,
    identityFile: draft.identityFile.trim() || undefined,
    identityRef: draft.identityRef.trim() || undefined,
    knownHostsFile: draft.knownHostsFile.trim() || undefined,
    autoReconnectEnabled: draft.autoReconnectEnabled,
    password: draft.password.trim() || undefined,
    keyPassphrase: draft.keyPassphrase.trim() || undefined,
    savePassword: draft.savePassword,
    saveKeyPassphrase: draft.saveKeyPassphrase
  }
}

function isHostKeyChallengeLike(value: unknown): value is RemoteSshHostKeyChallenge {
  if (value == null || typeof value !== 'object') return false
  const obj = value as Record<string, unknown>
  return (
    typeof obj.challengeId === 'string' &&
    typeof obj.host === 'string' &&
    typeof obj.sshPort === 'number' &&
    typeof obj.algorithm === 'string' &&
    typeof obj.fingerprintSha256 === 'string' &&
    typeof obj.knownHostsTargetPath === 'string' &&
    typeof obj.expiresAt === 'string'
  )
}

function extractHostKeyChallenge(error: unknown): RemoteSshHostKeyChallenge | null {
  if (!(error instanceof RalphIpcError)) return null
  const item = error.ralph.context.find(ctx => ctx.key === 'ssh_hostkey_challenge')
  if (!item) return null
  return isHostKeyChallengeLike(item.value) ? item.value : null
}

async function fileToBase64(file: File): Promise<string> {
  const bytes = new Uint8Array(await file.arrayBuffer())
  let binary = ''
  for (const byte of bytes) {
    binary += String.fromCharCode(byte)
  }
  return btoa(binary)
}

function readLegacyProfile(): LegacyStoredProfile | null {
  if (typeof window === 'undefined') return null
  const raw = window.localStorage.getItem(LEGACY_STORAGE_KEY)
  if (!raw) return null

  try {
    const parsed = JSON.parse(raw) as LegacyStoredProfile
    if (
      typeof parsed.host !== 'string' ||
      typeof parsed.username !== 'string' ||
      typeof parsed.sshPort !== 'number' ||
      typeof parsed.remotePort !== 'number'
    ) {
      return null
    }
    return parsed
  } catch {
    return null
  }
}

export function formatLastUsed(lastUsedAt: string | undefined): string | null {
  if (!lastUsedAt) return null
  const epoch = Date.parse(lastUsedAt)
  if (Number.isNaN(epoch)) return lastUsedAt
  return new Date(epoch).toLocaleString()
}

export function orderProfilesForDisplay(
  profiles: RemoteSshProfile[],
  activeProfileId: string | undefined
): RemoteSshProfile[] {
  return profiles.slice().sort((a, b) => {
    const aIsActive = activeProfileId === a.id ? 1 : 0
    const bIsActive = activeProfileId === b.id ? 1 : 0
    if (aIsActive !== bIsActive) return bIsActive - aIsActive

    const aLastUsed = a.lastUsedAt ? Date.parse(a.lastUsedAt) : 0
    const bLastUsed = b.lastUsedAt ? Date.parse(b.lastUsedAt) : 0
    if (aLastUsed !== bLastUsed) return bLastUsed - aLastUsed

    return a.name.localeCompare(b.name)
  })
}

// biome-ignore lint/complexity/noExcessiveCognitiveComplexity: This component is the mobile SSH connection orchestrator and intentionally owns profile CRUD, connect, trust, and migration UX in one screen.
export function RemoteConnectionPanel({
  status,
  sshStatus,
  statusError,
  onConnected,
  onDisconnected
}: RemoteConnectionPanelProps) {
  const queryClient = useQueryClient()
  const {
    data: profiles,
    error: profilesError,
    isLoading: isLoadingProfiles
  } = useInvoke<RemoteSshProfile[]>('remote_ssh_profile_list')

  const [search, setSearch] = useState('')
  const [panelError, setPanelError] = useState<Error | string | null>(null)
  const [isEditorOpen, setIsEditorOpen] = useState(false)
  const [draft, setDraft] = useState<ProfileDraft>(defaultDraft())
  const [importKeyFile, setImportKeyFile] = useState<File | null>(null)
  const [importKeyPassphrase, setImportKeyPassphrase] = useState('')
  const [saveImportKeyPassphrase, setSaveImportKeyPassphrase] = useState(false)
  const [isSavingProfile, setIsSavingProfile] = useState(false)
  const [profileIdToDelete, setProfileIdToDelete] = useState<string | null>(null)
  const [isDeletingProfile, setIsDeletingProfile] = useState(false)
  const [connectPrompt, setConnectPrompt] = useState<ConnectPromptState | null>(null)
  const [isConnecting, setIsConnecting] = useState(false)
  const [isDisconnecting, setIsDisconnecting] = useState(false)
  const [hostKeyChallenge, setHostKeyChallenge] = useState<RemoteSshHostKeyChallenge | null>(null)
  const [isApprovingHostKey, setIsApprovingHostKey] = useState(false)
  const pendingConnectRef = useRef<ConnectPromptState | null>(null)
  const didMigrateLegacyRef = useRef(false)
  const didAutoReconnectRef = useRef(false)

  useEffect(() => {
    if (!statusError) return
    setPanelError(statusError)
  }, [statusError])

  useEffect(() => {
    if (didMigrateLegacyRef.current) return
    if (!profiles) return
    if (profiles.length > 0) {
      didMigrateLegacyRef.current = true
      return
    }

    const legacy = readLegacyProfile()
    if (!legacy) {
      didMigrateLegacyRef.current = true
      return
    }

    didMigrateLegacyRef.current = true
    void (async () => {
      try {
        await tauriInvoke<RemoteSshProfile>('remote_ssh_profile_upsert', {
          id: null,
          name: `Migrated ${legacy.username}@${legacy.host}`,
          host: legacy.host,
          username: legacy.username,
          sshPort: legacy.sshPort,
          remotePort: legacy.remotePort,
          authMode: 'key',
          identityFile: legacy.identityFile ?? null,
          identityRef: null,
          knownHostsFile: legacy.knownHostsFile ?? null,
          autoReconnectEnabled: false,
          password: null,
          keyPassphrase: null,
          savePassword: false,
          saveKeyPassphrase: false
        })
        window.localStorage.removeItem(LEGACY_STORAGE_KEY)
        void queryClient.invalidateQueries({ queryKey: ['app', 'remote_ssh_profile_list'] })
        onDisconnected()
      } catch (error) {
        setPanelError(error instanceof Error ? error : String(error))
      }
    })()
  }, [profiles, onDisconnected, queryClient])

  useEffect(() => {
    if (didAutoReconnectRef.current) return
    if (!profiles || profiles.length === 0) return
    if (status?.connected) {
      didAutoReconnectRef.current = true
      return
    }

    const auto = profiles
      .filter(profile => profile.autoReconnectEnabled)
      .sort((a, b) => {
        const aTime = a.lastUsedAt ? Date.parse(a.lastUsedAt) : 0
        const bTime = b.lastUsedAt ? Date.parse(b.lastUsedAt) : 0
        return bTime - aTime
      })

    const target = auto[0]
    if (!target) {
      didAutoReconnectRef.current = true
      return
    }

    didAutoReconnectRef.current = true
    void (async () => {
      setPanelError(null)
      setIsConnecting(true)
      try {
        await tauriInvoke<RemoteSshConnectResult>('remote_ssh_profile_connect', {
          profileId: target.id,
          password: null,
          keyPassphrase: null
        })
        pendingConnectRef.current = null
        setHostKeyChallenge(null)
        void queryClient.invalidateQueries({ queryKey: ['app', 'remote_ssh_profile_list'] })
        onConnected()
      } catch (error) {
        const challenge = extractHostKeyChallenge(error)
        if (challenge) {
          pendingConnectRef.current = { profile: target, password: '', keyPassphrase: '' }
          setHostKeyChallenge(challenge)
        } else {
          setPanelError(error instanceof Error ? error : String(error))
        }
      } finally {
        setIsConnecting(false)
      }
    })()
  }, [profiles, status?.connected, onConnected, queryClient])

  const filteredProfiles = (profiles ?? []).filter(profile => {
    const q = search.trim().toLowerCase()
    if (q.length === 0) return true
    return (
      profile.name.toLowerCase().includes(q) ||
      profile.host.toLowerCase().includes(q) ||
      profile.username.toLowerCase().includes(q)
    )
  })

  const orderedProfiles = orderProfilesForDisplay(filteredProfiles, sshStatus?.activeProfileId)

  const activeProfile = (profiles ?? []).find(profile => profile.id === sshStatus?.activeProfileId) ?? null
  const hasProfiles = (profiles ?? []).length > 0

  const openNewProfile = () => {
    setDraft(defaultDraft())
    setImportKeyFile(null)
    setImportKeyPassphrase('')
    setSaveImportKeyPassphrase(false)
    setIsEditorOpen(true)
  }

  const openEditProfile = (profile: RemoteSshProfile) => {
    setDraft(draftFromProfile(profile))
    setImportKeyFile(null)
    setImportKeyPassphrase('')
    setSaveImportKeyPassphrase(false)
    setIsEditorOpen(true)
  }

  // biome-ignore lint/complexity/noExcessiveCognitiveComplexity: Profile save/import path intentionally validates and persists multiple SSH auth surfaces in one transaction.
  const saveProfile = async () => {
    let prepared: ReturnType<typeof validateDraft>
    try {
      prepared = validateDraft(draft)
    } catch (error) {
      setPanelError(error instanceof Error ? error : String(error))
      return
    }

    setPanelError(null)
    setIsSavingProfile(true)
    try {
      const saved = await tauriInvoke<RemoteSshProfile>('remote_ssh_profile_upsert', {
        id: prepared.id ?? null,
        name: prepared.name,
        host: prepared.host,
        username: prepared.username,
        sshPort: prepared.sshPort,
        remotePort: prepared.remotePort,
        authMode: prepared.authMode,
        identityFile: prepared.identityFile ?? null,
        identityRef: prepared.identityRef ?? null,
        knownHostsFile: prepared.knownHostsFile ?? null,
        autoReconnectEnabled: prepared.autoReconnectEnabled,
        password: prepared.password ?? null,
        keyPassphrase: prepared.keyPassphrase ?? null,
        savePassword: prepared.savePassword,
        saveKeyPassphrase: prepared.saveKeyPassphrase
      })

      if (importKeyFile) {
        const bytesBase64 = await fileToBase64(importKeyFile)
        await tauriInvoke('remote_ssh_identity_import', {
          profileId: saved.id,
          fileName: importKeyFile.name,
          bytesBase64,
          passphrase: importKeyPassphrase.trim() || null,
          savePassphrase: saveImportKeyPassphrase
        })
      }

      setIsEditorOpen(false)
      setImportKeyFile(null)
      setImportKeyPassphrase('')
      setSaveImportKeyPassphrase(false)
      void queryClient.invalidateQueries({ queryKey: ['app', 'remote_ssh_profile_list'] })
      onDisconnected()
    } catch (error) {
      setPanelError(error instanceof Error ? error : String(error))
    } finally {
      setIsSavingProfile(false)
    }
  }

  const deleteProfile = async (profileId: string) => {
    setPanelError(null)
    setIsDeletingProfile(true)
    try {
      await tauriInvoke('remote_ssh_profile_delete', { profileId })
      setProfileIdToDelete(null)
      void queryClient.invalidateQueries({ queryKey: ['app', 'remote_ssh_profile_list'] })
      onDisconnected()
    } catch (error) {
      setPanelError(error instanceof Error ? error : String(error))
    } finally {
      setIsDeletingProfile(false)
    }
  }

  const connectProfile = async (prompt: ConnectPromptState) => {
    setPanelError(null)
    setIsConnecting(true)
    try {
      await tauriInvoke<RemoteSshConnectResult>('remote_ssh_profile_connect', {
        profileId: prompt.profile.id,
        password: prompt.password.trim() || null,
        keyPassphrase: prompt.keyPassphrase.trim() || null
      })
      setConnectPrompt(null)
      pendingConnectRef.current = null
      setHostKeyChallenge(null)
      void queryClient.invalidateQueries({ queryKey: ['app', 'remote_ssh_profile_list'] })
      onConnected()
    } catch (error) {
      const challenge = extractHostKeyChallenge(error)
      if (challenge) {
        pendingConnectRef.current = prompt
        setHostKeyChallenge(challenge)
      } else {
        setPanelError(error instanceof Error ? error : String(error))
      }
    } finally {
      setIsConnecting(false)
    }
  }

  const approveHostKeyAndRetry = async () => {
    if (!hostKeyChallenge) return
    setPanelError(null)
    setIsApprovingHostKey(true)
    try {
      await tauriInvoke('remote_ssh_hostkey_challenge_approve', {
        challengeId: hostKeyChallenge.challengeId
      })
      const pending = pendingConnectRef.current
      setHostKeyChallenge(null)
      if (pending) {
        await connectProfile(pending)
      }
    } catch (error) {
      setPanelError(error instanceof Error ? error : String(error))
    } finally {
      setIsApprovingHostKey(false)
    }
  }

  const rejectHostKey = async () => {
    if (!hostKeyChallenge) return
    setPanelError(null)
    try {
      await tauriInvoke('remote_ssh_hostkey_challenge_reject', {
        challengeId: hostKeyChallenge.challengeId
      })
    } catch (error) {
      setPanelError(error instanceof Error ? error : String(error))
    } finally {
      pendingConnectRef.current = null
      setHostKeyChallenge(null)
    }
  }

  const dismissHostKeyChallenge = () => {
    pendingConnectRef.current = null
    setHostKeyChallenge(null)
  }

  const disconnect = async () => {
    setPanelError(null)
    setIsDisconnecting(true)
    try {
      await tauriInvoke('remote_ssh_disconnect')
      onDisconnected()
    } catch (error) {
      setPanelError(error instanceof Error ? error : String(error))
    } finally {
      setIsDisconnecting(false)
    }
  }

  return (
    <div className="min-h-svh bg-background">
      <div className="mx-auto w-full max-w-xl px-3 pb-6 pt-[calc(env(safe-area-inset-top)+0.75rem)]">
        <Card className="border-border/80">
          <CardHeader className="pb-4">
            <CardTitle>SSH Connections</CardTitle>
            <CardDescription>
              Mobile connects to ralphd through embedded SSH. Save multiple connections and connect without typing WS
              endpoints.
            </CardDescription>
          </CardHeader>

          <CardContent className="space-y-4">
            <InlineError error={panelError ?? profilesError ?? null} onDismiss={() => setPanelError(null)} />

            <div
              className={cn(
                'rounded-md border px-3 py-3',
                sshStatus?.active ? 'border-emerald-500/40 bg-emerald-500/10' : 'border-border bg-muted/20'
              )}>
              <p className="text-[11px] font-medium uppercase tracking-wide text-muted-foreground">Tunnel status</p>
              <p className="mt-1 text-sm font-semibold">
                {sshStatus?.active
                  ? `${sshStatus.username ?? activeProfile?.username ?? 'unknown'}@${sshStatus.host ?? activeProfile?.host ?? 'unknown'}`
                  : 'Disconnected'}
              </p>
              <p className="text-xs text-muted-foreground">
                {sshStatus?.active
                  ? `SSH ${sshStatus.sshPort ?? activeProfile?.sshPort ?? DEFAULT_SSH_PORT} -> ralphd:${sshStatus.remotePort ?? activeProfile?.remotePort ?? DEFAULT_REMOTE_PORT}`
                  : 'Select a profile below to start a secure tunnel.'}
              </p>
              {sshStatus?.active ? (
                <p className="mt-1 text-[11px] text-muted-foreground">Session {sshStatus.sshSessionId ?? 'unknown'}</p>
              ) : null}
            </div>

            <div className="grid grid-cols-1 gap-2 sm:grid-cols-[1fr_auto]">
              <Field>
                <FieldLabel>Search</FieldLabel>
                <Input
                  value={search}
                  onChange={event => setSearch(event.target.value)}
                  placeholder="Search profiles"
                  autoCapitalize="none"
                  autoCorrect="off"
                  spellCheck={false}
                />
              </Field>

              <Button onClick={openNewProfile} variant="outline" className="sm:self-end" disabled={isSavingProfile}>
                <Plus className="mr-1 h-4 w-4" /> New Profile
              </Button>
            </div>

            {status?.connected ? (
              <Button
                onClick={disconnect}
                variant="destructive"
                disabled={isDisconnecting || isConnecting}
                className="w-full">
                {isDisconnecting ? 'Disconnecting...' : 'Disconnect Tunnel'}
              </Button>
            ) : null}

            <Separator />

            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <p className="text-sm font-medium">Saved Profiles</p>
                <p className="text-xs text-muted-foreground">
                  {orderedProfiles.length}
                  {search.trim().length > 0 ? ' matching' : ' total'}
                </p>
              </div>

              {isLoadingProfiles ? (
                <div className="text-sm text-muted-foreground">Loading profiles...</div>
              ) : orderedProfiles.length === 0 ? (
                <div className="rounded-md border border-dashed px-3 py-4 text-sm text-muted-foreground">
                  {hasProfiles ? `No profiles match "${search.trim()}".` : 'No SSH profiles saved yet.'}
                </div>
              ) : (
                // biome-ignore lint/complexity/noExcessiveCognitiveComplexity: Profile rows intentionally include status, trust, and action controls in one mobile-first block.
                orderedProfiles.map(profile => {
                  const isActive = sshStatus?.activeProfileId === profile.id
                  const lastUsed = formatLastUsed(profile.lastUsedAt)
                  return (
                    <div
                      key={profile.id}
                      className={cn(
                        'rounded-md border px-3 py-3',
                        isActive ? 'border-primary bg-primary/5' : 'border-border/80'
                      )}>
                      <div className="flex items-start justify-between gap-2">
                        <div className="min-w-0">
                          <p className="truncate text-sm font-semibold">{profile.name}</p>
                          <p className="truncate text-xs text-muted-foreground">
                            {profile.username}@{profile.host}:{profile.sshPort}
                          </p>
                          <p className="text-xs text-muted-foreground">ralphd:{profile.remotePort}</p>
                        </div>
                        <div className="flex shrink-0 flex-col items-end gap-1">
                          <Badge variant="outline">{profile.authMode}</Badge>
                          {profile.autoReconnectEnabled ? <Badge variant="secondary">Auto Reconnect</Badge> : null}
                          {isActive ? (
                            <Badge>
                              <CheckCircle2 className="mr-1 h-3 w-3" />
                              Active
                            </Badge>
                          ) : null}
                        </div>
                      </div>

                      {lastUsed ? (
                        <p className="mt-2 flex items-center gap-1 text-[11px] text-muted-foreground">
                          <Clock3 className="h-3 w-3" />
                          Last used {lastUsed}
                        </p>
                      ) : null}

                      <div className="mt-3 space-y-2">
                        <Button
                          size="sm"
                          onClick={() => setConnectPrompt({ profile, password: '', keyPassphrase: '' })}
                          disabled={isConnecting || isDisconnecting}
                          className="w-full">
                          <Wifi className="mr-1 h-4 w-4" />
                          Connect
                        </Button>
                        <div className="grid grid-cols-2 gap-2">
                          <Button
                            size="sm"
                            variant="outline"
                            onClick={() => openEditProfile(profile)}
                            disabled={isSavingProfile}>
                            Edit
                          </Button>
                          <Button
                            size="sm"
                            variant="outline"
                            onClick={() => setProfileIdToDelete(profile.id)}
                            disabled={isDeletingProfile}>
                            <Trash2 className="mr-1 h-4 w-4" />
                            Delete
                          </Button>
                        </div>
                      </div>
                    </div>
                  )
                })
              )}
            </div>

            <div className="rounded-md border border-amber-500/40 bg-amber-500/10 px-3 py-2">
              <p className="flex items-center gap-1 text-[11px] font-medium text-amber-700 dark:text-amber-300">
                <ShieldAlert className="h-3.5 w-3.5" />
                Host key verification required
              </p>
              <p className="mt-1 text-[11px] text-muted-foreground">
                Unknown host keys are never auto-trusted. You must explicitly approve fingerprints.
              </p>
            </div>
          </CardContent>
        </Card>
      </div>

      <Dialog open={isEditorOpen} onOpenChange={setIsEditorOpen}>
        <DialogContent className="max-h-[90svh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>{draft.id ? 'Edit SSH Profile' : 'New SSH Profile'}</DialogTitle>
            <DialogDescription>
              Profile metadata is stored by backend ownership. Secrets are optional and keychain-backed.
            </DialogDescription>
          </DialogHeader>

          <FieldGroup className="space-y-4">
            <Field>
              <FieldLabel>Profile Name</FieldLabel>
              <Input
                value={draft.name}
                onChange={event => setDraft(prev => ({ ...prev, name: event.target.value }))}
                placeholder="Work Mac"
                autoCapitalize="words"
                autoCorrect="off"
                spellCheck={false}
              />
            </Field>

            <Field>
              <FieldLabel>SSH Host</FieldLabel>
              <Input
                value={draft.host}
                onChange={event => setDraft(prev => ({ ...prev, host: event.target.value }))}
                placeholder="dev.example.com"
                autoCapitalize="none"
                autoCorrect="off"
                spellCheck={false}
              />
            </Field>

            <Field>
              <FieldLabel>SSH Username</FieldLabel>
              <Input
                value={draft.username}
                onChange={event => setDraft(prev => ({ ...prev, username: event.target.value }))}
                placeholder="vince"
                autoCapitalize="none"
                autoCorrect="off"
                spellCheck={false}
              />
            </Field>

            <div className="grid grid-cols-2 gap-2">
              <Field>
                <FieldLabel>SSH Port</FieldLabel>
                <Input
                  value={draft.sshPort}
                  onChange={event => setDraft(prev => ({ ...prev, sshPort: event.target.value }))}
                  inputMode="numeric"
                  pattern="[0-9]*"
                />
              </Field>

              <Field>
                <FieldLabel>Ralphd Port</FieldLabel>
                <Input
                  value={draft.remotePort}
                  onChange={event => setDraft(prev => ({ ...prev, remotePort: event.target.value }))}
                  inputMode="numeric"
                  pattern="[0-9]*"
                />
              </Field>
            </div>

            <Field>
              <FieldLabel>Auth Mode</FieldLabel>
              <Select
                value={draft.authMode}
                onValueChange={value =>
                  setDraft(prev =>
                    value === 'password'
                      ? {
                          ...prev,
                          authMode: value as ProfileAuthMode,
                          identityFile: '',
                          identityRef: '',
                          keyPassphrase: '',
                          saveKeyPassphrase: false
                        }
                      : {
                          ...prev,
                          authMode: value as ProfileAuthMode,
                          password: '',
                          savePassword: false
                        }
                  )
                }>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="key">Key</SelectItem>
                  <SelectItem value="password">Password</SelectItem>
                </SelectContent>
              </Select>
              <FieldDescription>Choose how this profile authenticates over SSH.</FieldDescription>
            </Field>

            <details className="rounded-md border px-3 py-2">
              <summary className="cursor-pointer text-sm font-medium">Advanced Paths (Optional)</summary>
              <div className="mt-3 space-y-3">
                <Field>
                  <FieldLabel>Known Hosts File</FieldLabel>
                  <Input
                    value={draft.knownHostsFile}
                    onChange={event => setDraft(prev => ({ ...prev, knownHostsFile: event.target.value }))}
                    placeholder="/Users/vince/.ssh/known_hosts"
                    autoCapitalize="none"
                    autoCorrect="off"
                    spellCheck={false}
                  />
                </Field>

                {draft.authMode === 'key' ? (
                  <Field>
                    <FieldLabel>Identity File Path</FieldLabel>
                    <Input
                      value={draft.identityFile}
                      onChange={event => setDraft(prev => ({ ...prev, identityFile: event.target.value }))}
                      placeholder="/Users/vince/.ssh/id_ed25519"
                      autoCapitalize="none"
                      autoCorrect="off"
                      spellCheck={false}
                    />
                    <FieldDescription>Leave blank to use default SSH key discovery.</FieldDescription>
                  </Field>
                ) : null}
              </div>
            </details>

            {draft.authMode === 'key' ? (
              <>
                <Field>
                  <FieldLabel>Import Private Key (Optional)</FieldLabel>
                  <Input
                    type="file"
                    onChange={event => setImportKeyFile(event.target.files?.[0] ?? null)}
                    accept=".pem,.key,.ppk,.txt,*/*"
                  />
                  <FieldDescription>
                    Imported key material is stored as secret and bound to this profile.
                  </FieldDescription>
                </Field>

                {importKeyFile ? (
                  <>
                    <Field>
                      <FieldLabel>Import Key Passphrase (Optional)</FieldLabel>
                      <Input
                        type="password"
                        value={importKeyPassphrase}
                        onChange={event => setImportKeyPassphrase(event.target.value)}
                      />
                    </Field>
                    <Field className="flex items-center justify-between rounded-md border px-3 py-2">
                      <div>
                        <FieldLabel>Save Import Passphrase</FieldLabel>
                        <FieldDescription>Store passphrase in keychain for reconnects.</FieldDescription>
                      </div>
                      <Switch checked={saveImportKeyPassphrase} onCheckedChange={setSaveImportKeyPassphrase} />
                    </Field>
                  </>
                ) : null}

                <Field>
                  <FieldLabel>Key Passphrase (Optional)</FieldLabel>
                  <Input
                    type="password"
                    value={draft.keyPassphrase}
                    onChange={event => setDraft(prev => ({ ...prev, keyPassphrase: event.target.value }))}
                  />
                </Field>

                <Field className="flex items-center justify-between rounded-md border px-3 py-2">
                  <div>
                    <FieldLabel>Save Key Passphrase</FieldLabel>
                    <FieldDescription>Persist passphrase in keychain for this profile.</FieldDescription>
                  </div>
                  <Switch
                    checked={draft.saveKeyPassphrase}
                    onCheckedChange={checked => setDraft(prev => ({ ...prev, saveKeyPassphrase: checked }))}
                  />
                </Field>
              </>
            ) : (
              <>
                <Field>
                  <FieldLabel>Password</FieldLabel>
                  <Input
                    type="password"
                    value={draft.password}
                    onChange={event => setDraft(prev => ({ ...prev, password: event.target.value }))}
                  />
                </Field>

                <Field className="flex items-center justify-between rounded-md border px-3 py-2">
                  <div>
                    <FieldLabel>Save Password</FieldLabel>
                    <FieldDescription>Persist password in keychain for this profile.</FieldDescription>
                  </div>
                  <Switch
                    checked={draft.savePassword}
                    onCheckedChange={checked => setDraft(prev => ({ ...prev, savePassword: checked }))}
                  />
                </Field>
              </>
            )}

            <Field className="flex items-center justify-between rounded-md border px-3 py-2">
              <div>
                <FieldLabel>Auto Reconnect</FieldLabel>
                <FieldDescription>Attempt one reconnect at app launch.</FieldDescription>
              </div>
              <Switch
                checked={draft.autoReconnectEnabled}
                onCheckedChange={checked => setDraft(prev => ({ ...prev, autoReconnectEnabled: checked }))}
              />
            </Field>
          </FieldGroup>

          <DialogFooter>
            <Button variant="outline" onClick={() => setIsEditorOpen(false)} disabled={isSavingProfile}>
              Cancel
            </Button>
            <Button onClick={saveProfile} disabled={isSavingProfile}>
              {isSavingProfile ? 'Saving...' : 'Save Profile'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={connectPrompt !== null} onOpenChange={open => (open ? null : setConnectPrompt(null))}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Connect Profile</DialogTitle>
            <DialogDescription>
              {connectPrompt ? `Connect to ${connectPrompt.profile.username}@${connectPrompt.profile.host}` : 'Connect'}
            </DialogDescription>
          </DialogHeader>

          {connectPrompt ? (
            <FieldGroup>
              <div className="rounded-md border bg-muted/20 px-3 py-2 text-xs text-muted-foreground">
                <p className="font-medium text-foreground">
                  {connectPrompt.profile.username}@{connectPrompt.profile.host}:{connectPrompt.profile.sshPort}
                </p>
                <p>Auth mode: {connectPrompt.profile.authMode}</p>
              </div>

              {connectPrompt.profile.authMode === 'password' ? (
                <Field>
                  <FieldLabel>Password (Optional if saved)</FieldLabel>
                  <Input
                    type="password"
                    value={connectPrompt.password}
                    onChange={event =>
                      setConnectPrompt(prev => (prev ? { ...prev, password: event.target.value } : prev))
                    }
                  />
                </Field>
              ) : (
                <Field>
                  <FieldLabel>Key Passphrase (Optional if saved)</FieldLabel>
                  <Input
                    type="password"
                    value={connectPrompt.keyPassphrase}
                    onChange={event =>
                      setConnectPrompt(prev => (prev ? { ...prev, keyPassphrase: event.target.value } : prev))
                    }
                  />
                </Field>
              )}
            </FieldGroup>
          ) : null}

          <DialogFooter>
            <Button variant="outline" onClick={() => setConnectPrompt(null)} disabled={isConnecting}>
              Cancel
            </Button>
            <Button onClick={() => (connectPrompt ? connectProfile(connectPrompt) : null)} disabled={isConnecting}>
              {isConnecting ? 'Connecting...' : 'Connect Now'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <AlertDialog open={profileIdToDelete !== null} onOpenChange={open => (open ? null : setProfileIdToDelete(null))}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete SSH profile?</AlertDialogTitle>
            <AlertDialogDescription>
              This removes profile metadata and stored secrets for that profile.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={isDeletingProfile}>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => (profileIdToDelete ? deleteProfile(profileIdToDelete) : null)}
              disabled={isDeletingProfile}>
              {isDeletingProfile ? 'Deleting...' : 'Delete'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AlertDialog open={hostKeyChallenge !== null} onOpenChange={open => (open ? null : dismissHostKeyChallenge())}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle className="flex items-center gap-2">
              <ShieldAlert className="h-5 w-5" /> Trust SSH Host Key?
            </AlertDialogTitle>
            <AlertDialogDescription>
              {hostKeyChallenge ? `Host ${hostKeyChallenge.host}:${hostKeyChallenge.sshPort} is not trusted yet.` : ''}
            </AlertDialogDescription>
          </AlertDialogHeader>

          {hostKeyChallenge ? (
            <div className="space-y-2 rounded-md border bg-muted/30 p-3 text-xs">
              <p>
                <strong>Algorithm:</strong> {hostKeyChallenge.algorithm}
              </p>
              <p className="break-all">
                <strong>Fingerprint:</strong> {hostKeyChallenge.fingerprintSha256}
              </p>
              <p className="break-all">
                <strong>known_hosts:</strong> {hostKeyChallenge.knownHostsTargetPath}
              </p>
              <p>
                <strong>Expires:</strong> {hostKeyChallenge.expiresAt}
              </p>
            </div>
          ) : null}

          <AlertDialogFooter>
            <AlertDialogCancel onClick={rejectHostKey} disabled={isApprovingHostKey}>
              Reject
            </AlertDialogCancel>
            <AlertDialogAction onClick={approveHostKeyAndRetry} disabled={isApprovingHostKey}>
              {isApprovingHostKey ? 'Approving...' : 'Trust And Continue'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
