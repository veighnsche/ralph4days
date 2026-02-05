import type { LucideIcon } from "lucide-react";
import {
  BookOpen,
  Circle,
  Cloud,
  Database,
  FlaskConical,
  Megaphone,
  Monitor,
  Palette,
  Plug,
  Server,
  Shield,
} from "lucide-react";

/** Maps icon string names (from disciplines.yaml) to Lucide React components */
const ICON_REGISTRY: Record<string, LucideIcon> = {
  Monitor,
  Server,
  Database,
  FlaskConical,
  Cloud,
  Shield,
  BookOpen,
  Palette,
  Megaphone,
  Plug,
  Circle,
};

/** Resolve an icon name string to a Lucide component. Falls back to Circle. */
export function resolveIcon(name: string): LucideIcon {
  return ICON_REGISTRY[name] ?? Circle;
}
