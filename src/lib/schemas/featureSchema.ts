import { z } from "zod";

export const featureSchema = z.object({
  name: z.string(),
  displayName: z.string().min(1, "Display name is required"),
  acronym: z.string().min(1, "Acronym is required"),
  description: z.string(),
});

export type FeatureFormData = z.infer<typeof featureSchema>;
