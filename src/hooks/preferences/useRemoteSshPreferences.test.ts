import { beforeEach, describe, expect, it } from 'vitest'
import { expectNoStoreTransitions, expectStoreTransitions } from '@/test/zustand-store-test-utils'
import { useRemoteSshPreferences } from './useRemoteSshPreferences'

describe('useRemoteSshPreferences', () => {
  beforeEach(() => {
    localStorage.clear()
    useRemoteSshPreferences.getState().setDefaultProfileId(null)
  })

  it('does not emit redundant transitions for same-value writes', () => {
    const store = useRemoteSshPreferences.getState()
    store.setDefaultProfileId('profile-1')

    expectNoStoreTransitions(useRemoteSshPreferences, () => {
      store.setDefaultProfileId('profile-1')
      store.ensureDefaultProfileId('profile-1')
      store.ensureDefaultProfileId('profile-2')
      store.setDefaultProfileId('profile-1')
    })
  })

  it('sets default once when empty and ignores subsequent ensure calls', () => {
    expectStoreTransitions(
      useRemoteSshPreferences,
      () => {
        useRemoteSshPreferences.getState().ensureDefaultProfileId('profile-a')
        useRemoteSshPreferences.getState().ensureDefaultProfileId('profile-b')
      },
      1
    )

    expect(useRemoteSshPreferences.getState().defaultProfileId).toBe('profile-a')
  })

  it('updates default when explicitly set', () => {
    expectStoreTransitions(
      useRemoteSshPreferences,
      () => {
        useRemoteSshPreferences.getState().setDefaultProfileId('profile-z')
      },
      1
    )

    expect(useRemoteSshPreferences.getState().defaultProfileId).toBe('profile-z')
  })
})
