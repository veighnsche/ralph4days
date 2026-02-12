import { z } from 'zod'

export const featureDetailTabParamsSchema = z.object({
  entityId: z.string().trim().min(1)
})

export type FeatureDetailTabParams = z.infer<typeof featureDetailTabParamsSchema>

export function parseFeatureDetailTabParams(params: unknown): FeatureDetailTabParams {
  return featureDetailTabParamsSchema.parse(params)
}
