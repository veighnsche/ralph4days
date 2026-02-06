import { X } from "lucide-react";
import { useEffect, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { NativeSelect } from "@/components/ui/native-select";
import { Textarea } from "@/components/ui/textarea";
import { useDisciplines } from "@/hooks/useDisciplines";
import { useInvoke } from "@/hooks/useInvoke";
import type { Feature, PRDTask } from "@/types/prd";

export interface TaskFormData {
  feature: string;
  discipline: string;
  title: string;
  description: string;
  priority: string;
  tags: string[];
  depends_on: number[];
  acceptance_criteria: string[];
}

export interface TaskFormProps {
  initialData?: Partial<PRDTask>;
  onChange: (data: TaskFormData) => void;
  disabled?: boolean;
}

/**
 * Task creation/edit form.
 * Used by both create and edit modals (mode determined by initialData presence).
 */
export function TaskForm({ initialData, onChange, disabled }: TaskFormProps) {
  const { disciplines } = useDisciplines();
  const { data: features = [] } = useInvoke<Feature[]>("get_features");

  const [formData, setFormData] = useState<TaskFormData>({
    feature: initialData?.feature || "",
    discipline: initialData?.discipline || "",
    title: initialData?.title || "",
    description: initialData?.description || "",
    priority: initialData?.priority || "medium",
    tags: initialData?.tags || [],
    depends_on: initialData?.depends_on || [],
    acceptance_criteria: initialData?.acceptance_criteria || [],
  });

  const [newTag, setNewTag] = useState("");
  const [newCriterion, setNewCriterion] = useState("");

  useEffect(() => {
    if (initialData) {
      setFormData({
        feature: initialData.feature || "",
        discipline: initialData.discipline || "",
        title: initialData.title || "",
        description: initialData.description || "",
        priority: initialData.priority || "medium",
        tags: initialData.tags || [],
        depends_on: initialData.depends_on || [],
        acceptance_criteria: initialData.acceptance_criteria || [],
      });
    }
  }, [initialData]);

  useEffect(() => {
    onChange(formData);
  }, [formData, onChange]);

  const addTag = () => {
    if (newTag.trim() && !formData.tags.includes(newTag.trim())) {
      setFormData({ ...formData, tags: [...formData.tags, newTag.trim()] });
      setNewTag("");
    }
  };

  const removeTag = (tag: string) => {
    setFormData({ ...formData, tags: formData.tags.filter((t) => t !== tag) });
  };

  const addCriterion = () => {
    if (newCriterion.trim()) {
      setFormData({
        ...formData,
        acceptance_criteria: [...formData.acceptance_criteria, newCriterion.trim()],
      });
      setNewCriterion("");
    }
  };

  const removeCriterion = (index: number) => {
    setFormData({
      ...formData,
      acceptance_criteria: formData.acceptance_criteria.filter((_, i) => i !== index),
    });
  };

  return (
    <div className="space-y-4">
      {/* Feature */}
      <div className="space-y-2">
        <Label htmlFor="feature">
          Feature <span className="text-destructive">*</span>
        </Label>
        {features.length > 0 ? (
          <NativeSelect
            id="feature"
            value={formData.feature}
            onChange={(e) => setFormData({ ...formData, feature: e.target.value })}
            required
            disabled={disabled}
          >
            <option value="">Select a feature...</option>
            {features.map((feature) => (
              <option key={feature.name} value={feature.name}>
                {feature.display_name}
              </option>
            ))}
          </NativeSelect>
        ) : (
          <Input
            id="feature"
            value={formData.feature}
            onChange={(e) => setFormData({ ...formData, feature: e.target.value })}
            placeholder="Enter feature name"
            required
            disabled={disabled}
          />
        )}
      </div>

      {/* Discipline */}
      <div className="space-y-2">
        <Label htmlFor="discipline">
          Discipline <span className="text-destructive">*</span>
        </Label>
        <NativeSelect
          id="discipline"
          value={formData.discipline}
          onChange={(e) => setFormData({ ...formData, discipline: e.target.value })}
          required
          disabled={disabled}
        >
          <option value="">Select a discipline...</option>
          {disciplines.map((discipline) => (
            <option key={discipline.name} value={discipline.name}>
              {discipline.displayName}
            </option>
          ))}
        </NativeSelect>
      </div>

      {/* Title */}
      <div className="space-y-2">
        <Label htmlFor="title">
          Title <span className="text-destructive">*</span>
        </Label>
        <Input
          id="title"
          value={formData.title}
          onChange={(e) => setFormData({ ...formData, title: e.target.value })}
          placeholder="Enter task title"
          required
          disabled={disabled}
        />
      </div>

      {/* Description */}
      <div className="space-y-2">
        <Label htmlFor="description">Description</Label>
        <Textarea
          id="description"
          value={formData.description}
          onChange={(e) => setFormData({ ...formData, description: e.target.value })}
          placeholder="Enter task description"
          rows={4}
          disabled={disabled}
        />
      </div>

      {/* Priority */}
      <div className="space-y-2">
        <Label htmlFor="priority">Priority</Label>
        <NativeSelect
          id="priority"
          value={formData.priority}
          onChange={(e) => setFormData({ ...formData, priority: e.target.value })}
          disabled={disabled}
        >
          <option value="low">Low</option>
          <option value="medium">Medium</option>
          <option value="high">High</option>
          <option value="critical">Critical</option>
        </NativeSelect>
      </div>

      {/* Tags */}
      <div className="space-y-2">
        <Label>Tags</Label>
        <div className="flex gap-2">
          <Input
            value={newTag}
            onChange={(e) => setNewTag(e.target.value)}
            placeholder="Add a tag"
            disabled={disabled}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.preventDefault();
                addTag();
              }
            }}
          />
          <Button type="button" onClick={addTag} variant="outline" disabled={disabled}>
            Add
          </Button>
        </div>
        {formData.tags.length > 0 && (
          <div className="flex flex-wrap gap-2 mt-2">
            {formData.tags.map((tag) => (
              <Badge key={tag} variant="secondary" className="gap-1">
                {tag}
                <button
                  type="button"
                  onClick={() => removeTag(tag)}
                  className="hover:text-destructive"
                  disabled={disabled}
                >
                  <X className="h-3 w-3" />
                </button>
              </Badge>
            ))}
          </div>
        )}
      </div>

      {/* Acceptance Criteria */}
      <div className="space-y-2">
        <Label>Acceptance Criteria</Label>
        <div className="flex gap-2">
          <Input
            value={newCriterion}
            onChange={(e) => setNewCriterion(e.target.value)}
            placeholder="Add acceptance criterion"
            disabled={disabled}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.preventDefault();
                addCriterion();
              }
            }}
          />
          <Button type="button" onClick={addCriterion} variant="outline" disabled={disabled}>
            Add
          </Button>
        </div>
        {formData.acceptance_criteria.length > 0 && (
          <ul className="space-y-1 mt-2">
            {formData.acceptance_criteria.map((criterion, index) => (
              <li key={index} className="flex items-start gap-2 text-sm">
                <span className="flex-1">{criterion}</span>
                <button
                  type="button"
                  onClick={() => removeCriterion(index)}
                  className="text-muted-foreground hover:text-destructive"
                  disabled={disabled}
                >
                  <X className="h-4 w-4" />
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}
