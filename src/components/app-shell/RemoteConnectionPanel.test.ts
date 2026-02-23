import { describe, expect, it } from 'vitest'
import type { RemoteSshProfile } from '@/types/generated'
import { formatLastUsed, orderProfilesForDisplay } from './RemoteConnectionPanel'

function makeProfile(overrides: Partial<RemoteSshProfile>): RemoteSshProfile {
  return {
    id: 'profile-id',
    name: 'Profile',
    host: 'example.com',
    username: 'vince',
    sshPort: 22,
    remotePort: 9944,
    authMode: 'key',
    autoReconnectEnabled: false,
    ...overrides
  }
}

describe('orderProfilesForDisplay', () => {
  it('prioritizes active profile over recency', () => {
    const profiles: RemoteSshProfile[] = [
      makeProfile({ id: 'b', name: 'Recent', lastUsedAt: '2026-02-16T02:00:00.000Z' }),
      makeProfile({ id: 'a', name: 'Active', lastUsedAt: '2026-02-16T01:00:00.000Z' })
    ]

    const ordered = orderProfilesForDisplay(profiles, 'a', null)

    expect(ordered[0]?.id).toBe('a')
    expect(ordered[1]?.id).toBe('b')
  })

  it('orders non-active profiles by last used timestamp then name', () => {
    const profiles: RemoteSshProfile[] = [
      makeProfile({ id: 'c', name: 'Zulu', lastUsedAt: '2026-02-16T01:00:00.000Z' }),
      makeProfile({ id: 'a', name: 'Alpha', lastUsedAt: '2026-02-16T03:00:00.000Z' }),
      makeProfile({ id: 'b', name: 'Bravo', lastUsedAt: '2026-02-16T03:00:00.000Z' })
    ]

    const ordered = orderProfilesForDisplay(profiles, undefined, null)

    expect(ordered.map(profile => profile.id)).toEqual(['a', 'b', 'c'])
  })

  it('prioritizes default profile when no active profile is present', () => {
    const profiles: RemoteSshProfile[] = [
      makeProfile({ id: 'a', name: 'Alpha', lastUsedAt: '2026-02-16T03:00:00.000Z' }),
      makeProfile({ id: 'b', name: 'Bravo', lastUsedAt: '2026-02-16T01:00:00.000Z' }),
      makeProfile({ id: 'c', name: 'Charlie', lastUsedAt: '2026-02-16T04:00:00.000Z' })
    ]

    const ordered = orderProfilesForDisplay(profiles, undefined, 'b')

    expect(ordered.map(profile => profile.id)).toEqual(['b', 'c', 'a'])
  })
})

describe('formatLastUsed', () => {
  it('returns null for missing timestamps', () => {
    expect(formatLastUsed(undefined)).toBeNull()
  })

  it('returns raw value for invalid timestamp strings', () => {
    expect(formatLastUsed('not-a-timestamp')).toBe('not-a-timestamp')
  })

  it('formats valid timestamps using local date formatting', () => {
    const source = '2026-02-16T04:30:00.000Z'
    expect(formatLastUsed(source)).toBe(new Date(source).toLocaleString())
  })
})
