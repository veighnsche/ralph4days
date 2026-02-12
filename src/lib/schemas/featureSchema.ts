import { z } from 'zod'
import { normalizeFeatureName } from '@/lib/acronym'
import { featureNameValidation } from './commonSchemas'

export const featureSchema = z.object({
  name: featureNameValidation.transform(normalizeFeatureName),
  displayName: z.string().min(1, 'Display name is required'),
  acronym: z.string().min(1, 'Acronym is required'),
  description: z.string()
})

export type FeatureFormData = z.infer<typeof featureSchema>
