import { z } from "zod";

export const taskSchema = z.object({
  feature: z.string().min(1, "Feature is required"),
  discipline: z.string().min(1, "Discipline is required"),
  title: z.string().min(1, "Title is required"),
  description: z.string(),
  priority: z.enum(["low", "medium", "high", "critical"]),
  tags: z.array(z.string()),
  dependsOn: z.array(z.number()),
  acceptanceCriteria: z.array(z.string()),
});

export type TaskFormData = z.infer<typeof taskSchema>;
