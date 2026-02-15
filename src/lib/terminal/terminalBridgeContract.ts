export const TERMINAL_BRIDGE_COMMANDS = {
  startSession: 'terminal_start_session',
  startTaskSession: 'terminal_start_task_session',
  startHumanSession: 'terminal_start_human_session',
  resolveTaskLaunchConfig: 'terminal_resolve_task_launch_config',
  listModelFormTree: 'terminal_list_model_form_tree',
  sendInput: 'terminal_send_input',
  resize: 'terminal_resize',
  terminate: 'terminal_terminate',
  setStreamMode: 'terminal_set_stream_mode',
  replayOutput: 'terminal_replay_output',
  emitSystemMessage: 'terminal_emit_system_message'
} as const

export const TERMINAL_BRIDGE_EVENTS = {
  output: 'terminal:output',
  closed: 'terminal:closed'
} as const
