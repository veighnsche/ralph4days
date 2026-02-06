import { z } from "zod";

export const disciplineSchema = z.object({
  name: z.string(),
  displayName: z.string().min(1, "Display name is required"),
  acronym: z.string().min(1, "Acronym is required"),
  icon: z.string(),
  color: z.string(),
});

export type DisciplineFormData = z.infer<typeof disciplineSchema>;
