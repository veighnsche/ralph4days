import { AlertCircle, Ban, CheckCircle2, Circle, Clock, type LucideIcon, Play, Slash } from "lucide-react";
import type { InferredTaskStatus, PRDTask } from "@/types/prd";

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

export const INFERRED_STATUS_CONFIG = {
  ready: {
    label: "Ready",
    icon: CheckCircle2,
    color: "var(--status-done)", // Green
    bgColor: "color-mix(in oklch, var(--status-done) 15%, transparent)",
    description: "All dependencies met - can be claimed",
  },
  waiting_on_deps: {
    label: "Waiting on Deps",
    icon: Clock,
    color: "var(--priority-medium)", // Yellow/Orange
    bgColor: "color-mix(in oklch, var(--priority-medium) 15%, transparent)",
    description: "Waiting for dependencies to complete",
  },
  externally_blocked: {
    label: "Blocked",
    icon: AlertCircle,
    color: "var(--status-blocked)", // Red
    bgColor: "color-mix(in oklch, var(--status-blocked) 15%, transparent)",
    description: "Externally blocked",
  },
  in_progress: {
    label: "In Progress",
    icon: Play,
    color: "var(--status-in-progress)", // Blue
    bgColor: "color-mix(in oklch, var(--status-in-progress) 15%, transparent)",
    description: "Currently being worked on",
  },
  done: {
    label: "Done",
    icon: CheckCircle2,
    color: "var(--status-done)", // Green
    bgColor: "color-mix(in oklch, var(--status-done) 15%, transparent)",
    description: "Completed",
  },
  skipped: {
    label: "Skipped",
    icon: Slash,
    color: "var(--status-skipped)", // Gray
    bgColor: "color-mix(in oklch, var(--status-skipped) 15%, transparent)",
    description: "Intentionally skipped",
  },
} as const satisfies Record<
  InferredTaskStatus,
  { label: string; icon: LucideIcon; color: string; bgColor: string; description: string }
>;

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
