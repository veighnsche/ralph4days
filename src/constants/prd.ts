import { Ban, CheckCircle2, Circle, Play, Slash } from "lucide-react";
import type { PRDTask } from "@/types/prd";

export const STATUS_CONFIG = {
  pending: {
    label: "Pending",
    icon: Circle,
    color: "var(--status-pending)",
    bgColor: "color-mix(in oklch, var(--status-pending) 10%, transparent)",
  },
  in_progress: {
    label: "In Progress",
    icon: Play,
    color: "var(--status-in-progress)",
    bgColor: "color-mix(in oklch, var(--status-in-progress) 10%, transparent)",
  },
  blocked: {
    label: "Blocked",
    icon: Ban,
    color: "var(--status-blocked)",
    bgColor: "color-mix(in oklch, var(--status-blocked) 10%, transparent)",
  },
  done: {
    label: "Done",
    icon: CheckCircle2,
    color: "var(--status-done)",
    bgColor: "color-mix(in oklch, var(--status-done) 10%, transparent)",
  },
  skipped: {
    label: "Skipped",
    icon: Slash,
    color: "var(--status-skipped)",
    bgColor: "color-mix(in oklch, var(--status-skipped) 10%, transparent)",
  },
} as const;

export const PRIORITY_CONFIG = {
  low: {
    label: "Low",
    color: "var(--priority-low)",
    bgColor: "color-mix(in oklch, var(--priority-low) 15%, transparent)",
  },
  medium: {
    label: "Medium",
    color: "var(--priority-medium)",
    bgColor: "color-mix(in oklch, var(--priority-medium) 15%, transparent)",
  },
  high: {
    label: "High",
    color: "var(--priority-high)",
    bgColor: "color-mix(in oklch, var(--priority-high) 15%, transparent)",
  },
  critical: {
    label: "Critical",
    color: "var(--priority-critical)",
    bgColor: "color-mix(in oklch, var(--priority-critical) 15%, transparent)",
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
