import { describe, expect, it } from 'vitest'
import { TERMINAL_BRIDGE_COMMANDS, TERMINAL_BRIDGE_EVENTS } from './terminalBridgeContract'

describe('terminalBridgeContract', () => {
  it('keeps canonical terminal bridge command names', () => {
    expect(TERMINAL_BRIDGE_COMMANDS.startSession).toBe('terminal_bridge_start_session')
    expect(TERMINAL_BRIDGE_COMMANDS.startTaskSession).toBe('terminal_bridge_start_task_session')
    expect(TERMINAL_BRIDGE_COMMANDS.sendInput).toBe('terminal_bridge_send_input')
    expect(TERMINAL_BRIDGE_COMMANDS.resize).toBe('terminal_bridge_resize')
    expect(TERMINAL_BRIDGE_COMMANDS.terminate).toBe('terminal_bridge_terminate')
    expect(TERMINAL_BRIDGE_COMMANDS.emitSystemMessage).toBe('terminal_bridge_emit_system_message')
  })

  it('keeps canonical terminal bridge event names', () => {
    expect(TERMINAL_BRIDGE_EVENTS.output).toBe('terminal_bridge:output')
    expect(TERMINAL_BRIDGE_EVENTS.closed).toBe('terminal_bridge:closed')
  })
})
