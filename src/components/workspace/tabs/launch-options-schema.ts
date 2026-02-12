import { z } from 'zod'

export const agentSchema = z.enum(['claude', 'codex'])
export const effortSchema = z.enum(['low', 'medium', 'high'])
export const permissionLevelSchema = z.enum(['safe', 'balanced', 'auto', 'full_auto'])

export const optionalLaunchOptionsSchema = z
  .object({
    agent: agentSchema.optional(),
    model: z.string().trim().min(1).optional(),
    effort: effortSchema.optional(),
    thinking: z.boolean().optional(),
    permissionLevel: permissionLevelSchema.optional()
  })
  .strict()
