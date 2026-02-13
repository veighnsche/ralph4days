import { z } from 'zod'

export const featureDetailTabParamsSchema = z.object({
  entityId: z.number().int().positive()
})

export type FeatureDetailTabParams = z.infer<typeof featureDetailTabParamsSchema>

export function parseFeatureDetailTabParams(params: unknown): FeatureDetailTabParams {
  return featureDetailTabParamsSchema.parse(params)
}
