import { z } from 'zod'

export const disciplineDetailTabParamsSchema = z.object({
  entityId: z.string().trim().min(1)
})

export type DisciplineDetailTabParams = z.infer<typeof disciplineDetailTabParamsSchema>

export function parseDisciplineDetailTabParams(params: unknown): DisciplineDetailTabParams {
  return disciplineDetailTabParamsSchema.parse(params)
}
