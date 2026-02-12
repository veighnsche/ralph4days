import { z } from 'zod'
import type { Task } from '@/types/generated'

const taskEntitySchema = z.custom<Task>(value => typeof value === 'object' && value != null && !Array.isArray(value), {
  error: 'Invalid task detail tab params.entity'
})

export const taskDetailTabParamsSchema = z.object({
  entityId: z.number().int().positive(),
  entity: taskEntitySchema.optional()
})

export type TaskDetailTabParams = z.infer<typeof taskDetailTabParamsSchema>

export function parseTaskDetailTabParams(params: unknown): TaskDetailTabParams {
  return taskDetailTabParamsSchema.parse(params)
}
