export const TERMINAL_BRIDGE_COMMANDS = {
  startSession: 'terminal_bridge_start_session',
  startTaskSession: 'terminal_bridge_start_task_session',
  startHumanSession: 'terminal_bridge_start_human_session',
  listModels: 'terminal_bridge_list_models',
  sendInput: 'terminal_bridge_send_input',
  resize: 'terminal_bridge_resize',
  terminate: 'terminal_bridge_terminate',
  emitSystemMessage: 'terminal_bridge_emit_system_message'
} as const

export const TERMINAL_BRIDGE_EVENTS = {
  output: 'terminal_bridge:output',
  closed: 'terminal_bridge:closed'
} as const
