import { Circle, Play, CheckCircle2, Ban, Slash } from "lucide-react";
import type { PRDTask } from "@/types/prd";

export const STATUS_CONFIG = {
  pending: {
    label: "Pending",
    icon: Circle,
    color: "hsl(var(--status-pending))",
    bgColor: "hsl(var(--status-pending) / 0.1)",
  },
  in_progress: {
    label: "In Progress",
    icon: Play,
    color: "hsl(var(--status-in-progress))",
    bgColor: "hsl(var(--status-in-progress) / 0.1)",
  },
  blocked: {
    label: "Blocked",
    icon: Ban,
    color: "hsl(var(--status-blocked))",
    bgColor: "hsl(var(--status-blocked) / 0.1)",
  },
  done: {
    label: "Done",
    icon: CheckCircle2,
    color: "hsl(var(--status-done))",
    bgColor: "hsl(var(--status-done) / 0.1)",
  },
  skipped: {
    label: "Skipped",
    icon: Slash,
    color: "hsl(var(--status-skipped))",
    bgColor: "hsl(var(--status-skipped) / 0.1)",
  },
} as const;

export const PRIORITY_CONFIG = {
  low: {
    label: "Low",
    color: "hsl(var(--priority-low))",
    bgColor: "hsl(var(--priority-low) / 0.15)",
  },
  medium: {
    label: "Medium",
    color: "hsl(var(--priority-medium))",
    bgColor: "hsl(var(--priority-medium) / 0.15)",
  },
  high: {
    label: "High",
    color: "hsl(var(--priority-high))",
    bgColor: "hsl(var(--priority-high) / 0.15)",
  },
  critical: {
    label: "Critical",
    color: "hsl(var(--priority-critical))",
    bgColor: "hsl(var(--priority-critical) / 0.15)",
  },
} as const;

export const COLUMN_DEFINITIONS = [
  {
    status: "pending" as PRDTask["status"],
    ...STATUS_CONFIG.pending,
  },
  {
    status: "in_progress" as PRDTask["status"],
    ...STATUS_CONFIG.in_progress,
  },
  {
    status: "blocked" as PRDTask["status"],
    ...STATUS_CONFIG.blocked,
  },
  {
    status: "done" as PRDTask["status"],
    ...STATUS_CONFIG.done,
  },
  {
    status: "skipped" as PRDTask["status"],
    ...STATUS_CONFIG.skipped,
  },
] as const;
