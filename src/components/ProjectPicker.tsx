import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { FolderOpen, AlertCircle, CheckCircle } from "lucide-react";

interface RalphProject {
  name: string;
  path: string;
}

interface ProjectPickerProps {
  onProjectLocked: (path: string) => void;
}

export function ProjectPicker({ onProjectLocked }: ProjectPickerProps) {
  const [projects, setProjects] = useState<RalphProject[]>([]);
  const [scanning, setScanning] = useState(true);
  const [selectedPath, setSelectedPath] = useState("");
  const [validationStatus, setValidationStatus] = useState<"idle" | "validating" | "valid" | "error">("idle");
  const [validationError, setValidationError] = useState<string | null>(null);
  const [lockingProject, setLockingProject] = useState(false);

  // Scan for projects on mount
  useEffect(() => {
    if (typeof window !== 'undefined' && '__TAURI__' in window) {
      invoke<RalphProject[]>("scan_for_ralph_projects")
        .then((found) => {
          setProjects(found);
          if (found.length === 1) {
            // Auto-select if only one project found
            setSelectedPath(found[0].path);
          }
          setScanning(false);
        })
        .catch((err) => {
          console.error("Scan error:", err);
          setScanning(false);
        });
    } else {
      setScanning(false);
    }
  }, []);

  // Debounced validation
  useEffect(() => {
    if (!selectedPath) {
      setValidationStatus("idle");
      setValidationError(null);
      return;
    }

    setValidationStatus("validating");
    const timer = setTimeout(() => {
      invoke<void>("validate_project_path", { path: selectedPath })
        .then(() => {
          setValidationStatus("valid");
          setValidationError(null);
        })
        .catch((err) => {
          setValidationStatus("error");
          setValidationError(String(err));
        });
    }, 500);

    return () => clearTimeout(timer);
  }, [selectedPath]);

  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Project Directory",
      });
      if (selected && typeof selected === "string") {
        setSelectedPath(selected);
      }
    } catch (e) {
      console.error("Failed to open folder dialog:", e);
    }
  };

  const handleLockProject = useCallback(async () => {
    if (validationStatus !== "valid") return;

    setLockingProject(true);
    try {
      await invoke("set_locked_project", { path: selectedPath });
      onProjectLocked(selectedPath);
    } catch (err) {
      setValidationError(`Failed to lock project: ${err}`);
      setValidationStatus("error");
      setLockingProject(false);
    }
  }, [selectedPath, validationStatus, onProjectLocked]);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <Card className="w-[600px] max-w-[90vw]">
        <CardHeader>
          <CardTitle>Select Project</CardTitle>
          <p className="text-sm text-[hsl(var(--muted-foreground))]">
            Choose a Ralph project to lock for this session
          </p>
        </CardHeader>
        <CardContent className="space-y-4">
          {scanning && (
            <div className="text-sm text-[hsl(var(--muted-foreground))]">
              Scanning for .ralph projects...
            </div>
          )}

          {!scanning && projects.length > 1 && (
            <div className="space-y-2">
              <label className="text-sm font-medium">
                Discovered Projects
              </label>
              <select
                className="flex h-9 w-full border border-[hsl(var(--input))] bg-transparent px-3 py-1 text-sm"
                value={selectedPath}
                onChange={(e) => setSelectedPath(e.target.value)}
                disabled={lockingProject}
              >
                <option value="">-- Select a project --</option>
                {projects.map((project) => (
                  <option key={project.path} value={project.path}>
                    {project.name} ({project.path})
                  </option>
                ))}
              </select>
            </div>
          )}

          <div className="space-y-2">
            <label className="text-sm font-medium">
              Project Path
            </label>
            <div className="flex gap-2">
              <Input
                value={selectedPath}
                onChange={(e) => setSelectedPath(e.target.value)}
                placeholder="/path/to/project"
                disabled={lockingProject}
                className="flex-1"
              />
              <Button
                variant="outline"
                size="icon"
                onClick={handleSelectFolder}
                disabled={lockingProject}
                title="Browse for project"
              >
                <FolderOpen className="h-4 w-4" />
              </Button>
            </div>
          </div>

          {validationStatus === "validating" && (
            <div className="text-sm text-[hsl(var(--muted-foreground))]">
              Validating...
            </div>
          )}

          {validationStatus === "valid" && (
            <div className="flex items-center gap-2 text-sm text-green-600">
              <CheckCircle className="h-4 w-4" />
              Valid Ralph project
            </div>
          )}

          {validationStatus === "error" && validationError && (
            <div className="rounded-md border border-[hsl(var(--destructive))] bg-[hsl(var(--destructive))]/10 p-3">
              <div className="flex items-start gap-2 text-sm text-[hsl(var(--destructive))]">
                <AlertCircle className="h-4 w-4 mt-0.5 shrink-0" />
                <div className="space-y-2">
                  <div className="font-medium">Not a valid Ralph project</div>
                  <pre className="text-xs whitespace-pre-wrap font-mono">{validationError}</pre>
                  <div className="text-xs opacity-80">
                    Create the required structure, then try again.
                  </div>
                </div>
              </div>
            </div>
          )}

          <Button
            onClick={handleLockProject}
            disabled={validationStatus !== "valid" || lockingProject}
            className="w-full"
          >
            {lockingProject ? "Locking..." : "Lock Project"}
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
