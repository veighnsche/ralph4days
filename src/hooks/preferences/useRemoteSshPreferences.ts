import { create } from 'zustand'
import { createJSONStorage, persist } from 'zustand/middleware'

type RemoteSshPreferencesStore = {
  defaultProfileId: string | null
  setDefaultProfileId: (value: string | null) => void
  ensureDefaultProfileId: (value: string) => void
}

export const useRemoteSshPreferences = create<RemoteSshPreferencesStore>()(
  persist(
    set => ({
      defaultProfileId: null,
      setDefaultProfileId: value =>
        set(state => {
          if (state.defaultProfileId === value) return state
          return { defaultProfileId: value }
        }),
      ensureDefaultProfileId: value =>
        set(state => {
          if (state.defaultProfileId !== null) return state
          return { defaultProfileId: value }
        })
    }),
    {
      name: 'ralph.preferences.remote-ssh',
      storage: createJSONStorage(() => localStorage),
      partialize: state => ({ defaultProfileId: state.defaultProfileId })
    }
  )
)
