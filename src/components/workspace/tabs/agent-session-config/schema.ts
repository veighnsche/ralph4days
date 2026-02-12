import { z } from 'zod'

const agentSchema = z.enum(['claude', 'codex'])
const effortSchema = z.enum(['low', 'medium', 'high'])
const permissionLevelSchema = z.enum(['safe', 'balanced', 'auto', 'full_auto'])

export const agentSessionConfigTabParamsSchema = z
  .object({
    agent: agentSchema.optional(),
    model: z.string().trim().min(1).optional(),
    effort: effortSchema.optional(),
    thinking: z.boolean().optional(),
    permissionLevel: permissionLevelSchema.optional()
  })
  .strict()

export type AgentSessionConfigTabParams = z.infer<typeof agentSessionConfigTabParamsSchema>

export function parseAgentSessionConfigTabParams(params: unknown): AgentSessionConfigTabParams {
  if (params == null) return {}
  return agentSessionConfigTabParamsSchema.parse(params)
}
