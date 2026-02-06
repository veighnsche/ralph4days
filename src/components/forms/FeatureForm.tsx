import { useEffect, useState } from "react";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import type { Feature } from "@/types/prd";

export interface FeatureFormData {
  name: string;
  display_name: string;
  acronym: string;
  description: string;
}

export interface FeatureFormProps {
  initialData?: Partial<Feature>;
  onSubmit: (data: FeatureFormData) => void;
  isSubmitting?: boolean;
}

/**
 * Feature creation/edit form.
 * Used by both create and edit modals (mode determined by initialData presence).
 */
export function FeatureForm({ initialData, onSubmit }: FeatureFormProps) {
  const [formData, setFormData] = useState<FeatureFormData>({
    name: initialData?.name || "",
    display_name: initialData?.display_name || "",
    acronym: "",
    description: initialData?.description || "",
  });

  useEffect(() => {
    if (initialData) {
      setFormData({
        name: initialData.name || "",
        display_name: initialData.display_name || "",
        acronym: "",
        description: initialData.description || "",
      });
    }
  }, [initialData]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSubmit(formData);
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      {/* Display Name */}
      <div className="space-y-2">
        <Label htmlFor="display_name">
          Display Name <span className="text-destructive">*</span>
        </Label>
        <Input
          id="display_name"
          value={formData.display_name}
          onChange={(e) => setFormData({ ...formData, display_name: e.target.value })}
          placeholder="Enter feature display name"
          required
        />
        <p className="text-xs text-muted-foreground">Human-readable name shown in the UI</p>
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
          placeholder="FTR (3-4 uppercase letters)"
          maxLength={4}
          required
          className="font-mono"
        />
        <p className="text-xs text-muted-foreground">3-4 uppercase letters for task IDs (e.g., AUTH, USER, PROD)</p>
      </div>

      {/* Name (Internal ID) */}
      <div className="space-y-2">
        <Label htmlFor="name">Internal Name</Label>
        <Input
          id="name"
          value={formData.name}
          onChange={(e) => setFormData({ ...formData, name: e.target.value })}
          placeholder="auto-generated-from-display-name"
          disabled={!!initialData?.name}
        />
        <p className="text-xs text-muted-foreground">
          {initialData?.name
            ? "Internal name cannot be changed after creation"
            : "Auto-generated (lowercase with hyphens)"}
        </p>
      </div>

      {/* Description */}
      <div className="space-y-2">
        <Label htmlFor="description">Description</Label>
        <Textarea
          id="description"
          value={formData.description}
          onChange={(e) => setFormData({ ...formData, description: e.target.value })}
          placeholder="Enter feature description"
          rows={4}
        />
      </div>
    </form>
  );
}
