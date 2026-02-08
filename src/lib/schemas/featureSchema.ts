import { z } from 'zod'
import { normalizeFeatureName } from '@/lib/acronym'

export const featureSchema = z.object({
  name: z
    .string()
    .refine(name => !(name.includes('/') || name.includes(':') || name.includes('\\')), {
      message: 'Feature name cannot contain /, :, or \\'
    })
    .transform(normalizeFeatureName),
  displayName: z.string().min(1, 'Display name is required'),
  acronym: z.string().min(1, 'Acronym is required'),
  description: z.string()
})

export type FeatureFormData = z.infer<typeof featureSchema>
