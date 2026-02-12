import { z } from 'zod'
import { optionalLaunchOptionsSchema } from '../launch-options-schema'

const terminalTabInputSchema = optionalLaunchOptionsSchema
  .extend({
    taskId: z.number().int().positive().optional(),
    initPrompt: z.string().optional(),
    title: z.string().trim().min(1).optional()
  })
  .strict()
export const terminalTabParamsSchema = terminalTabInputSchema.transform(params => ({
  ...params,
  agent: params.agent ?? 'claude',
  permissionLevel: params.permissionLevel ?? 'balanced',
  kind: params.taskId != null ? 'task_execution' : 'manual'
}))

export type TerminalTabInput = z.input<typeof terminalTabParamsSchema>
export type TerminalTabParams = z.output<typeof terminalTabParamsSchema>

export function parseTerminalTabParams(params: unknown): TerminalTabParams {
  return terminalTabParamsSchema.parse(params ?? {})
}
