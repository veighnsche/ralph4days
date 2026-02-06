import { useFormContext } from "react-hook-form";
import { FormControl, FormField, FormItem, FormLabel, FormMessage } from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import type { FeatureFormData } from "@/lib/schemas";

export function FeatureFormFields({ disabled, isEditing }: { disabled?: boolean; isEditing?: boolean }) {
  const { control } = useFormContext<FeatureFormData>();

  return (
    <div className="space-y-3">
      {/* Display Name */}
      <FormField
        control={control}
        name="displayName"
        render={({ field }) => (
          <FormItem>
            <FormLabel>
              Display Name <span className="text-destructive">*</span>
            </FormLabel>
            <FormControl>
              <Input {...field} placeholder="Enter feature display name" required disabled={disabled} />
            </FormControl>
            <p className="text-xs text-muted-foreground">Human-readable name shown in the UI</p>
            <FormMessage />
          </FormItem>
        )}
      />

      {/* Acronym */}
      <FormField
        control={control}
        name="acronym"
        render={({ field }) => (
          <FormItem>
            <FormLabel>
              Acronym <span className="text-destructive">*</span>
            </FormLabel>
            <FormControl>
              <Input
                {...field}
                onChange={(e) => field.onChange(e.target.value.toUpperCase())}
                placeholder="FTR (3-4 uppercase letters)"
                maxLength={4}
                required
                className="font-mono"
                disabled={disabled}
              />
            </FormControl>
            <p className="text-xs text-muted-foreground">3-4 uppercase letters for task IDs (e.g., AUTH, USER, PROD)</p>
            <FormMessage />
          </FormItem>
        )}
      />

      {/* Name (Internal ID) */}
      <FormField
        control={control}
        name="name"
        render={({ field }) => (
          <FormItem>
            <FormLabel>Internal Name</FormLabel>
            <FormControl>
              <Input {...field} placeholder="auto-generated-from-display-name" disabled={disabled || isEditing} />
            </FormControl>
            <p className="text-xs text-muted-foreground">
              {isEditing ? "Internal name cannot be changed after creation" : "Auto-generated (lowercase with hyphens)"}
            </p>
            <FormMessage />
          </FormItem>
        )}
      />

      {/* Description */}
      <FormField
        control={control}
        name="description"
        render={({ field }) => (
          <FormItem>
            <FormLabel>Description</FormLabel>
            <FormControl>
              <Textarea {...field} placeholder="Enter feature description" rows={4} disabled={disabled} />
            </FormControl>
            <FormMessage />
          </FormItem>
        )}
      />
    </div>
  );
}
