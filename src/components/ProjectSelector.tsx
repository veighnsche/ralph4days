import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { InputGroup, InputGroupInput, InputGroupAddon, InputGroupButton } from "@/components/ui/input-group";
import { Field, FieldGroup, FieldLabel, FieldDescription } from "@/components/ui/field";
import { Separator } from "@/components/ui/separator";
import { FolderOpen } from "lucide-react";

interface RalphProject {
  name: string;
  path: string;
}

interface ProjectSelectorProps {
  onProjectSelected: (path: string) => void;
}

export function ProjectSelector({ onProjectSelected }: ProjectSelectorProps) {
  // Left side - Initialize
  const [initPath, setInitPath] = useState("");
  const [initializing, setInitializing] = useState(false);

  // Right side - Open existing
  const [projects, setProjects] = useState<RalphProject[]>([]);
  const [selectedProject, setSelectedProject] = useState("");
  const [scanning, setScanning] = useState(true);

  // Scan for projects on mount
  useEffect(() => {
    if (typeof window !== "undefined" && "__TAURI__" in window) {
      invoke<RalphProject[]>("scan_for_ralph_projects")
        .then((found) => {
          setProjects(found);
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

  const handleBrowseInit = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Directory to Initialize",
      });
      if (selected && typeof selected === "string") {
        setInitPath(selected);
      }
    } catch (e) {
      console.error("Failed to open folder dialog:", e);
    }
  };

  const handleInitialize = async () => {
    if (!initPath) return;

    setInitializing(true);
    try {
      const title = initPath.split("/").pop() || "Project";
      await invoke("initialize_ralph_project", { path: initPath, projectTitle: title });
      await invoke("set_locked_project", { path: initPath });
      onProjectSelected(initPath);
    } catch (err) {
      alert(`Failed to initialize: ${err}`);
      setInitializing(false);
    }
  };

  const handleOpenProject = async () => {
    if (!selectedProject) return;

    try {
      await invoke("set_locked_project", { path: selectedProject });
      onProjectSelected(selectedProject);
    } catch (err) {
      alert(`Failed to open: ${err}`);
    }
  };

  return (
    <Dialog open={true}>
      <DialogContent className="max-w-[900px]" showCloseButton={false}>
        <div className="grid grid-cols-[1fr_auto_1fr] gap-6">
          {/* Left Half - Initialize */}
          <div className="flex flex-col">
            <DialogTitle>Initialize Existing Project</DialogTitle>
            <FieldGroup className="flex-1">
              <div className="flex-1">
                <Field>
                  <FieldLabel>Project Directory</FieldLabel>
                  <InputGroup>
                    <InputGroupInput
                      value={initPath}
                      onChange={(e) => setInitPath(e.target.value)}
                      placeholder="/path/to/your-project"
                    />
                    <InputGroupAddon align="inline-end">
                      <InputGroupButton size="icon-xs" onClick={handleBrowseInit}>
                        <FolderOpen className="h-4 w-4" />
                      </InputGroupButton>
                    </InputGroupAddon>
                  </InputGroup>
                  <FieldDescription>Creates .ralph/ folder with template files</FieldDescription>
                </Field>
              </div>

              <Button onClick={handleInitialize} disabled={!initPath || initializing} className="w-full">
                {initializing ? "Initializing..." : "Initialize Ralph"}
              </Button>
            </FieldGroup>
          </div>

          <Separator orientation="vertical" />

          {/* Right Half - Open Existing */}
          <div className="flex flex-col">
            <DialogTitle>Open Existing Project</DialogTitle>
            <FieldGroup className="flex-1">
              {scanning ? (
                <FieldDescription>Scanning for Ralph projects...</FieldDescription>
              ) : (
                <>
                  <div className="flex-1">
                    <Field>
                      <FieldLabel>Discovered Projects ({projects.length})</FieldLabel>
                      <Select value={selectedProject} onValueChange={setSelectedProject}>
                        <SelectTrigger className="w-full">
                          <SelectValue placeholder="-- Select a project --" />
                        </SelectTrigger>
                        <SelectContent>
                          {projects.map((project) => (
                            <SelectItem key={project.path} value={project.path}>
                              {project.name}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </Field>
                  </div>

                  <Button onClick={handleOpenProject} disabled={!selectedProject} className="w-full">
                    Open Project
                  </Button>
                </>
              )}
            </FieldGroup>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
