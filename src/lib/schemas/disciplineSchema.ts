import { z } from 'zod'
import { acronymValidation } from './commonSchemas'

const mcpServerSchema = z.object({
  name: z.string().min(1, 'Server name is required'),
  command: z.string().min(1, 'Command is required'),
  args: z.array(z.string()),
  env: z.record(z.string(), z.string())
})

export const disciplineSchema = z.object({
  name: z.string(),
  displayName: z.string().min(1, 'Display name is required'),
  acronym: acronymValidation,
  icon: z.string(),
  color: z.string(),
  systemPrompt: z.string().optional(),
  agent: z.enum(['claude', 'codex']).optional(),
  model: z.string().optional(),
  effort: z.enum(['low', 'medium', 'high']).optional(),
  thinking: z.boolean().optional(),
  skills: z.array(z.string()).optional(),
  conventions: z.string().optional(),
  mcpServers: z.array(mcpServerSchema).optional()
})

export type DisciplineFormData = z.infer<typeof disciplineSchema>
export type McpServerData = z.infer<typeof mcpServerSchema>
