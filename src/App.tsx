import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useState } from "react";
import { BottomBar } from "@/components/BottomBar";
import { ProjectSelector } from "@/components/ProjectSelector";
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from "@/components/ui/resizable";
import { Toaster } from "@/components/ui/sonner";
import { WorkspacePanel } from "@/components/WorkspacePanel";
import { useInvoke } from "@/hooks/useInvoke";
import type { Page } from "@/hooks/useNavigation";
import { DisciplinesPage } from "@/pages/DisciplinesPage";
import { FeaturesPage } from "@/pages/FeaturesPage";
import { TasksPage } from "@/pages/TasksPage";
import "./index.css";

function App() {
  const [lockedProject, setLockedProject] = useState<string | null>(null);
  const [currentPage, setCurrentPage] = useState<Page>("tasks");

  // Fetch locked project
  const { data: fetchedProject, isLoading: isLoadingProject } = useInvoke<string | null>("get_locked_project");

  // Sync fetched project to local state (allows onProjectSelected to update without refetch)
  useEffect(() => {
    if (fetchedProject !== undefined) {
      setLockedProject(fetchedProject);
    }
  }, [fetchedProject]);

  // Set window title when project changes
  useEffect(() => {
    if (lockedProject && typeof window !== "undefined" && "__TAURI__" in window) {
      const projectName = lockedProject.split("/").pop() || "Unknown";
      getCurrentWindow()
        .setTitle(`Ralph4days - ${projectName}`)
        .catch((err) => {
          console.error("Failed to set window title:", err);
        });
    }
  }, [lockedProject]);

  // Show loading or project picker
  if (isLoadingProject) {
    return (
      <div className="flex h-screen items-center justify-center">
        <div className="text-[hsl(var(--muted-foreground))]">Loading...</div>
      </div>
    );
  }

  if (!lockedProject) {
    return (
      <ProjectSelector
        onProjectSelected={async (project) => {
          setLockedProject(project);
          const projectName = project.split("/").pop() || "Unknown";
          try {
            await getCurrentWindow().setTitle(`Ralph4days - ${projectName}`);
          } catch (err) {
            console.error("Failed to set window title:", err);
          }
        }}
      />
    );
  }

  return (
    <>
      <ResizablePanelGroup direction="horizontal" className="h-screen">
        {/* Left: Pages */}
        <ResizablePanel defaultSize={50} minSize={40}>
          <div className="h-full flex flex-col overflow-hidden">
            <div className="flex-1 min-h-0 overflow-hidden relative">
              {/* Preload all pages, show/hide with CSS */}
              <div className={currentPage === "tasks" ? "h-full" : "hidden"}>
                <TasksPage />
              </div>
              <div className={currentPage === "features" ? "h-full" : "hidden"}>
                <FeaturesPage />
              </div>
              <div className={currentPage === "disciplines" ? "h-full" : "hidden"}>
                <DisciplinesPage />
              </div>
            </div>
            {/* Bottom Bar */}
            <BottomBar lockedProject={lockedProject} currentPage={currentPage} onPageChange={setCurrentPage} />
          </div>
        </ResizablePanel>

        <ResizableHandle withHandle />

        {/* Right: Output (always visible) */}
        <ResizablePanel defaultSize={50} minSize={20}>
          <div className="h-full">
            <WorkspacePanel />
          </div>
        </ResizablePanel>
      </ResizablePanelGroup>
      <Toaster />
    </>
  );
}

export default App;
