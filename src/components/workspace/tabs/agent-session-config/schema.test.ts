import { describe, expect, it } from 'vitest'
import { parseAgentSessionConfigTabParams } from './schema'

describe('agent session config tab params schema', () => {
  it('fails hard on invalid permission level', () => {
    expect(() => parseAgentSessionConfigTabParams({ permissionLevel: 'delegated' })).toThrow()
  })

  it('fails hard on unknown keys', () => {
    expect(() =>
      parseAgentSessionConfigTabParams({
        permissionLevel: 'balanced',
        unsupported: true
      })
    ).toThrow()
  })
})
