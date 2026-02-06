import * as LucideIcons from "lucide-react";
import { useEffect, useState } from "react";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { NativeSelect } from "@/components/ui/native-select";
import type { DisciplineConfig } from "@/hooks/useDisciplines";

export interface DisciplineFormData {
  name: string;
  display_name: string;
  acronym: string;
  icon: string;
  color: string;
}

export interface DisciplineFormProps {
  initialData?: Partial<DisciplineConfig>;
  onChange: (data: DisciplineFormData) => void;
  disabled?: boolean;
}

// Common icon options for disciplines
const COMMON_ICONS = [
  "Code",
  "Palette",
  "Database",
  "Shield",
  "TestTube",
  "BookOpen",
  "Wrench",
  "Rocket",
  "Monitor",
  "Cloud",
  "Cpu",
  "Settings",
  "FileText",
  "Layers",
  "Package",
  "Target",
] as const;

// Common color options
const COMMON_COLORS = [
  { name: "Blue", value: "#3b82f6" },
  { name: "Green", value: "#22c55e" },
  { name: "Yellow", value: "#eab308" },
  { name: "Red", value: "#ef4444" },
  { name: "Purple", value: "#a855f7" },
  { name: "Pink", value: "#ec4899" },
  { name: "Orange", value: "#f97316" },
  { name: "Teal", value: "#14b8a6" },
  { name: "Indigo", value: "#6366f1" },
  { name: "Cyan", value: "#06b6d4" },
] as const;

/**
 * Discipline creation/edit form.
 * Used by both create and edit modals (mode determined by initialData presence).
 */
export function DisciplineForm({ initialData, onChange, disabled }: DisciplineFormProps) {
  const [formData, setFormData] = useState<DisciplineFormData>({
    name: initialData?.name || "",
    display_name: initialData?.displayName || "",
    acronym: initialData?.acronym || "",
    icon: initialData?.icon ? String(initialData.icon.displayName || "Code") : "Code",
    color: initialData?.color || "#3b82f6",
  });

  useEffect(() => {
    if (initialData) {
      setFormData({
        name: initialData.name || "",
        display_name: initialData.displayName || "",
        acronym: initialData.acronym || "",
        icon: initialData.icon ? String(initialData.icon.displayName || "Code") : "Code",
        color: initialData.color || "#3b82f6",
      });
    }
  }, [initialData]);

  useEffect(() => {
    onChange(formData);
  }, [formData, onChange]);

  // Get the icon component for preview
  const IconComponent = LucideIcons[formData.icon as keyof typeof LucideIcons] as LucideIcons.LucideIcon;

  return (
    <div className="space-y-3">
      {/* Display Name */}
      <div className="space-y-2">
        <Label htmlFor="display_name">
          Display Name <span className="text-destructive">*</span>
        </Label>
        <Input
          id="display_name"
          value={formData.display_name}
          onChange={(e) => setFormData({ ...formData, display_name: e.target.value })}
          placeholder="Enter discipline display name"
          required
          disabled={disabled}
        />
        <p className="text-xs text-muted-foreground">The human-readable name shown in the UI</p>
      </div>

      {/* Acronym */}
      <div className="space-y-2">
        <Label htmlFor="acronym">
          Acronym <span className="text-destructive">*</span>
        </Label>
        <Input
          id="acronym"
          value={formData.acronym}
          onChange={(e) => setFormData({ ...formData, acronym: e.target.value.toUpperCase() })}
          placeholder="FRNT (3-4 uppercase letters)"
          maxLength={4}
          required
          className="font-mono"
          disabled={disabled}
        />
        <p className="text-xs text-muted-foreground">3-4 uppercase letters for task IDs (e.g., FRNT, BACK, TEST)</p>
      </div>

      {/* Name (Internal ID) */}
      <div className="space-y-2">
        <Label htmlFor="name">Internal Name</Label>
        <Input
          id="name"
          value={formData.name}
          onChange={(e) => setFormData({ ...formData, name: e.target.value })}
          placeholder="auto-generated-from-display-name"
          disabled={disabled || !!initialData?.name}
        />
        <p className="text-xs text-muted-foreground">
          {initialData?.name
            ? "Internal name cannot be changed after creation"
            : "Auto-generated from display name (lowercase with hyphens)"}
        </p>
      </div>

      {/* Icon */}
      <div className="space-y-2">
        <Label htmlFor="icon">
          Icon <span className="text-destructive">*</span>
        </Label>
        <div className="flex gap-2 items-center">
          <NativeSelect
            id="icon"
            value={formData.icon}
            onChange={(e) => setFormData({ ...formData, icon: e.target.value })}
            required
            className="flex-1"
            disabled={disabled}
          >
            {COMMON_ICONS.map((icon) => (
              <option key={icon} value={icon}>
                {icon}
              </option>
            ))}
          </NativeSelect>
          {IconComponent && (
            <div
              className="p-2 rounded-md shrink-0"
              style={{
                backgroundColor: `color-mix(in oklch, ${formData.color} 15%, transparent)`,
                color: formData.color,
              }}
            >
              <IconComponent className="h-5 w-5" />
            </div>
          )}
        </div>
      </div>

      {/* Color */}
      <div className="space-y-2">
        <Label htmlFor="color">
          Color <span className="text-destructive">*</span>
        </Label>
        <div className="flex gap-2 items-center">
          <NativeSelect
            id="color"
            value={formData.color}
            onChange={(e) => setFormData({ ...formData, color: e.target.value })}
            required
            className="flex-1"
            disabled={disabled}
          >
            {COMMON_COLORS.map((color) => (
              <option key={color.value} value={color.value}>
                {color.name}
              </option>
            ))}
          </NativeSelect>
          <div
            className="w-8 h-8 rounded-md border shrink-0"
            style={{ backgroundColor: formData.color }}
            title={formData.color}
          />
        </div>
      </div>
    </div>
  );
}
