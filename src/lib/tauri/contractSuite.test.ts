import { describe, expectTypeOf, it } from 'vitest'
import type {
  PtyOutputEvent,
  RalphError,
  RalphErrorContextItem,
  RalphErrorLocation,
  RemoteWireFrame,
  TerminalBridgeReplayOutputArgs,
  TerminalBridgeReplayOutputChunk
} from '@/types/generated'

describe('IPC Contract Suite (frontend)', () => {
  it('uses JSON-safe numbers for terminal sequencing', () => {
    expectTypeOf<PtyOutputEvent['seq']>().toEqualTypeOf<number>()
    expectTypeOf<TerminalBridgeReplayOutputChunk['seq']>().toEqualTypeOf<number>()
    expectTypeOf<TerminalBridgeReplayOutputArgs['afterSeq']>().toEqualTypeOf<number>()
  })

  it('uses JSON-safe numbers for remote frame ids', () => {
    type RpcRequest = Extract<RemoteWireFrame, { type: 'rpc-request' }>
    type RpcOk = Extract<RemoteWireFrame, { type: 'rpc-ok' }>
    type RpcErr = Extract<RemoteWireFrame, { type: 'rpc-err' }>
    expectTypeOf<RpcRequest['id']>().toEqualTypeOf<number>()
    expectTypeOf<RpcOk['id']>().toEqualTypeOf<number>()
    expectTypeOf<RpcErr['id']>().toEqualTypeOf<number>()
  })

  it('surfaces a canonical structured error payload', () => {
    expectTypeOf<RalphError>().toEqualTypeOf<{
      code: number
      message: string
      location: RalphErrorLocation
      context: RalphErrorContextItem[]
      hint?: string
    }>()
  })
})
