import {
  Ban,
  BookOpen,
  CheckCircle2,
  Circle,
  Cloud,
  Database,
  FlaskConical,
  Megaphone,
  Monitor,
  Palette,
  Play,
  Plug,
  Server,
  Shield,
  Slash,
} from "lucide-react";
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

export const DISCIPLINE_CONFIG = {
  frontend: {
    label: "frontend",
    icon: Monitor,
    color: "var(--discipline-frontend)",
    bgColor: "color-mix(in oklch, var(--discipline-frontend) 15%, transparent)",
  },
  backend: {
    label: "backend ",
    icon: Server,
    color: "var(--discipline-backend)",
    bgColor: "color-mix(in oklch, var(--discipline-backend) 15%, transparent)",
  },
  database: {
    label: "database",
    icon: Database,
    color: "var(--discipline-database)",
    bgColor: "color-mix(in oklch, var(--discipline-database) 15%, transparent)",
  },
  testing: {
    label: "testing ",
    icon: FlaskConical,
    color: "var(--discipline-testing)",
    bgColor: "color-mix(in oklch, var(--discipline-testing) 15%, transparent)",
  },
  infra: {
    label: "infra   ",
    icon: Cloud,
    color: "var(--discipline-infrastructure)",
    bgColor: "color-mix(in oklch, var(--discipline-infrastructure) 15%, transparent)",
  },
  security: {
    label: "security",
    icon: Shield,
    color: "var(--discipline-security)",
    bgColor: "color-mix(in oklch, var(--discipline-security) 15%, transparent)",
  },
  docs: {
    label: "docs    ",
    icon: BookOpen,
    color: "var(--discipline-documentation)",
    bgColor: "color-mix(in oklch, var(--discipline-documentation) 15%, transparent)",
  },
  design: {
    label: "design  ",
    icon: Palette,
    color: "var(--discipline-design)",
    bgColor: "color-mix(in oklch, var(--discipline-design) 15%, transparent)",
  },
  promo: {
    label: "promo   ",
    icon: Megaphone,
    color: "var(--discipline-marketing)",
    bgColor: "color-mix(in oklch, var(--discipline-marketing) 15%, transparent)",
  },
  api: {
    label: "api     ",
    icon: Plug,
    color: "var(--discipline-api)",
    bgColor: "color-mix(in oklch, var(--discipline-api) 15%, transparent)",
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
