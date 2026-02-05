import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";

interface DisciplineSelectProps {
  value: string;
  onChange: (value: string) => void;
}

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

  return (
    <div className="space-y-2">
      <Label htmlFor="discipline">
        Discipline <span className="text-destructive">*</span>
      </Label>
      <Select value={value} onValueChange={onChange}>
        <SelectTrigger id="discipline">
          <SelectValue placeholder="Select a discipline" />
        </SelectTrigger>
        <SelectContent>
          {disciplines.map((d) => (
            <SelectItem key={d} value={d}>
              {d.charAt(0).toUpperCase() + d.slice(1)}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
