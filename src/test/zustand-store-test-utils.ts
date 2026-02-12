import { expect } from 'vitest'

type StoreLike<TState> = {
  getState: () => TState
  subscribe: (listener: (state: TState, prevState: TState) => void) => () => void
}

export function countStoreTransitions<TState>(store: StoreLike<TState>, run: () => void): number {
  let calls = 0
  const unsubscribe = store.subscribe(() => {
    calls += 1
  })

  try {
    run()
    return calls
  } finally {
    unsubscribe()
  }
}

export function expectNoStoreTransitions<TState>(store: StoreLike<TState>, run: () => void) {
  expect(countStoreTransitions(store, run)).toBe(0)
}

export function expectStoreTransitions<TState>(store: StoreLike<TState>, run: () => void, expected: number) {
  expect(countStoreTransitions(store, run)).toBe(expected)
}
