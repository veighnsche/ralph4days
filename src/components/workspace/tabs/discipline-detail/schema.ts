import { z } from 'zod'

export const disciplineDetailTabParamsSchema = z.object({
  entityId: z.number().int().positive()
})

export type DisciplineDetailTabParams = z.infer<typeof disciplineDetailTabParamsSchema>

export function parseDisciplineDetailTabParams(params: unknown): DisciplineDetailTabParams {
  return disciplineDetailTabParamsSchema.parse(params)
}
