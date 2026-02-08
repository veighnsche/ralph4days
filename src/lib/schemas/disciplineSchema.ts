import { z } from 'zod'

const mcpServerSchema = z.object({
  name: z.string().min(1, 'Server name is required'),
  command: z.string().min(1, 'Command is required'),
  args: z.array(z.string()),
  env: z.record(z.string(), z.string())
})

export const disciplineSchema = z.object({
  name: z.string(),
  displayName: z.string().min(1, 'Display name is required'),
  acronym: z.string().min(1, 'Acronym is required'),
  icon: z.string(),
  color: z.string(),
  systemPrompt: z.string().optional(),
  skills: z.array(z.string()).optional(),
  conventions: z.string().optional(),
  mcpServers: z.array(mcpServerSchema).optional()
})

export type DisciplineFormData = z.infer<typeof disciplineSchema>
export type McpServerData = z.infer<typeof mcpServerSchema>
