import { z } from 'zod'

export const subsystemDetailTabParamsSchema = z.object({
  entityId: z.number().int().positive()
})

export type SubsystemDetailTabParams = z.infer<typeof subsystemDetailTabParamsSchema>

export function parseSubsystemDetailTabParams(params: unknown): SubsystemDetailTabParams {
  return subsystemDetailTabParamsSchema.parse(params)
}
