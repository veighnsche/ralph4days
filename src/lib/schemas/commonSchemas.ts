import { z } from 'zod'

export const featureNameValidation = z
  .string()
  .refine(name => !(name.includes('/') || name.includes(':') || name.includes('\\')), {
    message: 'Subsystem name cannot contain /, :, or \\'
  })
