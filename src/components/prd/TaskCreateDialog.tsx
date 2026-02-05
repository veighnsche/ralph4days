import { invoke } from "@tauri-apps/api/core";
import { Plus } from "lucide-react";
import { useState } from "react";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { DisciplineSelect } from "./DisciplineSelect";
import { TaskIdPreview } from "./TaskIdPreview";

interface TaskCreateDialogProps {
  onTaskCreated: () => void;
}

export function TaskCreateDialog({ onTaskCreated }: TaskCreateDialogProps) {
  const [open, setOpen] = useState(false);
  const [feature, setFeature] = useState("");
  const [discipline, setDiscipline] = useState("");
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [priority, setPriority] = useState<"low" | "medium" | "high" | "critical">("medium");
  const [tags, setTags] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const resetForm = () => {
    setFeature("");
    setDiscipline("");
    setTitle("");
    setDescription("");
    setPriority("medium");
    setTags("");
    setError(null);
  };

  const handleCreate = async () => {
    // Validate inputs
    if (!feature.trim()) {
      setError("Feature is required");
      return;
    }
    if (!discipline) {
      setError("Discipline is required");
      return;
    }
    if (!title.trim()) {
      setError("Title is required");
      return;
    }

    // Validate feature format (before normalization)
    if (feature.includes("/")) {
      setError("Feature cannot contain slashes");
      return;
    }

    setLoading(true);
    setError(null);

    try {
      // BUG FIX: Proper tag parsing
      const tagArray = tags
        .split(",")
        .map((t) => t.trim())
        .filter((t) => t.length > 0);

      const taskId = await invoke<string>("create_task", {
        feature: feature.trim(),
        discipline,
        title: title.trim(),
        description: description.trim() || null,
        priority,
        tags: tagArray,
      });

      console.log("Task created:", taskId);
      onTaskCreated();
      resetForm();
      setOpen(false);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={(newOpen) => {
        setOpen(newOpen);
        if (!newOpen) resetForm();
      }}
    >
      <DialogTrigger asChild>
        <Button size="sm" variant="default">
          <Plus className="h-4 w-4 mr-2" />
          New Task
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>Create New Task</DialogTitle>
          <DialogDescription>Add a new task to your PRD. The task ID will be auto-generated.</DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* Task ID Preview */}
          <TaskIdPreview feature={feature} discipline={discipline} />

          {/* Feature Input */}
          <div className="space-y-2">
            <Label htmlFor="feature">
              Feature <span className="text-destructive">*</span>
            </Label>
            <Input
              id="feature"
              placeholder="e.g., auth, search, profile"
              value={feature}
              onChange={(e) => setFeature(e.target.value)}
              autoComplete="off"
            />
            <p className="text-xs text-muted-foreground">Will be auto-formatted to lowercase with hyphens</p>
          </div>

          {/* Discipline Select */}
          <DisciplineSelect value={discipline} onChange={setDiscipline} />

          {/* Title Input */}
          <div className="space-y-2">
            <Label htmlFor="title">
              Title <span className="text-destructive">*</span>
            </Label>
            <Input
              id="title"
              placeholder="Brief task description"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
            />
          </div>

          {/* Description Textarea */}
          <div className="space-y-2">
            <Label htmlFor="description">Description</Label>
            <Textarea
              id="description"
              placeholder="Detailed task description..."
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={3}
            />
          </div>

          {/* Priority Select */}
          <div className="space-y-2">
            <Label htmlFor="priority">Priority</Label>
            <Select value={priority} onValueChange={(v) => setPriority(v as "low" | "medium" | "high" | "critical")}>
              <SelectTrigger id="priority">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="low">Low</SelectItem>
                <SelectItem value="medium">Medium</SelectItem>
                <SelectItem value="high">High</SelectItem>
                <SelectItem value="critical">Critical</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Tags Input */}
          <div className="space-y-2">
            <Label htmlFor="tags">Tags</Label>
            <Input
              id="tags"
              placeholder="comma, separated, tags"
              value={tags}
              onChange={(e) => setTags(e.target.value)}
            />
          </div>

          {/* Error Display */}
          {error && (
            <Alert variant="destructive">
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => {
              resetForm();
              setOpen(false);
            }}
          >
            Cancel
          </Button>
          <Button onClick={handleCreate} disabled={loading}>
            {loading ? "Creating..." : "Create Task"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
