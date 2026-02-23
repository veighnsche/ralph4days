import { Clock3, EllipsisVertical, Search, Star, Trash2, Wifi, X } from 'lucide-react'
import type { RefObject } from 'react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Drawer, DrawerContent, DrawerDescription, DrawerHeader, DrawerTitle } from '@/components/ui/drawer'
import { Field, FieldGroup } from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import { cn } from '@/lib/utils'
import type { RemoteSshProfile } from '@/types/generated'

interface RemoteSshProfilesSectionProps {
  isMobile: boolean
  search: string
  isSearchOpen: boolean
  setSearch: (value: string) => void
  toggleSearch: () => void
  closeSearch: () => void
  searchInputRef: RefObject<HTMLInputElement | null>
  searchToggleRef: RefObject<HTMLButtonElement | null>
  isLoadingProfiles: boolean
  orderedProfiles: RemoteSshProfile[]
  hasProfiles: boolean
  activeProfileId: string | undefined
  defaultProfileId: string | null
  isConnecting: boolean
  isDisconnecting: boolean
  isSavingProfile: boolean
  isDeletingProfile: boolean
  statusConnected: boolean
  quickConnectProfile: RemoteSshProfile | null
  formatLastUsed: (lastUsedAt: string | undefined) => string | null
  onConnectProfile: (profile: RemoteSshProfile) => void
  onOpenProfileActions: (profileId: string) => void
  onOpenEditProfile: (profile: RemoteSshProfile) => void
  onDeleteProfile: (profileId: string) => void
  onSetDefaultProfile: (profileId: string) => void
}

function profileNeedsKeySetup(profile: RemoteSshProfile): boolean {
  if (profile.authMode !== 'key') return false
  const hasIdentityRef = (profile.identityRef ?? '').trim().length > 0
  const hasIdentityFile = (profile.identityFile ?? '').trim().length > 0
  return !hasIdentityRef && !hasIdentityFile
}

export function RemoteSshProfilesSection({
  isMobile,
  search,
  isSearchOpen,
  setSearch,
  toggleSearch,
  closeSearch,
  searchInputRef,
  searchToggleRef,
  isLoadingProfiles,
  orderedProfiles,
  hasProfiles,
  activeProfileId,
  defaultProfileId,
  isConnecting,
  isDisconnecting,
  isSavingProfile,
  isDeletingProfile,
  statusConnected,
  quickConnectProfile,
  formatLastUsed,
  onConnectProfile,
  onOpenProfileActions,
  onOpenEditProfile,
  onDeleteProfile,
  onSetDefaultProfile
}: RemoteSshProfilesSectionProps) {
  return (
    <>
      <div className="space-y-[var(--mobile-gap-tight)]">
        <div className="flex items-center justify-between">
          <p className="text-sm font-semibold">Saved Profiles</p>
          <div className="flex items-center gap-1.5">
            <p className="text-xs text-muted-foreground">
              {orderedProfiles.length}
              {search.trim().length > 0 ? ' matching' : ' total'}
            </p>
            {isMobile ? (
              <Button
                ref={searchToggleRef}
                variant="outline"
                size="icon"
                data-testid="ssh-search-toggle"
                aria-label={isSearchOpen ? 'Close search' : 'Open search'}
                onClick={toggleSearch}>
                <Search className="h-4 w-4" />
              </Button>
            ) : null}
          </div>
        </div>

        {isMobile && isSearchOpen ? (
          <Field className="gap-1.5">
            <div className="flex items-center gap-2">
              <Input
                ref={searchInputRef}
                data-testid="ssh-search-input"
                value={search}
                onChange={event => setSearch(event.target.value)}
                onKeyDown={event => {
                  if (event.key !== 'Escape') return
                  if (search.trim().length > 0) {
                    setSearch('')
                    return
                  }
                  closeSearch()
                }}
                placeholder="Search profiles"
                autoCapitalize="none"
                autoCorrect="off"
                spellCheck={false}
              />
              <Button
                type="button"
                variant="ghost"
                size="icon"
                data-testid="ssh-search-clear"
                onClick={() => setSearch('')}
                aria-label="Clear search">
                <X className="h-4 w-4" />
              </Button>
            </div>
          </Field>
        ) : null}

        {isLoadingProfiles ? (
          <div className="text-sm text-muted-foreground">Loading profiles...</div>
        ) : orderedProfiles.length === 0 ? (
          <div className="rounded-lg border border-dashed px-3 py-4 text-sm text-muted-foreground">
            {hasProfiles ? `No profiles match "${search.trim()}".` : 'No SSH profiles saved yet.'}
          </div>
        ) : (
          // biome-ignore lint/complexity/noExcessiveCognitiveComplexity: Profile rows intentionally include status, trust, and action controls in one mobile-first block.
          orderedProfiles.map(profile => {
            const isActive = activeProfileId === profile.id
            const isDefault = defaultProfileId === profile.id
            const requiresKeySetup = profileNeedsKeySetup(profile)
            const lastUsed = formatLastUsed(profile.lastUsedAt)
            return (
              <div
                key={profile.id}
                data-testid={`ssh-profile-card-${profile.id}`}
                className={cn(
                  'group/profile rounded-[var(--mobile-surface-radius)] border px-3.5 py-3.5 transition-[transform,border-color,background-color,box-shadow] duration-200 ease-out active:scale-[0.995]',
                  isActive
                    ? 'border-primary/70 bg-primary/5 ring-1 ring-primary/30 shadow-sm'
                    : 'border-border/80 sm:hover:border-primary/30'
                )}>
                <div className="flex items-start justify-between gap-3">
                  <div className="min-w-0">
                    <p className="truncate text-base leading-tight font-semibold">{profile.name}</p>
                    <p className="truncate text-xs text-muted-foreground">
                      {profile.username}@{profile.host}:{profile.sshPort}
                    </p>
                    <p className="text-xs text-muted-foreground">ralphd:{profile.remotePort}</p>
                  </div>
                  <div className="flex shrink-0 flex-col items-end gap-1.5">
                    <Badge variant="outline">{profile.authMode}</Badge>
                    {profile.autoReconnectEnabled ? <Badge variant="secondary">Auto Reconnect</Badge> : null}
                    {isDefault ? (
                      <Badge variant="secondary" data-testid={`ssh-profile-default-indicator-${profile.id}`}>
                        <Star className="h-3 w-3" /> Default
                      </Badge>
                    ) : null}
                    {isActive ? (
                      <Badge className="gap-1.5">
                        <span className="relative flex size-2">
                          <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-current opacity-60" />
                          <span className="relative inline-flex size-2 rounded-full bg-current" />
                        </span>
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
                  {requiresKeySetup ? (
                    <Button
                      variant="outline"
                      onClick={() => onOpenEditProfile(profile)}
                      disabled={isSavingProfile}
                      data-testid={`ssh-profile-edit-required-${profile.id}`}
                      className="w-full">
                      Edit Profile
                    </Button>
                  ) : (
                    <Button
                      onClick={() => onConnectProfile(profile)}
                      disabled={isConnecting || isDisconnecting}
                      data-testid={`ssh-profile-connect-${profile.id}`}
                      className="w-full">
                      <Wifi className="h-4 w-4" />
                      Connect
                    </Button>
                  )}
                  {isMobile ? (
                    <Button
                      variant="outline"
                      onClick={() => onOpenProfileActions(profile.id)}
                      data-testid={`ssh-profile-actions-${profile.id}`}
                      className="w-full">
                      <EllipsisVertical className="h-4 w-4" />
                      More
                    </Button>
                  ) : (
                    <>
                      <div className="grid grid-cols-2 gap-[var(--mobile-gap-tight)]">
                        <Button
                          variant="outline"
                          onClick={() => onOpenEditProfile(profile)}
                          disabled={isSavingProfile}
                          data-testid={`ssh-profile-edit-${profile.id}`}>
                          Edit
                        </Button>
                        <Button
                          variant="outline"
                          onClick={() => onDeleteProfile(profile.id)}
                          disabled={isDeletingProfile}
                          data-testid={`ssh-profile-delete-${profile.id}`}>
                          <Trash2 className="h-4 w-4" />
                          Delete
                        </Button>
                      </div>
                      {isDefault ? (
                        <Button
                          variant="outline"
                          disabled
                          data-testid={`ssh-profile-default-${profile.id}`}
                          className="w-full">
                          <Star className="h-4 w-4" />
                          Default Profile
                        </Button>
                      ) : (
                        <Button
                          variant="outline"
                          onClick={() => onSetDefaultProfile(profile.id)}
                          data-testid={`ssh-profile-set-default-${profile.id}`}
                          className="w-full">
                          <Star className="h-4 w-4" />
                          Set Default
                        </Button>
                      )}
                    </>
                  )}
                </div>
              </div>
            )
          })
        )}
      </div>

      {quickConnectProfile ? (
        <div className="sticky bottom-[calc(env(safe-area-inset-bottom)+var(--mobile-gap-tight))] z-10 rounded-[var(--mobile-surface-radius)] border border-primary/40 bg-background/85 p-[var(--mobile-gap-tight)] shadow-lg backdrop-blur-md">
          <p className="px-2 text-[11px] font-medium uppercase tracking-wide text-muted-foreground">Quick connect</p>
          {profileNeedsKeySetup(quickConnectProfile) ? (
            <Button
              variant="outline"
              onClick={() => onOpenEditProfile(quickConnectProfile)}
              disabled={isSavingProfile}
              data-testid="ssh-quick-edit-button"
              className="mt-1 w-full justify-between">
              <span className="truncate">Edit {quickConnectProfile.name}</span>
            </Button>
          ) : (
            <Button
              onClick={() => onConnectProfile(quickConnectProfile)}
              disabled={isConnecting || isDisconnecting}
              data-testid="ssh-quick-connect-button"
              className="mt-1 w-full justify-between">
              <span className="truncate">
                {statusConnected ? `Reconnect ${quickConnectProfile.name}` : `Connect ${quickConnectProfile.name}`}
              </span>
              <Wifi className="h-4 w-4 shrink-0" />
            </Button>
          )}
        </div>
      ) : null}
    </>
  )
}

interface RemoteSshProfileActionsDrawerProps {
  profile: RemoteSshProfile | null
  defaultProfileId: string | null
  isSavingProfile: boolean
  isDeletingProfile: boolean
  onOpenChange: (open: boolean) => void
  onEdit: () => void
  onDelete: () => void
  onSetDefault: () => void
}

export function RemoteSshProfileActionsDrawer({
  profile,
  defaultProfileId,
  isSavingProfile,
  isDeletingProfile,
  onOpenChange,
  onEdit,
  onDelete,
  onSetDefault
}: RemoteSshProfileActionsDrawerProps) {
  return (
    <Drawer open={profile !== null} onOpenChange={onOpenChange}>
      <DrawerContent data-testid="ssh-profile-actions-dialog">
        <DrawerHeader>
          <DrawerTitle>Profile Actions</DrawerTitle>
          <DrawerDescription>{profile ? `Manage ${profile.name}` : 'Manage selected SSH profile'}</DrawerDescription>
        </DrawerHeader>

        {profile ? (
          <FieldGroup className="space-y-2 px-4 pb-4">
            <Button
              variant="outline"
              onClick={onEdit}
              disabled={isSavingProfile}
              data-testid={`ssh-profile-action-edit-${profile.id}`}>
              Edit Profile
            </Button>
            <Button
              variant="outline"
              onClick={onDelete}
              disabled={isDeletingProfile}
              data-testid={`ssh-profile-action-delete-${profile.id}`}>
              <Trash2 className="h-4 w-4" />
              Delete Profile
            </Button>
            {defaultProfileId === profile.id ? (
              <Button variant="outline" disabled data-testid={`ssh-profile-action-default-${profile.id}`}>
                <Star className="h-4 w-4" />
                Default Profile
              </Button>
            ) : (
              <Button variant="outline" onClick={onSetDefault} data-testid={`ssh-profile-action-set-default-${profile.id}`}>
                <Star className="h-4 w-4" />
                Set As Default
              </Button>
            )}
          </FieldGroup>
        ) : null}
      </DrawerContent>
    </Drawer>
  )
}
