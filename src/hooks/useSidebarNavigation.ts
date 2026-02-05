import { useCallback, useEffect, useState } from "react";
import type { PRDTask } from "@/types/prd";

export function useSidebarNavigation(tasks: PRDTask[]) {
  const [selectedTask, setSelectedTask] = useState<PRDTask | null>(null);
  const [sidebarOpen, setSidebarOpen] = useState(false);

  const handleTaskClick = useCallback((task: PRDTask) => {
    setSelectedTask(task);
    setSidebarOpen(true);
  }, []);

  const handleNavigateNext = useCallback(() => {
    if (!selectedTask) return;
    const currentIndex = tasks.findIndex((t) => t.id === selectedTask.id);
    if (currentIndex < tasks.length - 1) {
      setSelectedTask(tasks[currentIndex + 1]);
    }
  }, [selectedTask, tasks]);

  const handleNavigatePrev = useCallback(() => {
    if (!selectedTask) return;
    const currentIndex = tasks.findIndex((t) => t.id === selectedTask.id);
    if (currentIndex > 0) {
      setSelectedTask(tasks[currentIndex - 1]);
    }
  }, [selectedTask, tasks]);

  // Keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!sidebarOpen || !selectedTask) return;

      if (e.key === "ArrowUp" || e.key === "k") {
        e.preventDefault();
        handleNavigatePrev();
      } else if (e.key === "ArrowDown" || e.key === "j") {
        e.preventDefault();
        handleNavigateNext();
      } else if (e.key === "Escape") {
        e.preventDefault();
        setSidebarOpen(false);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [sidebarOpen, selectedTask, handleNavigateNext, handleNavigatePrev]);

  return {
    selectedTask,
    sidebarOpen,
    handleTaskClick,
    handleNavigateNext,
    handleNavigatePrev,
    setSidebarOpen,
  };
}
