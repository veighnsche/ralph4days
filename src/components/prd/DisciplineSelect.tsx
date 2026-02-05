import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useDisciplines } from "@/hooks/useDisciplines";

interface DisciplineSelectProps {
  value: string;
  onChange: (value: string) => void;
}

export function DisciplineSelect({ value, onChange }: DisciplineSelectProps) {
  const { disciplines, configMap } = useDisciplines();

  const selectedConfig = value ? configMap[value] : undefined;
  const SelectedIcon = selectedConfig?.icon;

  return (
    <div className="space-y-2">
      <Label htmlFor="discipline">
        Discipline <span className="text-destructive">*</span>
      </Label>
      <Select value={value} onValueChange={onChange}>
        <SelectTrigger id="discipline">
          <SelectValue placeholder="Select a discipline">
            {selectedConfig && SelectedIcon && (
              <div className="flex items-center gap-2">
                <SelectedIcon size={16} style={{ color: selectedConfig.color }} />
                <span>{selectedConfig.displayName}</span>
              </div>
            )}
          </SelectValue>
        </SelectTrigger>
        <SelectContent>
          {disciplines.map((d) => {
            const Icon = d.icon;
            return (
              <SelectItem key={d.name} value={d.name}>
                <div className="flex items-center gap-2">
                  <Icon size={16} style={{ color: d.color }} />
                  <span>{d.displayName}</span>
                </div>
              </SelectItem>
            );
          })}
        </SelectContent>
      </Select>
    </div>
  );
}
