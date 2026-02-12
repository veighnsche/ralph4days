import type { z } from 'zod'
import { optionalLaunchOptionsSchema } from '../launch-options-schema'

export const agentSessionConfigTabParamsSchema = optionalLaunchOptionsSchema

export type AgentSessionConfigTabParams = z.infer<typeof agentSessionConfigTabParamsSchema>

export function parseAgentSessionConfigTabParams(params: unknown): AgentSessionConfigTabParams {
  return agentSessionConfigTabParamsSchema.parse(params ?? {})
}
