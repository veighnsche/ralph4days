import { useQueryClient } from '@tanstack/react-query'
import { Plus, ShieldAlert } from 'lucide-react'
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
import { DottedList, DottedListItem } from '@/components/ui/dotted-list'
import { Field, FieldGroup, FieldLabel } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { MobileScrollPage } from '@/components/ui/mobile-scroll-page'
import { Separator } from '@/components/ui/separator'
import { useInvoke } from '@/hooks/api'
import { useRemoteSshPreferences } from '@/hooks/preferences/useRemoteSshPreferences'
import { useIsMobile } from '@/hooks/use-mobile'
import { RemoteSshProfileEditorDialog } from '@/components/app-shell/remote-ssh/ProfileEditorDialog'
import { ConnectPromptState, ProfileDraft } from '@/components/app-shell/remote-ssh/types'
import { RalphIpcError, tauriInvoke } from '@/lib/tauri/invoke'
import { RemoteSshProfileActionsDrawer, RemoteSshProfilesSection } from '@/components/app-shell/remote-ssh/ProfileListSection'
import type {
  RemoteSshConnectResult,
  RemoteSshHostKeyChallenge,
  RemoteSshProfile,
  RemoteSshStatus,
  RemoteStatus
} from '@/types/generated'

const LEGACY_STORAGE_KEY = 'ralph.remote.sshProfile.v1'
const DEFAULT_SSH_PORT = 22
const DEFAULT_REMOTE_PORT = 9944

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

function normalizePanelError(error: unknown): Error | string {
  if (error instanceof Error) return error
  if (typeof error === 'string') return error

  if (error && typeof error === 'object') {
    const message = (error as { message?: unknown }).message
    if (typeof message === 'string' && message.trim().length > 0) {
      return message
    }
    try {
      return JSON.stringify(error)
    } catch {
      return 'Unserializable error object'
    }
  }

  return String(error)
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
  activeProfileId: string | undefined,
  defaultProfileId: string | null
): RemoteSshProfile[] {
  return profiles.slice().sort((a, b) => {
    const aIsActive = activeProfileId === a.id ? 1 : 0
    const bIsActive = activeProfileId === b.id ? 1 : 0
    if (aIsActive !== bIsActive) return bIsActive - aIsActive

    const aIsDefault = defaultProfileId === a.id ? 1 : 0
    const bIsDefault = defaultProfileId === b.id ? 1 : 0
    if (aIsDefault !== bIsDefault) return bIsDefault - aIsDefault

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
  const isMobile = useIsMobile()
  const queryClient = useQueryClient()
  const defaultProfileId = useRemoteSshPreferences(s => s.defaultProfileId)
  const setDefaultProfileId = useRemoteSshPreferences(s => s.setDefaultProfileId)
  const {
    data: profiles,
    error: profilesError,
    isLoading: isLoadingProfiles
  } = useInvoke<RemoteSshProfile[]>('remote_ssh_profile_list')

  const [search, setSearch] = useState('')
  const [isSearchOpen, setIsSearchOpen] = useState(false)
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
  const [profileActionsProfileId, setProfileActionsProfileId] = useState<string | null>(null)
  const pendingConnectRef = useRef<ConnectPromptState | null>(null)
  const didMigrateLegacyRef = useRef(false)
  const didAutoReconnectRef = useRef(false)
  const searchInputRef = useRef<HTMLInputElement | null>(null)
  const searchToggleRef = useRef<HTMLButtonElement | null>(null)

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
        setPanelError(normalizePanelError(error))
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
          setPanelError(normalizePanelError(error))
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

  const orderedProfiles = orderProfilesForDisplay(filteredProfiles, sshStatus?.activeProfileId, defaultProfileId)

  const hasProfiles = (profiles ?? []).length > 0
  const quickConnectProfile =
    orderedProfiles.find(profile => profile.id === sshStatus?.activeProfileId) ?? orderedProfiles[0] ?? null
  const profileActionsProfile =
    orderedProfiles.find(profile => profile.id === profileActionsProfileId) ??
    (profiles ?? []).find(profile => profile.id === profileActionsProfileId) ??
    null

  useEffect(() => {
    if (!isSearchOpen) return
    searchInputRef.current?.focus()
  }, [isSearchOpen])

  useEffect(() => {
    if (isMobile) return
    setIsSearchOpen(false)
  }, [isMobile])

  const toggleSearch = () => {
    setIsSearchOpen(prev => {
      const next = !prev
      if (!next) {
        searchToggleRef.current?.focus()
      }
      return next
    })
  }

  const closeSearch = () => {
    setIsSearchOpen(false)
    searchToggleRef.current?.focus()
  }

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
      setPanelError(normalizePanelError(error))
    } finally {
      setIsSavingProfile(false)
    }
  }

  const deleteProfile = async (profileId: string) => {
    setPanelError(null)
    setIsDeletingProfile(true)
    try {
      await tauriInvoke('remote_ssh_profile_delete', { profileId })
      if (defaultProfileId === profileId) {
        setDefaultProfileId(null)
      }
      setProfileIdToDelete(null)
      void queryClient.invalidateQueries({ queryKey: ['app', 'remote_ssh_profile_list'] })
      onDisconnected()
    } catch (error) {
      setPanelError(normalizePanelError(error))
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
        setPanelError(normalizePanelError(error))
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
      setPanelError(normalizePanelError(error))
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
      setPanelError(normalizePanelError(error))
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
      setPanelError(normalizePanelError(error))
    } finally {
      setIsDisconnecting(false)
    }
  }

  const openProfileActions = (profileId: string) => {
    setProfileActionsProfileId(profileId)
  }

  const closeProfileActions = () => {
    setProfileActionsProfileId(null)
  }

  const editProfileFromActions = () => {
    if (!profileActionsProfile) {
      throw new Error('Profile actions dialog opened without a selected profile')
    }
    setProfileActionsProfileId(null)
    openEditProfile(profileActionsProfile)
  }

  const deleteProfileFromActions = () => {
    if (!profileActionsProfile) {
      throw new Error('Delete profile action requested without a selected profile')
    }
    setProfileActionsProfileId(null)
    setProfileIdToDelete(profileActionsProfile.id)
  }

  const setDefaultFromActions = () => {
    if (!profileActionsProfile) {
      throw new Error('Set default action requested without a selected profile')
    }
    if (defaultProfileId === profileActionsProfile.id) return
    setDefaultProfileId(profileActionsProfile.id)
    setProfileActionsProfileId(null)
  }

  return (
    <MobileScrollPage className="fixed inset-0" includeBounceSentinel={false} data-testid="ssh-page-scroll-root">
      <div className="mx-auto w-full max-w-md px-[var(--mobile-card-padding-inline)] pb-[calc(env(safe-area-inset-bottom)+var(--mobile-gap-loose))] pt-[calc(env(safe-area-inset-top)+var(--mobile-gap))]">
        <Card className="border-border/70 bg-card/95 shadow-md backdrop-blur-sm" data-testid="ssh-connections-panel">
          <CardHeader className="gap-2 pb-1">
            <CardTitle className="text-xl tracking-tight">Connect to Ralph</CardTitle>
            <CardDescription>
              <DottedList>
                <DottedListItem>Securely reach your remote Ralph server.</DottedListItem>
                <DottedListItem>Use one profile across Linux, Windows, and macOS.</DottedListItem>
                <DottedListItem>Approve host fingerprints before any SSH trust.</DottedListItem>
              </DottedList>
            </CardDescription>
          </CardHeader>

          <CardContent className="space-y-[var(--mobile-gap-loose)]">
            <InlineError error={panelError ?? profilesError ?? null} onDismiss={() => setPanelError(null)} />

            <div className="rounded-[var(--mobile-surface-radius)] border border-amber-500/40 bg-amber-500/10 px-3 py-3">
              <p className="flex items-center gap-1 text-[11px] font-medium text-amber-700 dark:text-amber-300">
                <ShieldAlert className="h-3.5 w-3.5" />
                Host key verification required
              </p>
              <p className="mt-1 text-[11px] text-muted-foreground">
                Unknown host keys are never auto-trusted. You must explicitly approve fingerprints.
              </p>
            </div>

            <div className="space-y-[var(--mobile-gap)]">
              <Button
                onClick={openNewProfile}
                variant="outline"
                className="w-full"
                disabled={isSavingProfile}
                data-testid="ssh-new-profile-button">
                <Plus className="h-4 w-4" /> New Profile
              </Button>

              {!isMobile ? (
                <Field className="gap-1.5">
                  <FieldLabel className="text-[11px] uppercase tracking-wide text-muted-foreground">Search</FieldLabel>
                  <Input
                    data-testid="ssh-search-input"
                    value={search}
                    onChange={event => setSearch(event.target.value)}
                    placeholder="Search profiles"
                    autoCapitalize="none"
                    autoCorrect="off"
                    spellCheck={false}
                  />
                </Field>
              ) : null}
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

            <RemoteSshProfilesSection
              isMobile={isMobile}
              search={search}
              isSearchOpen={isSearchOpen}
              setSearch={setSearch}
              toggleSearch={toggleSearch}
              closeSearch={closeSearch}
              searchInputRef={searchInputRef}
              searchToggleRef={searchToggleRef}
              isLoadingProfiles={isLoadingProfiles}
              orderedProfiles={orderedProfiles}
              hasProfiles={hasProfiles}
              activeProfileId={sshStatus?.activeProfileId}
              defaultProfileId={defaultProfileId}
              isConnecting={isConnecting}
              isDisconnecting={isDisconnecting}
              isSavingProfile={isSavingProfile}
              isDeletingProfile={isDeletingProfile}
              statusConnected={status?.connected === true}
              quickConnectProfile={quickConnectProfile}
              formatLastUsed={formatLastUsed}
              onConnectProfile={profile => setConnectPrompt({ profile, password: '', keyPassphrase: '' })}
              onOpenProfileActions={openProfileActions}
              onOpenEditProfile={openEditProfile}
              onDeleteProfile={profileId => setProfileIdToDelete(profileId)}
              onSetDefaultProfile={setDefaultProfileId}
            />
          </CardContent>
        </Card>
      </div>

      <RemoteSshProfileActionsDrawer
        profile={profileActionsProfile}
        defaultProfileId={defaultProfileId}
        isSavingProfile={isSavingProfile}
        isDeletingProfile={isDeletingProfile}
        onOpenChange={open => (open ? null : closeProfileActions())}
        onEdit={editProfileFromActions}
        onDelete={deleteProfileFromActions}
        onSetDefault={setDefaultFromActions}
      />

      <RemoteSshProfileEditorDialog
        isOpen={isEditorOpen}
        onOpenChange={setIsEditorOpen}
        draft={draft}
        setDraft={setDraft}
        importKeyFile={importKeyFile}
        setImportKeyFile={setImportKeyFile}
        importKeyPassphrase={importKeyPassphrase}
        setImportKeyPassphrase={setImportKeyPassphrase}
        saveImportKeyPassphrase={saveImportKeyPassphrase}
        setSaveImportKeyPassphrase={setSaveImportKeyPassphrase}
        isSavingProfile={isSavingProfile}
        onSaveProfile={saveProfile}
      />

      <Dialog open={connectPrompt !== null} onOpenChange={open => (open ? null : setConnectPrompt(null))}>
        <DialogContent data-testid="ssh-connect-dialog">
          <DialogHeader>
            <DialogTitle>Connect Profile</DialogTitle>
            <DialogDescription>
              {connectPrompt ? `Connect to ${connectPrompt.profile.username}@${connectPrompt.profile.host}` : 'Connect'}
            </DialogDescription>
          </DialogHeader>

          {connectPrompt ? (
            <FieldGroup>
              <div className="rounded-lg border bg-muted/20 px-3 py-2 text-xs text-muted-foreground">
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
            <Button
              variant="outline"
              onClick={() => setConnectPrompt(null)}
              disabled={isConnecting}
              data-testid="ssh-connect-cancel-button"
              className="w-full sm:w-auto">
              Cancel
            </Button>
            <Button
              onClick={() => (connectPrompt ? connectProfile(connectPrompt) : null)}
              disabled={isConnecting}
              data-testid="ssh-connect-now-button"
              className="w-full sm:w-auto">
              {isConnecting ? 'Connecting...' : 'Connect Now'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <AlertDialog open={profileIdToDelete !== null} onOpenChange={open => (open ? null : setProfileIdToDelete(null))}>
        <AlertDialogContent data-testid="ssh-delete-dialog">
          <AlertDialogHeader>
            <AlertDialogTitle>Delete SSH profile?</AlertDialogTitle>
            <AlertDialogDescription>
              This removes profile metadata and stored secrets for that profile.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={isDeletingProfile} data-testid="ssh-delete-cancel-button">
              Cancel
            </AlertDialogCancel>
            <AlertDialogAction
              onClick={() => (profileIdToDelete ? deleteProfile(profileIdToDelete) : null)}
              disabled={isDeletingProfile}
              data-testid="ssh-delete-confirm-button">
              {isDeletingProfile ? 'Deleting...' : 'Delete'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AlertDialog open={hostKeyChallenge !== null} onOpenChange={open => (open ? null : dismissHostKeyChallenge())}>
        <AlertDialogContent data-testid="ssh-hostkey-dialog">
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
            <AlertDialogCancel
              onClick={rejectHostKey}
              disabled={isApprovingHostKey}
              data-testid="ssh-hostkey-reject-button">
              Reject
            </AlertDialogCancel>
            <AlertDialogAction
              onClick={approveHostKeyAndRetry}
              disabled={isApprovingHostKey}
              data-testid="ssh-hostkey-approve-button">
              {isApprovingHostKey ? 'Approving...' : 'Trust And Continue'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </MobileScrollPage>
  )
}
