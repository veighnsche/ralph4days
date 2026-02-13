import { z } from 'zod'

export const featureNameValidation = z
  .string()
  .refine(name => !(name.includes('/') || name.includes(':') || name.includes('\\')), {
    message: 'Subsystem name cannot contain /, :, or \\'
  })

export const acronymValidation = z
  .string()
  .trim()
  .toUpperCase()
  .regex(/^[A-Z0-9]{4}$/, 'Acronym must be exactly 4 uppercase letters or numbers')
