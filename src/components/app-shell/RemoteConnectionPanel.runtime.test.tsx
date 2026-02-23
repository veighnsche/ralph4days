import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { cleanup, render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useRemoteSshPreferences } from '@/hooks/preferences/useRemoteSshPreferences'
import { RalphIpcError } from '@/lib/tauri/invoke'
import type { RemoteSshProfile, RemoteSshStatus, RemoteStatus } from '@/types/generated'
import { RemoteConnectionPanel } from './RemoteConnectionPanel'

const { isMobileMock, useInvokeMock, tauriInvokeMock } = vi.hoisted(() => ({
  isMobileMock: vi.fn(),
  useInvokeMock: vi.fn(),
  tauriInvokeMock: vi.fn()
}))

vi.mock('@/hooks/api', () => ({
  useInvoke: (...args: unknown[]) => useInvokeMock(...args)
}))

vi.mock('@/hooks/use-mobile', () => ({
  useIsMobile: () => isMobileMock()
}))

vi.mock('@/components/ui/drawer', () => ({
  Drawer: ({ open, children }: { open?: boolean; children: React.ReactNode }) => (open ? <div>{children}</div> : null),
  DrawerContent: ({ children, ...props }: React.ComponentProps<'div'>) => <div {...props}>{children}</div>,
  DrawerHeader: ({ children, ...props }: React.ComponentProps<'div'>) => <div {...props}>{children}</div>,
  DrawerTitle: ({ children, ...props }: React.ComponentProps<'div'>) => <div {...props}>{children}</div>,
  DrawerDescription: ({ children, ...props }: React.ComponentProps<'div'>) => <div {...props}>{children}</div>
}))

vi.mock('@/lib/tauri/invoke', async () => {
  const actual = await vi.importActual<typeof import('@/lib/tauri/invoke')>('@/lib/tauri/invoke')
  return {
    ...actual,
    tauriInvoke: (...args: unknown[]) => tauriInvokeMock(...args)
  }
})

const disconnectedRemoteStatus: RemoteStatus = { connected: false }
const disconnectedSshStatus: RemoteSshStatus = { active: false }

function makeProfile(overrides: Partial<RemoteSshProfile>): RemoteSshProfile {
  return {
    id: 'profile-1',
    name: 'Remote Host',
    host: '127.0.0.1',
    username: 'ralph',
    sshPort: 22,
    remotePort: 9944,
    authMode: 'key',
    identityRef: 'keychain:test-profile',
    autoReconnectEnabled: false,
    ...overrides
  }
}

function renderPanel(
  status: RemoteStatus = disconnectedRemoteStatus,
  sshStatus: RemoteSshStatus = disconnectedSshStatus
) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false }
    }
  })

  return render(
    <QueryClientProvider client={queryClient}>
      <RemoteConnectionPanel
        status={status}
        sshStatus={sshStatus}
        statusError={null}
        onConnected={vi.fn()}
        onDisconnected={vi.fn()}
      />
    </QueryClientProvider>
  )
}

describe('RemoteConnectionPanel runtime harness', () => {
  beforeEach(() => {
    cleanup()
    localStorage.clear()
    useRemoteSshPreferences.getState().setDefaultProfileId(null)
    isMobileMock.mockReset()
    isMobileMock.mockReturnValue(false)
    useInvokeMock.mockReset()
    tauriInvokeMock.mockReset()
  })

  it('renders SSH page in a dedicated scroll viewport', () => {
    useInvokeMock.mockImplementation((command: string) => {
      if (command !== 'remote_ssh_profile_list') {
        throw new Error(`Unexpected useInvoke command: ${command}`)
      }
      return { data: [], error: null, isLoading: false }
    })

    renderPanel()

    expect(screen.getByTestId('ssh-page-scroll-root')).toHaveClass('h-dvh', 'overflow-y-scroll', 'fixed', 'inset-0')
  })

  it('supports profile create/edit/connect/delete button flows', async () => {
    const user = userEvent.setup()
    let profiles: RemoteSshProfile[] = []

    useInvokeMock.mockImplementation((command: string) => {
      if (command !== 'remote_ssh_profile_list') {
        throw new Error(`Unexpected useInvoke command: ${command}`)
      }
      return { data: profiles, error: null, isLoading: false }
    })

    const savedProfile = makeProfile({ id: 'profile-1', name: 'E2E Host' })
    tauriInvokeMock.mockImplementation(async (command: string) => {
      if (command === 'remote_ssh_profile_upsert') return savedProfile
      if (command === 'remote_ssh_profile_delete') return undefined
      return undefined
    })

    renderPanel()

    await user.click(screen.getByTestId('ssh-new-profile-button'))
    await user.type(screen.getByTestId('ssh-profile-name-input'), 'E2E Host')
    await user.clear(screen.getByTestId('ssh-host-input'))
    await user.type(screen.getByTestId('ssh-host-input'), '127.0.0.1')
    await user.clear(screen.getByTestId('ssh-username-input'))
    await user.type(screen.getByTestId('ssh-username-input'), 'ralph')
    await user.click(screen.getByTestId('ssh-profile-save-button'))

    await waitFor(() =>
      expect(tauriInvokeMock).toHaveBeenCalledWith(
        'remote_ssh_profile_upsert',
        expect.objectContaining({
          name: 'E2E Host',
          host: '127.0.0.1',
          username: 'ralph'
        })
      )
    )

    cleanup()
    profiles = [savedProfile]
    renderPanel()

    await user.click(screen.getByTestId(`ssh-profile-edit-${savedProfile.id}`))
    expect(screen.getByTestId('ssh-profile-editor')).toBeInTheDocument()
    await user.click(screen.getByTestId('ssh-profile-cancel-button'))

    await user.click(screen.getByTestId(`ssh-profile-connect-${savedProfile.id}`))
    expect(screen.getByTestId('ssh-connect-dialog')).toBeInTheDocument()
    await user.click(screen.getByTestId('ssh-connect-cancel-button'))

    await user.click(screen.getByTestId(`ssh-profile-delete-${savedProfile.id}`))
    expect(screen.getByTestId('ssh-delete-dialog')).toBeInTheDocument()
    await user.click(screen.getByTestId('ssh-delete-confirm-button'))

    await waitFor(() =>
      expect(tauriInvokeMock).toHaveBeenCalledWith('remote_ssh_profile_delete', { profileId: 'profile-1' })
    )
  })

  it('surfaces and rejects host-key challenge from connect flow', async () => {
    const user = userEvent.setup()
    const profile = makeProfile({ id: 'profile-hostkey', name: 'Hostkey Profile' })

    useInvokeMock.mockImplementation((command: string) => {
      if (command !== 'remote_ssh_profile_list') {
        throw new Error(`Unexpected useInvoke command: ${command}`)
      }
      return { data: [profile], error: null, isLoading: false }
    })

    const hostKeyChallenge = {
      challengeId: 'challenge-1',
      host: '127.0.0.1',
      sshPort: 22,
      algorithm: 'ssh-ed25519',
      fingerprintSha256: 'SHA256:abcdef123456',
      knownHostsTargetPath: '/tmp/known_hosts',
      expiresAt: '2030-01-01T00:00:00.000Z'
    }

    tauriInvokeMock.mockImplementation(async (command: string) => {
      if (command === 'remote_ssh_profile_connect') {
        throw new RalphIpcError(
          'remote_ssh_profile_connect',
          {
            code: 8100,
            message: 'host key challenge',
            location: { file: '<test>', line: 1, column: 1 },
            context: [{ key: 'ssh_hostkey_challenge', value: hostKeyChallenge }]
          },
          hostKeyChallenge
        )
      }
      if (command === 'remote_ssh_hostkey_challenge_reject') return undefined
      return undefined
    })

    renderPanel()

    await user.click(screen.getByTestId('ssh-quick-connect-button'))
    expect(screen.getByTestId('ssh-connect-dialog')).toBeInTheDocument()

    await user.click(screen.getByTestId('ssh-connect-now-button'))
    await waitFor(() => expect(screen.getByTestId('ssh-hostkey-dialog')).toBeInTheDocument())

    await user.click(screen.getByTestId('ssh-hostkey-reject-button'))
    await waitFor(() =>
      expect(tauriInvokeMock).toHaveBeenCalledWith('remote_ssh_hostkey_challenge_reject', {
        challengeId: 'challenge-1'
      })
    )
  })

  it('shows default indicator and allows switching default profile', async () => {
    const user = userEvent.setup()
    const profileA = makeProfile({ id: 'profile-a', name: 'Alpha' })
    const profileB = makeProfile({ id: 'profile-b', name: 'Bravo' })

    useInvokeMock.mockImplementation((command: string) => {
      if (command !== 'remote_ssh_profile_list') {
        throw new Error(`Unexpected useInvoke command: ${command}`)
      }
      return { data: [profileA, profileB], error: null, isLoading: false }
    })

    useRemoteSshPreferences.getState().setDefaultProfileId('profile-a')

    renderPanel()

    expect(screen.getByTestId('ssh-profile-default-indicator-profile-a')).toBeInTheDocument()
    expect(screen.getByTestId('ssh-profile-set-default-profile-b')).toBeInTheDocument()

    await user.click(screen.getByTestId('ssh-profile-set-default-profile-b'))

    await waitFor(() => {
      expect(screen.getByTestId('ssh-profile-default-indicator-profile-b')).toBeInTheDocument()
    })
  })

  it('uses search toggle and profile actions dialog in mobile mode', async () => {
    const user = userEvent.setup()
    const profileA = makeProfile({ id: 'profile-a', name: 'Alpha' })
    const profileB = makeProfile({ id: 'profile-b', name: 'Bravo' })

    isMobileMock.mockReturnValue(true)
    useInvokeMock.mockImplementation((command: string) => {
      if (command !== 'remote_ssh_profile_list') {
        throw new Error(`Unexpected useInvoke command: ${command}`)
      }
      return { data: [profileA, profileB], error: null, isLoading: false }
    })

    useRemoteSshPreferences.getState().setDefaultProfileId('profile-a')

    renderPanel()

    expect(screen.getByTestId('ssh-search-toggle')).toBeInTheDocument()
    expect(screen.queryByTestId('ssh-search-input')).not.toBeInTheDocument()

    await user.click(screen.getByTestId('ssh-search-toggle'))
    expect(screen.getByTestId('ssh-search-input')).toBeInTheDocument()

    await user.type(screen.getByTestId('ssh-search-input'), 'Bravo')
    expect(screen.queryByTestId('ssh-profile-card-profile-a')).not.toBeInTheDocument()
    expect(screen.getByTestId('ssh-profile-card-profile-b')).toBeInTheDocument()

    await user.click(screen.getByTestId('ssh-search-clear'))
    await user.click(screen.getByTestId('ssh-profile-actions-profile-b'))
    expect(screen.getByTestId('ssh-profile-actions-dialog')).toBeInTheDocument()

    await user.click(screen.getByTestId('ssh-profile-action-set-default-profile-b'))

    await waitFor(() => {
      expect(screen.getByTestId('ssh-profile-default-indicator-profile-b')).toBeInTheDocument()
    })
  })

  it('replaces connect with edit when key auth has no installed key', async () => {
    const profile = makeProfile({ id: 'profile-no-key', name: 'NoKey', identityRef: undefined, identityFile: undefined })

    useInvokeMock.mockImplementation((command: string) => {
      if (command !== 'remote_ssh_profile_list') {
        throw new Error(`Unexpected useInvoke command: ${command}`)
      }
      return { data: [profile], error: null, isLoading: false }
    })

    renderPanel()

    expect(screen.queryByTestId('ssh-profile-connect-profile-no-key')).not.toBeInTheDocument()
    expect(screen.getByTestId('ssh-profile-edit-required-profile-no-key')).toBeInTheDocument()
  })
})
