import { z } from 'zod'
import { normalizeFeatureName } from '@/lib/acronym'
import { acronymValidation, featureNameValidation } from './commonSchemas'

export const subsystemSchema = z.object({
  name: featureNameValidation.transform(normalizeFeatureName),
  displayName: z.string().min(1, 'Display name is required'),
  acronym: acronymValidation,
  description: z.string()
})

export type SubsystemFormData = z.infer<typeof subsystemSchema>
