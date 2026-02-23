import type { RemoteSshAuthMode, RemoteSshProfile } from '@/types/generated'

export type ProfileAuthMode = RemoteSshAuthMode

export interface ProfileDraft {
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

export interface ConnectPromptState {
  profile: RemoteSshProfile
  password: string
  keyPassphrase: string
}
