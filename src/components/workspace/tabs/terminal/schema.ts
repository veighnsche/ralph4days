import { z } from 'zod'

const agentSchema = z.enum(['claude', 'codex'])
const effortSchema = z.enum(['low', 'medium', 'high'])
const permissionLevelSchema = z.enum(['safe', 'balanced', 'auto', 'full_auto'])

export const terminalTabParamsSchema = z
  .object({
    agent: agentSchema.optional(),
    model: z.string().trim().min(1).optional(),
    effort: effortSchema.optional(),
    thinking: z.boolean().optional(),
    permissionLevel: permissionLevelSchema.optional(),
    taskId: z.number().int().positive().optional(),
    initPrompt: z.string().optional(),
    title: z.string().trim().min(1).optional()
  })
  .strict()

export type TerminalTabParams = z.infer<typeof terminalTabParamsSchema>

export function parseTerminalTabParams(params: unknown): TerminalTabParams {
  if (params == null) return {}
  return terminalTabParamsSchema.parse(params)
}
