import type { Dispatch, SetStateAction } from 'react'
import { Button } from '@/components/ui/button'
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
import { Switch } from '@/components/ui/switch'
import type { ProfileAuthMode, ProfileDraft } from '@/components/app-shell/remote-ssh/types'

interface RemoteSshProfileEditorDialogProps {
  isOpen: boolean
  onOpenChange: (open: boolean) => void
  draft: ProfileDraft
  setDraft: Dispatch<SetStateAction<ProfileDraft>>
  importKeyFile: File | null
  setImportKeyFile: (value: File | null) => void
  importKeyPassphrase: string
  setImportKeyPassphrase: (value: string) => void
  saveImportKeyPassphrase: boolean
  setSaveImportKeyPassphrase: (value: boolean) => void
  isSavingProfile: boolean
  onSaveProfile: () => void
}

export function RemoteSshProfileEditorDialog({
  isOpen,
  onOpenChange,
  draft,
  setDraft,
  importKeyFile,
  setImportKeyFile,
  importKeyPassphrase,
  setImportKeyPassphrase,
  saveImportKeyPassphrase,
  setSaveImportKeyPassphrase,
  isSavingProfile,
  onSaveProfile
}: RemoteSshProfileEditorDialogProps) {
  return (
    <Dialog open={isOpen} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[calc(100dvh-0.75rem)] overflow-y-auto sm:max-h-[90dvh]" data-testid="ssh-profile-editor">
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
              data-testid="ssh-profile-name-input"
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
              data-testid="ssh-host-input"
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
              data-testid="ssh-username-input"
              value={draft.username}
              onChange={event => setDraft(prev => ({ ...prev, username: event.target.value }))}
              placeholder="vince"
              autoCapitalize="none"
              autoCorrect="off"
              spellCheck={false}
            />
          </Field>

          <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
            <Field>
              <FieldLabel>SSH Port</FieldLabel>
              <Input
                data-testid="ssh-port-input"
                value={draft.sshPort}
                onChange={event => setDraft(prev => ({ ...prev, sshPort: event.target.value }))}
                inputMode="numeric"
                pattern="[0-9]*"
              />
            </Field>

            <Field>
              <FieldLabel>Ralphd Port</FieldLabel>
              <Input
                data-testid="ralphd-port-input"
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
              <SelectTrigger data-testid="ssh-auth-mode-select">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="key" data-testid="ssh-auth-mode-key">
                  Key
                </SelectItem>
                <SelectItem value="password" data-testid="ssh-auth-mode-password">
                  Password
                </SelectItem>
              </SelectContent>
            </Select>
            <FieldDescription>Choose how this profile authenticates over SSH.</FieldDescription>
          </Field>

          <details className="rounded-lg border px-3 py-2">
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
                    data-testid="ssh-identity-file-input"
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
                <FieldDescription>Imported key material is stored as secret and bound to this profile.</FieldDescription>
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
                  <Field className="flex items-center justify-between rounded-lg border px-3 py-2">
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

              <Field className="flex items-center justify-between rounded-lg border px-3 py-2">
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
                  data-testid="ssh-password-input"
                  type="password"
                  value={draft.password}
                  onChange={event => setDraft(prev => ({ ...prev, password: event.target.value }))}
                />
              </Field>

              <Field className="flex items-center justify-between rounded-lg border px-3 py-2">
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

          <Field className="flex items-center justify-between rounded-lg border px-3 py-2">
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
          <Button
            variant="outline"
            onClick={() => onOpenChange(false)}
            disabled={isSavingProfile}
            data-testid="ssh-profile-cancel-button"
            className="w-full sm:w-auto">
            Cancel
          </Button>
          <Button onClick={onSaveProfile} disabled={isSavingProfile} data-testid="ssh-profile-save-button" className="w-full sm:w-auto">
            {isSavingProfile ? 'Saving...' : 'Save Profile'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
