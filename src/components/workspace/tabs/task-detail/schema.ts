import { z } from 'zod'

export const taskDetailTabParamsSchema = z.object({
  entityId: z.number().int().positive()
})

export type TaskDetailTabParams = z.infer<typeof taskDetailTabParamsSchema>

export function parseTaskDetailTabParams(params: unknown): TaskDetailTabParams {
  return taskDetailTabParamsSchema.parse(params)
}
