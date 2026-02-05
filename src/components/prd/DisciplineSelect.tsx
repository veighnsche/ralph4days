import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { DISCIPLINE_CONFIG } from "@/constants/prd";

interface DisciplineSelectProps {
  value: string;
  onChange: (value: string) => void;
}

type DisciplineKey = keyof typeof DISCIPLINE_CONFIG;

export function DisciplineSelect({ value, onChange }: DisciplineSelectProps) {
  const [disciplines, setDisciplines] = useState<string[]>([]);

  useEffect(() => {
    invoke<string[]>("get_available_disciplines")
      .then(setDisciplines)
      .catch((err) => {
        console.error("Failed to load disciplines:", err);
        // Fallback to hardcoded list
        setDisciplines([
          "frontend",
          "backend",
          "database",
          "testing",
          "infrastructure",
          "security",
          "documentation",
          "design",
          "marketing",
          "api",
        ]);
      });
  }, []);

  const selectedDisciplineConfig = value && DISCIPLINE_CONFIG[value as DisciplineKey];
  const SelectedIcon = selectedDisciplineConfig?.icon;

  return (
    <div className="space-y-2">
      <Label htmlFor="discipline">
        Discipline <span className="text-destructive">*</span>
      </Label>
      <Select value={value} onValueChange={onChange}>
        <SelectTrigger id="discipline">
          <SelectValue placeholder="Select a discipline">
            {selectedDisciplineConfig && SelectedIcon && (
              <div className="flex items-center gap-2">
                <SelectedIcon size={16} style={{ color: selectedDisciplineConfig.color }} />
                <span>{selectedDisciplineConfig.label}</span>
              </div>
            )}
          </SelectValue>
        </SelectTrigger>
        <SelectContent>
          {disciplines.map((d) => {
            const config = DISCIPLINE_CONFIG[d as DisciplineKey];
            const Icon = config?.icon;
            return (
              <SelectItem key={d} value={d}>
                <div className="flex items-center gap-2">
                  {Icon && <Icon size={16} style={{ color: config.color }} />}
                  <span>{config?.label || d.charAt(0).toUpperCase() + d.slice(1)}</span>
                </div>
              </SelectItem>
            );
          })}
        </SelectContent>
      </Select>
    </div>
  );
}
